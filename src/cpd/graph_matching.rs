use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::data::{
    graph::Graph,
    utils::{get_edge_vector, get_vertex_vector},
};

#[derive(Debug)]
pub enum AlgoGraphMatching {
    CosineSimilarity { alpha: f32 },
}
#[derive(Debug, PartialEq, Eq)]
pub enum MatchingResult {
    ExactMatch,
    RelaxedMatch,
    NoMatch,
}

impl AlgoGraphMatching {
    pub fn calc_distance(&self, one_graph: &Graph, other_graph: &Graph) -> f32 {
        match self {
            AlgoGraphMatching::CosineSimilarity { alpha: alpha_value } => {
                graph_cosine_similarity(one_graph, other_graph, *alpha_value)
            }
        }
    }

    pub fn match_graphs(
        &self,
        one_graph: &Graph,
        other_graph: &Graph,
        matching_threshold: &f32,
    ) -> MatchingResult {
        let distance = self.calc_distance(one_graph, other_graph);
        match self {
            AlgoGraphMatching::CosineSimilarity { alpha: _ } => {
                if distance == 1.0f32 {
                    return MatchingResult::ExactMatch;
                } else if &distance >= matching_threshold {
                    return MatchingResult::RelaxedMatch;
                } else {
                    return MatchingResult::NoMatch;
                }
            }
        }
    }
}

fn graph_cosine_similarity(one_graph: &Graph, other_graph: &Graph, alpha: f32) -> f32 {
    let one_graph_vertex_vector = get_vertex_vector(&one_graph);
    let other_graph_vertex_vector = get_vertex_vector(&other_graph);
    let one_graph_edge_vector = get_edge_vector(&one_graph);
    let other_graph_edge_vector = get_edge_vector(&other_graph);
    let sim_vertices =
        _calc_cosine_similarity(&one_graph_vertex_vector, &other_graph_vertex_vector);
    let sim_edges = _calc_cosine_similarity(&one_graph_edge_vector, &other_graph_edge_vector);
    alpha * sim_vertices + (1.0f32 - alpha) * sim_edges
}

fn _calc_cosine_similarity<T: Eq + Hash>(
    one_vec: &HashMap<T, usize>,
    other_vec: &HashMap<T, usize>,
) -> f32 {
    let mut keys = HashSet::new();
    keys.extend(one_vec.keys());
    keys.extend(other_vec.keys());
    let mut dot = 0.0f32;
    let mut norm_one = 0.0f32;
    let mut norm_other = 0.0f32;

    for key in keys {
        let v1 = *one_vec.get(key).unwrap_or(&0) as f32;
        let v2 = *other_vec.get(key).unwrap_or(&0) as f32;

        dot += v1 * v2;
        norm_one += v1 * v1;
        norm_other += v2 * v2;
    }

    if norm_one == 0.0 || norm_other == 0.0 {
        return 0.0;
    }

    dot / (norm_one.sqrt() * norm_other.sqrt())
}

#[cfg(test)]
mod tests {
    use crate::data::graph::Graph;

    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let mut one_graph = Graph::new(1);
        one_graph.create_vertex_with_data(1, 2);
        one_graph.create_vertex_with_data(2, 2);
        one_graph.create_vertex_with_data(3, 4);
        one_graph.create_vertex_with_data(4, 2);
        one_graph.vertices.get_mut(0).unwrap().push(1, 0);
        one_graph.vertices.get_mut(0).unwrap().push(2, 0);
        one_graph.vertices.get_mut(1).unwrap().push(2, 0);
        one_graph.vertices.get_mut(1).unwrap().push(3, 0);
        one_graph.vertices.get_mut(3).unwrap().push(2, 0);
        let mut other_graph = Graph::new(1);
        other_graph.create_vertex_with_data(1, 2);
        other_graph.create_vertex_with_data(2, 2);
        other_graph.create_vertex_with_data(3, 4);
        other_graph.create_vertex_with_data(4, 2);
        other_graph.vertices.get_mut(0).unwrap().push(1, 0);
        other_graph.vertices.get_mut(0).unwrap().push(2, 0);
        other_graph.vertices.get_mut(1).unwrap().push(2, 0);
        other_graph.vertices.get_mut(1).unwrap().push(3, 0);
        other_graph.vertices.get_mut(3).unwrap().push(2, 0);
        assert_eq!(
            AlgoGraphMatching::CosineSimilarity { alpha: 0.5 }.match_graphs(
                &one_graph,
                &other_graph,
                &1.0f32
            ),
            MatchingResult::ExactMatch
        )
    }
}
