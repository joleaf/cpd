use crate::data::graph::Graph;

use super::{
    candidate_generation::AlgoCandidateGeneration,
    candidate_matching::{AlgoCandidateMatching, PatternResult},
    graph_matching::AlgoGraphMatching,
};

#[derive(Debug)]
pub struct CPDConfig {
    algo_candidate_generation: AlgoCandidateGeneration,
    algo_candidate_matching: AlgoCandidateMatching,
    algo_graph_matching: AlgoGraphMatching,
    support_exact: usize,
    support_relaxed: usize,
}

impl CPDConfig {
    pub fn new(
        algo_candidate_generation: AlgoCandidateGeneration,
        algo_graph_matching: AlgoGraphMatching,
        support_exact: usize,
        support_relaxed: usize,
    ) -> Self {
        Self {
            algo_candidate_generation,
            algo_candidate_matching: AlgoCandidateMatching::Parallel,
            algo_graph_matching,
            support_exact,
            support_relaxed,
        }
    }

    pub fn run(&self, graphs: &Vec<Graph>) -> Vec<PatternResult> {
        println!("Running subgraph mining");
        println!(
            "Candidate generation : {:?}",
            self.algo_candidate_generation
        );
        let candidates = self.algo_candidate_generation.get_candidates(graphs);
        println!("Candidate matching   : {:?}", self.algo_candidate_matching);
        println!(" - Exact support     : {:?}", self.support_exact);
        println!(" - Releaxed support  : {:?}", self.support_relaxed);
        println!(" - Graph matching    : {:?}", self.algo_graph_matching);

        self.algo_candidate_matching.run_matching(
            &candidates,
            &self.algo_graph_matching,
            self.support_exact,
            self.support_relaxed,
        )
    }
}
