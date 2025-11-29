use std::usize;

use crate::data::graph::Graph;

#[derive(Debug)]
pub enum AlgoCandidateGeneration {
    FullyConnected {
        activity_node_type: usize,
        object_node_types: Vec<usize>,
    },
}

pub fn get_candidates(graphs: &Vec<Graph>, algorithm: AlgoCandidateGeneration) -> Vec<Vec<Graph>> {
    match algorithm {
        AlgoCandidateGeneration::FullyConnected {
            activity_node_type,
            object_node_types,
        } => get_candidates_fully_connected(graphs, &activity_node_type, &object_node_types),
    }
}

pub fn get_candidates_fully_connected(
    graphs: &Vec<Graph>,
    activity_node_type: &usize,
    object_node_types: &Vec<usize>,
) -> Vec<Vec<Graph>> {
    println!("Searching candidates in {} graphs, with {} as activity node type and {:?} as object nodes types", graphs.len(), activity_node_type, object_node_types);

    todo!()
}
