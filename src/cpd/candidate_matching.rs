use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use super::{
    candidate_generation::Candidate,
    graph_matching::{AlgoGraphMatching, MatchingResult},
};
use crate::data::graph::Graph;
use dashmap::DashMap;
use rayon::prelude::*;

#[derive(Debug, Clone)]
/// Represents the result of evaluating a candidate pattern across a collection of graphs.
///
/// A pattern refers to a subgraph candidate that appears either exactly or in a relaxed form
/// across multiple input graphs. The frequencies record how often this pattern appears across
/// those graphs.
///
/// # Fields
/// - `pattern`: The subgraph pattern itself.
/// - `frequency_exact`: Number of graphs in which the pattern appears as an **exact match**.
/// - `frequency_relaxed`: Number of graphs in which the pattern appears as either an
///   **exact match** or a **relaxed match**.
///
/// # Notes
/// - The `pattern.id` value is reassigned after matching to ensure that resulting patterns
///   receive unique, consecutive identifiers.
/// - Relaxed matching criteria depend on the selected `AlgoGraphMatching` strategy.
pub struct PatternResult {
    pub pattern: Graph,
    pub frequency_exact: usize,
    pub frequency_relaxed: usize,
}

/// Specifies the strategy used to perform pairwise matching between candidate subgraphs.
///
/// Matching determines whether two candidate graphs represent the same underlying pattern,
/// either **exactly** or in a **relaxed** manner. The matching strategy affects runtime
/// performance but not correctness.
///
/// # Variants
///
/// - `Naive`:
///   Performs matching sequentially. Suitable for small datasets; easier to debug.
/// - `Parallel`:
///   Uses Rayon for parallel iteration and DashMap for a shared symmetric match cache.
///   Recommended for large candidate sets or many input graphs.
///
/// The matching logic itself is provided by `AlgoGraphMatching`.
#[derive(Debug)]
pub enum AlgoCandidateMatching {
    Naive,
    Parallel,
}

