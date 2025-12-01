use crate::data::graph::Graph;

use super::{
    candidate_generation::AlgoCandidateGeneration, candidate_matching::AlgoCandidateMatching,
};

#[derive(Debug)]
pub struct CPDConfig {
    algo_candidate_generation: AlgoCandidateGeneration,
    algo_candidate_matching: AlgoCandidateMatching,
}

impl CPDConfig {
    pub fn new(
        algo_candidate_generation: AlgoCandidateGeneration,
        algo_candidate_matching: AlgoCandidateMatching,
    ) -> Self {
        Self {
            algo_candidate_generation,
            algo_candidate_matching,
        }
    }

    pub fn run(&self, graphs: &Vec<Graph>) -> Vec<Graph> {
        println!("Running subgraph mining");
        println!(
            "Candidate generation : {:?}",
            self.algo_candidate_generation
        );
        println!("Candidate matching   : {:?}", self.algo_candidate_matching);
        let candidates = self.algo_candidate_generation.get_candidates(graphs);
        println!("Candidate matching   : {:?}", self.algo_candidate_matching);
        self.algo_candidate_matching.run_matching(&candidates)
    }
}
