use crate::data::graph::{self, Graph};

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
        println!("Hello {}", graphs.len());
    }
}
