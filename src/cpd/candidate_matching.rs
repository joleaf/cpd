use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
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
    ///
    /// # Example
    ///
    /// The following example builds two simple graphs that share the same structure and then
    /// runs exact/relaxed matching between their candidates:
    ///
    /// ```rust
    /// use crate::candidate_matching::{AlgoCandidateMatching, PatternResult};
    /// use crate::graph_matching::{AlgoGraphMatching, MatchingResult};
    /// use crate::data::graph::Graph;
    ///
    /// // Build two small graphs with identical structure
    /// fn make_graph(id: usize) -> Graph {
    ///     let mut g = Graph::new(id);
    ///     // Three activity vertices
    ///     g.create_vertex_with_data(1, 2);
    ///     g.create_vertex_with_data(2, 2);
    ///     g.create_vertex_with_data(3, 2);
    ///     // Fully connect them
    ///     g.vertices.get_mut(0).unwrap().push(1, 0);
    ///     g.vertices.get_mut(1).unwrap().push(2, 0);
    ///     g.vertices.get_mut(2).unwrap().push(0, 0);
    ///     g
    /// }
    ///
    /// let g1 = make_graph(1);
    /// let g2 = make_graph(2);
    ///
    /// // Each graph contributes its own set of candidates
    /// // Here we pretend each graph itself is a single candidate
    /// let candidates = vec![
    ///     vec![g1.clone()],
    ///     vec![g2.clone()],
    /// ];
    ///
    /// // Matching algorithm based on vertex/edge cosine similarity
    /// let matcher_algo = AlgoGraphMatching::CosineSimilarity {
    ///     alpha: 0.5,
    ///     matching_threshold: 0.8,
    /// };
    ///
    /// // Use naive matching for simplicity
    /// let matcher = AlgoCandidateMatching::Naive;
    ///
    /// let patterns: Vec<PatternResult> =
    ///     matcher.run_matching(&candidates, &matcher_algo, 1, 1);
    ///
    /// assert_eq!(patterns.len(), 1);
    /// assert_eq!(patterns[0].frequency_exact, 2);  // g1 matches g2 exactly
    ///
    /// println!(
    ///     "Discovered pattern with new id {}, occurring {} exact times",
    ///     patterns[0].pattern.id,
    ///     patterns[0].frequency_exact
    /// );
    /// ```
    ///
    /// # Notes
    ///
    /// - The method performs candidate-to-candidate comparisons across **different** input graphs only.
    /// - In parallel mode, a shared symmetric cache ensures each pair of graphs is matched at most once.
    /// - After matching, pattern IDs are reassigned to ensure stable ordering in output.
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
    let mut resulting_candidates = Vec::new();
    let mut can_be_skipped: HashSet<usize> = HashSet::new();
    let mut match_results: HashMap<(usize, usize), MatchingResult> = HashMap::new();
    let mut matches: Vec<usize> = Vec::new();
    for (i_a, candidates_of_graph_a) in candidates.iter().enumerate() {
        for (i_n_a, candidate_n_a) in candidates_of_graph_a.iter().enumerate() {
            for candidate_a in candidate_n_a.iter() {
                if can_be_skipped.contains(&candidate_a.graph.id) {
                    continue;
                }
                let mut freq_exact = 1;
                let mut freq_relaxed = 1;
                matches.clear();
                matches.push(candidate_a.graph.id);
                for (i_b, candidates_of_graph_b) in candidates.iter().enumerate() {
                    // Do not compare the graphs of the same graph
                    if i_a == i_b {
                        continue;
                    }
                    // Only compare graphs of the same n size :)
                    let candidates_of_graph_b: &Vec<Candidate> =
                        candidates_of_graph_b.get(i_n_a).unwrap();
                    let mut exact_match_id: Option<usize> = None;
                    let mut found_relaxed_match = false;
                    for candidate_b in candidates_of_graph_b.iter() {
                        let key = if candidate_a.graph.id < candidate_b.graph.id {
                            (candidate_a.graph.id, candidate_b.graph.id)
                        } else {
                            (candidate_b.graph.id, candidate_a.graph.id)
                        };
                        let match_result = match_results.entry(key).or_insert_with(|| {
                            algo_graph_matching.match_graphs(&candidate_a.graph, &candidate_b.graph)
                        });
                        match match_result {
                            MatchingResult::ExactMatch => {
                                exact_match_id = Some(candidate_b.graph.id);
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
                        pattern: candidate_a.graph.clone(),
                        frequency_exact: freq_exact,
                        frequency_relaxed: freq_relaxed,
                    });
                }
                can_be_skipped.extend(&matches);
            }
        }
    }
    resulting_candidates
}

fn run_parallel(
    candidates: &[Vec<Vec<Candidate>>],
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

            for (i_n_a, candidate_n_a) in candidates_of_graph_a.iter().enumerate() {
                for candidate_a in candidate_n_a.iter() {
                    let mut freq_exact = 1;
                    let mut freq_relaxed = 1;

                    // Check all other groups
                    for (i_b, candidates_of_graph_b) in candidates.iter().enumerate() {
                        if i_a == i_b {
                            continue;
                        }
                        // Only compare graphs of the same n size :)
                        let candidates_of_graph_b: &Vec<Candidate> =
                            candidates_of_graph_b.get(i_n_a).unwrap();

                        let mut exact_match_found = false;
                        let mut relaxed_found = false;

                        for candidate_b in candidates_of_graph_b.iter() {
                            let (a, b) = if candidate_a.graph.id < candidate_b.graph.id {
                                (candidate_a.graph.id, candidate_b.graph.id)
                            } else {
                                (candidate_b.graph.id, candidate_a.graph.id)
                            };

                            let result = *match_cache.entry((a, b)).or_insert_with(|| {
                                algo_graph_matching
                                    .match_graphs(&candidate_a.graph, &candidate_b.graph)
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
                            pattern: candidate_a.graph.clone(),
                            frequency_exact: freq_exact,
                            frequency_relaxed: freq_relaxed,
                        });
                    }
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
