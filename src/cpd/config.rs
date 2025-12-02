use std::time::Instant;

use crate::data::graph::Graph;

use super::{
    candidate_generation::AlgoCandidateGeneration,
    candidate_matching::{AlgoCandidateMatching, PatternResult},
    graph_matching::AlgoGraphMatching,
};

/// Runs the full Collaboration Pattern Discovery (CPD) pipeline:
///
/// 1. **Candidate generation** using `AlgoCandidateGeneration`
/// 2. **Candidate matching** across all graphs using `AlgoCandidateMatching`
/// 3. **Pattern extraction** based on exact and relaxed support thresholds
///
/// The function prints progress information unless `silence = true`.
///
/// # Arguments
///
/// * `graphs` — A list of input graphs from which candidates will be generated.
///
/// # Returns
///
/// A `Vec<PatternResult>` containing all pattern graphs that satisfy the
/// specified support thresholds.
///
/// # Example
///
/// The following example runs the full CPD pipeline on two graphs that share
/// a simple fully-connected activity structure:
///
/// ```rust
/// use crate::cpd::CPDConfig;
/// use crate::candidate_generation::AlgoCandidateGeneration;
/// use crate::graph_matching::AlgoGraphMatching;
/// use crate::data::graph::Graph;
///
/// // Build two small graphs with identical 2-activity fully connected structure
/// fn make_graph(id: usize) -> Graph {
///     let mut g = Graph::new(id);
///     g.create_vertex_with_data(1, 2);
///     g.create_vertex_with_data(2, 2);
///     g.vertices[0].push(1, 0);
///     g.vertices[1].push(0, 0);
///     g
/// }
///
/// let g1 = make_graph(1);
/// let g2 = make_graph(2);
/// let graphs = vec![g1, g2];
///
/// // Generate all fully connected subsets of size 2
/// let candidate_gen = AlgoCandidateGeneration::FullyConnected {
///     activity_vertex_type: 2,
///     object_vertex_types: vec![],
///     min_number_of_activity_vertices: 2,
///     max_number_of_activity_vertices: 2,
/// };
///
/// // Match candidates using cosine-similarity graph matching
/// let graph_match = AlgoGraphMatching::CosineSimilarity {
///     alpha: 0.5,
///     matching_threshold: 0.8,
/// };
///
/// // Build CPD config (Parallel matching is selected by default)
/// let cpd = CPDConfig::new(
///     candidate_gen,
///     graph_match,
///     1,   // support_exact
///     1,   // support_relaxed
///     false,
/// );
///
/// let patterns = cpd.run(&graphs);
///
/// assert!(!patterns.is_empty());
/// println!(
///     "Found {} patterns; first pattern has frequency_exact = {}",
///     patterns.len(),
///     patterns[0].frequency_exact
/// );
/// ```
///
/// # Notes
///
/// - Candidate generation runs in **parallel**
/// - Candidate matching runs in **parallel**
/// - Pattern IDs in the result are always rewritten to ensure they form a
///   contiguous sequence starting at zero.
#[derive(Debug)]
pub struct CPDConfig {
    algo_candidate_generation: AlgoCandidateGeneration,
    algo_candidate_matching: AlgoCandidateMatching,
    algo_graph_matching: AlgoGraphMatching,
    support_exact: usize,
    support_relaxed: usize,
    silence: bool,
}

impl CPDConfig {
    pub fn new(
        algo_candidate_generation: AlgoCandidateGeneration,
        algo_graph_matching: AlgoGraphMatching,
        support_exact: usize,
        support_relaxed: usize,
        silence: bool,
    ) -> Self {
        Self {
            algo_candidate_generation,
            algo_candidate_matching: AlgoCandidateMatching::Parallel,
            algo_graph_matching,
            support_exact,
            support_relaxed,
            silence,
        }
    }

    pub fn run(&self, graphs: &Vec<Graph>) -> Vec<PatternResult> {
        if !self.silence {
            println!(
                "1. Candidate generation : {:?}",
                self.algo_candidate_generation
            );
        }
        let now = Instant::now();
        let candidates = self.algo_candidate_generation.get_candidates(graphs);
        let delta = now.elapsed().as_millis();
        if !self.silence {
            let all_candidates: Vec<_> = candidates.iter().flatten().collect();
            println!(
                " -> Found {} candidates; took {}ms",
                all_candidates.len(),
                delta
            );
            println!(
                "2. Candidate matching   : {:?}",
                self.algo_candidate_matching
            );
            println!(" - Exact support        : {:?}", self.support_exact);
            println!(" - Releaxed support     : {:?}", self.support_relaxed);
            println!(" - Graph matching       : {:?}", self.algo_graph_matching);
        }

        let now = Instant::now();
        let result = self.algo_candidate_matching.run_matching(
            &candidates,
            &self.algo_graph_matching,
            self.support_exact,
            self.support_relaxed,
        );
        let delta = now.elapsed().as_millis();
        if !self.silence {
            println!(" -> Found {} patterns; took {delta}ms", result.len());
        }
        result
    }
}
