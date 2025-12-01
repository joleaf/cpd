use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use super::graph_matching::{AlgoGraphMatching, MatchingResult};
use crate::data::graph::Graph;
use dashmap::DashMap;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct PatternResult {
    pub pattern: Graph,
    pub frequency_exact: usize,
    pub frequency_relaxed: usize,
}

#[derive(Debug)]
pub enum AlgoCandidateMatching {
    Naive,
    Parallel,
}

impl AlgoCandidateMatching {
    pub fn run_matching(
        &self,
        candidates: &[Vec<Graph>],
        algo_graph_matching: &AlgoGraphMatching,
        support_exact: usize,
        support_relaxed: usize,
    ) -> Vec<PatternResult> {
        let mut result = match self {
            AlgoCandidateMatching::Naive => run_naive(
                candidates,
                algo_graph_matching,
                support_exact,
                support_relaxed,
            ),
            AlgoCandidateMatching::Parallel => run_parallel(
                candidates,
                algo_graph_matching,
                support_exact,
                support_relaxed,
            ),
        };
        // Update ids of graphs
        for (id_gen, pattern_result) in result.iter_mut().enumerate() {
            pattern_result.pattern.id = id_gen;
        }
        result
    }
}

fn run_naive(
    candidates: &[Vec<Graph>],
    algo_graph_matching: &AlgoGraphMatching,
    support_exact: usize,
    support_relaxed: usize,
) -> Vec<PatternResult> {
    let mut resulting_candidates = Vec::new();
    let mut can_be_skipped: HashSet<usize> = HashSet::new();
    let mut match_results: HashMap<(usize, usize), MatchingResult> = HashMap::new();
    let mut matches: Vec<usize> = Vec::new();
    for (i_a, candidates_of_graph_a) in candidates.iter().enumerate() {
        for candidate_a in candidates_of_graph_a.iter() {
            if can_be_skipped.contains(&candidate_a.id) {
                continue;
            }
            let mut freq_exact = 1;
            let mut freq_relaxed = 1;
            matches.clear();
            matches.push(candidate_a.id);
            for (i_b, candidates_of_graph_b) in candidates.iter().enumerate() {
                // Do not compare the graphs of the same graph
                if i_a == i_b {
                    continue;
                }
                let mut exact_match_id: Option<usize> = None;
                let mut found_relaxed_match = false;
                for candidate_b in candidates_of_graph_b.iter() {
                    let key = if candidate_a.id < candidate_b.id {
                        (candidate_a.id, candidate_b.id)
                    } else {
                        (candidate_b.id, candidate_a.id)
                    };
                    let match_result = match_results.entry(key).or_insert_with(|| {
                        algo_graph_matching.match_graphs(candidate_a, candidate_b)
                    });
                    match match_result {
                        MatchingResult::ExactMatch => {
                            exact_match_id = Some(candidate_b.id);
                            break; // We do not need search for other relaxed or exact matches
                        }
                        MatchingResult::RelaxedMatch => found_relaxed_match = true,
                        MatchingResult::NoMatch => {
                            // Nothing
                        }
                    }
                }
                if let Some(t_id) = exact_match_id {
                    freq_exact += 1;
                    freq_relaxed += 1;
                    matches.push(t_id);
                } else if found_relaxed_match {
                    freq_relaxed += 1;
                }
            }
            if freq_exact >= support_exact || freq_relaxed >= support_relaxed {
                resulting_candidates.push(PatternResult {
                    pattern: (*candidate_a).clone(),
                    frequency_exact: freq_exact,
                    frequency_relaxed: freq_relaxed,
                });
            }
            can_be_skipped.extend(&matches);
        }
    }
    resulting_candidates
}

fn run_parallel(
    candidates: &[Vec<Graph>],
    algo_graph_matching: &AlgoGraphMatching,
    support_exact: usize,
    support_relaxed: usize,
) -> Vec<PatternResult> {
    // Symmetric match result cache
    let match_cache = Arc::new(DashMap::<(usize, usize), MatchingResult>::new());

    // Parallel map over all groups
    let all_results: Vec<PatternResult> = candidates
        .par_iter()
        .enumerate()
        .flat_map(|(i_a, candidates_of_graph_a)| {
            let mut local = Vec::with_capacity(candidates_of_graph_a.len() / 4);

            for candidate_a in candidates_of_graph_a.iter() {
                let mut freq_exact = 1;
                let mut freq_relaxed = 1;

                // Check all other groups
                for (i_b, candidates_of_graph_b) in candidates.iter().enumerate() {
                    if i_a == i_b {
                        continue;
                    }

                    let mut exact_match_found = false;
                    let mut relaxed_found = false;

                    for candidate_b in candidates_of_graph_b.iter() {
                        let (a, b) = if candidate_a.id < candidate_b.id {
                            (candidate_a.id, candidate_b.id)
                        } else {
                            (candidate_b.id, candidate_a.id)
                        };

                        let result = *match_cache.entry((a, b)).or_insert_with(|| {
                            algo_graph_matching.match_graphs(candidate_a, candidate_b)
                        });

                        match result {
                            MatchingResult::ExactMatch => exact_match_found = true,
                            MatchingResult::RelaxedMatch => {
                                relaxed_found = true;
                            }
                            MatchingResult::NoMatch => {
                                // Nothing
                            }
                        }
                    }

                    if exact_match_found {
                        freq_exact += 1;
                        freq_relaxed += 1;
                    } else if relaxed_found {
                        freq_relaxed += 1;
                    }
                }

                if freq_exact >= support_exact || freq_relaxed >= support_relaxed {
                    local.push(PatternResult {
                        pattern: candidate_a.clone(),
                        frequency_exact: freq_exact,
                        frequency_relaxed: freq_relaxed,
                    });
                }
            }

            local
        })
        .collect();
    let mut unique = Vec::with_capacity(all_results.len());
    let mut visited = HashSet::<usize>::new();

    for (i_one_graph, one_graph) in all_results.iter().enumerate() {
        if visited.contains(&one_graph.pattern.id) {
            continue;
        }
        visited.insert(one_graph.pattern.id);

        for i_other_graph in (i_one_graph + 1)..all_results.len() {
            let other_graph = all_results.get(i_other_graph).unwrap();
            if one_graph.pattern.id == other_graph.pattern.id {
                continue;
            }

            let key = if one_graph.pattern.id < other_graph.pattern.id {
                (one_graph.pattern.id, other_graph.pattern.id)
            } else {
                (other_graph.pattern.id, one_graph.pattern.id)
            };

            if let Some(result) = match_cache.get(&key)
                && *result == MatchingResult::ExactMatch
            {
                visited.insert(other_graph.pattern.id);
            }
        }
        unique.push(one_graph.clone());
    }
    unique
}