impl AlgoCandidateMatching {
    /// Executes the matching process for all candidate subgraphs across all input graphs.
    ///
    /// Each candidate from graph *A* is compared to candidates from every *other* graph using
    /// the selected `AlgoGraphMatching` strategy. If a candidate satisfies either the exact
    /// or relaxed support threshold, it is included in the final result as a `PatternResult`.
    ///
    /// # Arguments
    ///
    /// * `candidates` — A slice where each element contains all candidate subgraphs generated
    ///   from one input graph.
    /// * `algo_graph_matching` — The matching algorithm used to compare two graphs.
    /// * `support_exact` — Minimum number of exact matches required for a candidate to be kept.
    /// * `support_relaxed` — Minimum number of relaxed-or-exact matches required.
    ///
    /// # Returns
    ///
    /// A flattened `Vec<PatternResult>` containing all subgraph patterns that reach the required
    /// frequency thresholds. The IDs of resulting pattern graphs are reassigned to ensure a
    /// consecutive sequence starting at zero.
    pub fn run_matching(
        &self,
        candidates: &[Vec<Vec<Candidate>>],
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
    candidates: &[Vec<Vec<Candidate>>],
    algo_graph_matching: &AlgoGraphMatching,
    support_exact: usize,
    support_relaxed: usize,
) -> Vec<PatternResult> {
    let mut all_results = Vec::new();
    let mut match_results: HashMap<(usize, usize), MatchingResult> = HashMap::new();

    for candidates_of_graph_a in candidates.iter() {
        for candidate_n_a in candidates_of_graph_a.iter() {
            for candidate_a in candidate_n_a.iter() {
                let mut freq_relaxed = 0;
                let mut freq_exact = 0;

                for candidates_of_graph_b in candidates.iter() {
                    let candidates_of_graph_b_iter: Box<
                        dyn std::iter::Iterator<Item = &Candidate>,
                    > = Box::new(candidates_of_graph_b.iter().flatten());

                    for candidate_b in candidates_of_graph_b_iter {
                        let key = if candidate_a.graph.id < candidate_b.graph.id {
                            (candidate_a.graph.id, candidate_b.graph.id)
                        } else {
                            (candidate_b.graph.id, candidate_a.graph.id)
                        };

                        let match_result = match_results.entry(key).or_insert_with(|| {
                            algo_graph_matching.match_graphs(&candidate_a.graph, &candidate_b.graph)
                        });

                        match *match_result {
                            MatchingResult::ExactMatch => {
                                freq_relaxed += 1;
                                freq_exact += 1;
                            }
                            MatchingResult::RelaxedMatch => freq_relaxed += 1,
                            MatchingResult::NoMatch => {}
                        }
                    }
                }

                if freq_exact >= support_exact || freq_relaxed >= support_relaxed {
                    all_results.push(PatternResult {
                        pattern: candidate_a.graph.clone(),
                        frequency_exact: freq_exact,
                        frequency_relaxed: freq_relaxed,
                    });
                }
            }
        }
    }

    deduplicate_exact_matches(all_results, &match_results)
}

fn run_parallel(
    candidates: &[Vec<Vec<Candidate>>],
    algo_graph_matching: &AlgoGraphMatching,
    support_exact: usize,
    support_relaxed: usize,
) -> Vec<PatternResult> {
    let match_cache = Arc::new(DashMap::<(usize, usize), MatchingResult>::new());

    let mut flat_candidates = Vec::new();
    let mut candidate_indices = Vec::new();

    for graph_candidates in candidates {
        let mut graph_indices = Vec::new();
        for size_group in graph_candidates {
            let mut size_indices = Vec::new();
            for candidate in size_group {
                size_indices.push(flat_candidates.len());
                flat_candidates.push(candidate);
            }
            graph_indices.push(size_indices);
        }
        candidate_indices.push(graph_indices);
    }

    let total_candidates = flat_candidates.len();
    let freq_exact = Arc::new(
        (0..total_candidates)
            .map(|_| AtomicUsize::new(0))
            .collect::<Vec<_>>(),
    );
    let freq_relaxed = Arc::new(
        (0..total_candidates)
            .map(|_| AtomicUsize::new(0))
            .collect::<Vec<_>>(),
    );

    (0..total_candidates).into_par_iter().for_each(|i| {
        for j in i + 1..total_candidates {
            process_match(
                i,
                j,
                flat_candidates[i],
                flat_candidates[j],
                algo_graph_matching,
                &match_cache,
                &freq_exact,
                &freq_relaxed,
            );
        }
    });

    let mut all_results = Vec::new();
    for (idx, candidate) in flat_candidates.iter().enumerate() {
        let f_exact = freq_exact[idx].load(Ordering::Relaxed) + 1;
        let f_relaxed = freq_relaxed[idx].load(Ordering::Relaxed) + 1;

        if f_exact >= support_exact || f_relaxed >= support_relaxed {
            all_results.push(PatternResult {
                pattern: candidate.graph.clone(),
                frequency_exact: f_exact,
                frequency_relaxed: f_relaxed,
            });
        }
    }

    // Use the cache to deduplicate
    let cache_map: HashMap<(usize, usize), MatchingResult> =
        match_cache.iter().map(|r| (*r.key(), *r.value())).collect();
    deduplicate_exact_matches(all_results, &cache_map)
}

fn deduplicate_exact_matches(
    all_results: Vec<PatternResult>,
    match_cache: &HashMap<(usize, usize), MatchingResult>,
) -> Vec<PatternResult> {
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

#[allow(clippy::too_many_arguments)]
fn process_match(
    idx_a: usize,
    idx_b: usize,
    candidate_a: &Candidate,
    candidate_b: &Candidate,
    algo_graph_matching: &AlgoGraphMatching,
    match_cache: &DashMap<(usize, usize), MatchingResult>,
    freq_exact: &[AtomicUsize],
    freq_relaxed: &[AtomicUsize],
) {
    let (id_a, id_b) = if candidate_a.graph.id < candidate_b.graph.id {
        (candidate_a.graph.id, candidate_b.graph.id)
    } else {
        (candidate_b.graph.id, candidate_a.graph.id)
    };

    let result = if let Some(res) = match_cache.get(&(id_a, id_b)) {
        *res
    } else {
        *match_cache.entry((id_a, id_b)).or_insert_with(|| {
            algo_graph_matching.match_graphs(&candidate_a.graph, &candidate_b.graph)
        })
    };

    match result {
        MatchingResult::ExactMatch => {
            freq_exact[idx_a].fetch_add(1, Ordering::Relaxed);
            freq_exact[idx_b].fetch_add(1, Ordering::Relaxed);
            freq_relaxed[idx_a].fetch_add(1, Ordering::Relaxed);
            freq_relaxed[idx_b].fetch_add(1, Ordering::Relaxed);
        }
        MatchingResult::RelaxedMatch => {
            freq_relaxed[idx_a].fetch_add(1, Ordering::Relaxed);
            freq_relaxed[idx_b].fetch_add(1, Ordering::Relaxed);
        }
        MatchingResult::NoMatch => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpd::graph_matching::AlgoGraphMatching;
    use crate::data::graph::Graph;

    fn make_graph(id: usize) -> Graph {
        let mut g = Graph::new(id);
        g.create_vertex_with_data(1, 2);
        g.create_vertex_with_data(2, 2);
        g.vertices.get_mut(0).unwrap().push(1, 10);
        g
    }

    #[test]
    fn test_parallel_vs_naive_matching() {
        let g1 = make_graph(1);
        let g2 = make_graph(2);
        let g3 = make_graph(3);

        let c1 = Candidate {
            n_activity: 2,
            id_parent: 1,
            graph: g1.clone(),
        };
        let c2 = Candidate {
            n_activity: 2,
            id_parent: 2,
            graph: g2.clone(),
        };
        let c3 = Candidate {
            n_activity: 2,
            id_parent: 3,
            graph: g3.clone(),
        };

        let candidates = vec![vec![vec![c1]], vec![vec![c2]], vec![vec![c3]]];

        let algo_matching = AlgoGraphMatching::VF2IsomorphismTest;
        let support_exact = 2;
        let support_relaxed = 2;

        let naive_matcher = AlgoCandidateMatching::Naive;
        let parallel_matcher = AlgoCandidateMatching::Parallel;

        let res_naive =
            naive_matcher.run_matching(&candidates, &algo_matching, support_exact, support_relaxed);
        let res_parallel = parallel_matcher.run_matching(
            &candidates,
            &algo_matching,
            support_exact,
            support_relaxed,
        );

        assert_eq!(res_naive.len(), res_parallel.len());
        for i in 0..res_naive.len() {
            assert_eq!(
                res_naive[i].frequency_exact,
                res_parallel[i].frequency_exact
            );
            assert_eq!(
                res_naive[i].frequency_relaxed,
                res_parallel[i].frequency_relaxed
            );
        }
    }
}
