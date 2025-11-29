use crate::data::graph::Graph;

use super::{candidates::AlgoCandidateGeneration, pattern_matching::AlgoPatternMatching};

#[derive(Debug)]
pub struct CPDConfig {
    algo_candidate_generation: AlgoCandidateGeneration,
    algo_pattern_matching: AlgoPatternMatching,
}

impl CPDConfig {
    pub fn new(
        algo_candidate_generation: AlgoCandidateGeneration,
        algo_pattern_matching: AlgoPatternMatching,
    ) -> Self {
        Self {
            algo_candidate_generation,
            algo_pattern_matching,
        }
    }

    pub fn run(&self, graphs: &Vec<Graph>) {
        println!("Running subgraph mining");
        println!("{:?}", self.algo_candidate_generation);
        println!("{:?}", self.algo_pattern_matching);
        let candidates = self.algo_candidate_generation.get_candidates(graphs);
        println!("Found candidates {}", candidates.len());
    }
}
