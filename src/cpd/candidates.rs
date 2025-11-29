use crate::{
    cpd::candidates,
    data::{graph::Graph, utils::vertices_are_connected, vertex::Vertex},
};
use itertools::Itertools;
use rayon::prelude::*;
use std::{collections::HashSet, usize};

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
        //.par_iter() // Parallel processing
        .iter()
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
    let candidates = Vec::new();

    // Get candidates for the requested number of activity vertices
    for number_of_activity_vertices in
        *min_number_of_activity_vertices..(max_number_of_activity_vertices + 1)
    {
        for comb in activity_vertices
            .iter()
            .combinations(number_of_activity_vertices)
        {
            let comb_ref: Vec<&Vertex> = comb.into_iter().copied().collect();
            // Check if the vertices are connected
            if vertices_are_connected(&comb_ref) {
                let activity_v_ids: HashSet<usize> = comb_ref.iter().map(|v| v.id).collect();
                let mut edges = Vec::new();
                let mut object_vertices = Vec::new();
                // Collect all object vertices and edges
                for activity_vertex in comb_ref.iter() {
                    let activity_vertex = *activity_vertex;
                    for edge in activity_vertex.edges.iter() {
                        let to_vertex: &Vertex = graph.vertices.get(edge.to).unwrap();
                        // Add object vertex
                        if object_vertex_types.contains(&to_vertex.vertex_type) {
                            object_vertices.push(to_vertex);
                            edges.push(edge);
                        }
                        // Add edges
                        else if activity_v_ids.contains(&to_vertex.id) {
                            edges.push(edge);
                        }
                    }
                }
                println!("");
                println!("{:?}", edges);
                println!("{:?}", object_vertices);
                // Create the new candidate graph
            }
        }
    }
    candidates
}
