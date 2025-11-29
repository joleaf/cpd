use crate::{cpd::candidates, data::graph::Graph};
use rayon::prelude::*;
use std::usize;

#[derive(Debug)]
pub enum AlgoCandidateGeneration {
    FullyConnected {
        activity_vertex_type: usize,
        object_vertex_types: Vec<usize>,
        min_number_of_activity_vertices: usize,
        max_number_of_activity_vertices: usize,
    },
}

impl AlgoCandidateGeneration {
    pub fn get_candidates(&self, graphs: &Vec<Graph>) -> Vec<Vec<Graph>> {
        match self {
            AlgoCandidateGeneration::FullyConnected {
                activity_vertex_type,
                object_vertex_types,
                min_number_of_activity_vertices,
                max_number_of_activity_vertices,
            } => get_fully_connected_candidates(
                graphs,
                &activity_vertex_type,
                &object_vertex_types,
                &min_number_of_activity_vertices,
                &max_number_of_activity_vertices,
            ),
        }
    }
}

fn get_fully_connected_candidates(
    graphs: &Vec<Graph>,
    activity_vertex_type: &usize,
    object_vertex_types: &Vec<usize>,
    min_number_of_activity_vertices: &usize,
    max_number_of_activity_vertices: &usize,
) -> Vec<Vec<Graph>> {
    println!("Searching candidates in {} graphs, with {} as activity vertex type and {:?} as object vertex types, from {} activtiy vertices to {} activity vertices", graphs.len(), activity_vertex_type, object_vertex_types, min_number_of_activity_vertices, max_number_of_activity_vertices);
    graphs
        .par_iter() // Parallel processing
        //.iter()
        .map(|g| {
            _get_fully_connected_candidates_of_graph(
                g,
                activity_vertex_type,
                object_vertex_types,
                min_number_of_activity_vertices,
                max_number_of_activity_vertices,
            )
        })
        .collect()
}

fn _get_fully_connected_candidates_of_graph(
    graph: &Graph,
    activity_vertex_type: &usize,
    object_vertex_types: &Vec<usize>,
    min_number_of_activity_vertices: &usize,
    max_number_of_activity_vertices: &usize,
) -> Vec<Graph> {
    println!("Searching in graph {:?}", graph);
    let activity_vertices = graph.get_vertices_by_type(*activity_vertex_type);
    let mut candidates = Vec::new();

    for number_of_activity_vertices in
        *min_number_of_activity_vertices..(max_number_of_activity_vertices + 1)
    {
        println!("{}", number_of_activity_vertices);
        // combis = combinations(sorted(activity_vertices), node_count)
        // First get all possible combinations for node_count of only the activity nodes!
    }
    todo!();
    candidates
}
