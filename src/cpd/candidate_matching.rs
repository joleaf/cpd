use crate::data::graph::Graph;

use super::graph_matching::AlgoGraphMatching;

#[derive(Debug)]
pub enum AlgoCandidateMatching {
    Naive {
        algo_graph_matching: AlgoGraphMatching,
    },
}

impl AlgoCandidateMatching {
    pub fn run_matching(&self, candidates: &[Vec<Graph>]) -> Vec<Graph> {
        let _t = candidates;
        todo!()
    }
}
