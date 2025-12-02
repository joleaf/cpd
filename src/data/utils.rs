use std::collections::{HashMap, HashSet};

use crate::data::edge::Edge;

use super::{graph::Graph, vertex::Vertex};

pub fn vertices_are_connected(vertices: &Vec<&Vertex>) -> bool {
    let v_ids: HashSet<usize> = vertices.iter().map(|v| v.id).collect();
    let mut edges: Vec<&Edge> = vertices
        .iter()
        .flat_map(|v| &v.edges)
        .filter(|e| v_ids.contains(&e.from) && v_ids.contains(&e.to))
        .collect();
    let mut visited_v = HashSet::with_capacity(v_ids.len());
    visited_v.insert(vertices.first().unwrap().id);
    let mut to_remove = HashSet::new();
    while v_ids.len() != visited_v.len() {
        for edge in edges.iter() {
            if visited_v.contains(&edge.from) || visited_v.contains(&edge.to) {
                visited_v.insert(edge.from);
                visited_v.insert(edge.to);
                to_remove.insert(edge.id);
            }
        }
        if to_remove.is_empty() {
            break;
        } else {
            edges.retain(|e| !to_remove.contains(&e.id));
            to_remove.clear();
        }
    }
    v_ids.len() == visited_v.len()
}

// Maybe implement caching for the two functions: https://stackoverflow.com/questions/36230889/what-is-the-idiomatic-way-to-implement-caching-on-a-function-that-is-not-a-struc

pub fn get_vertex_vector(graph: &Graph) -> HashMap<usize, usize> {
    let mut result = HashMap::new();
    graph.vertices.iter().for_each(|v| {
        *result.entry(v.label).or_insert(0) += 1;
    });
    result
}

pub fn get_edge_vector(graph: &Graph) -> HashMap<(usize, usize, usize), usize> {
    let mut result = HashMap::new();
    graph.vertices.iter().for_each(|v| {
        v.edges.iter().for_each(|e| {
            let to_label = graph.vertices.get(e.to).unwrap().label;
            *result.entry((v.label, to_label, e.e_label)).or_insert(0) += 1;
        });
    });
    result
}

#[cfg(test)]
mod tests {
    use crate::data::graph::Graph;

    use super::*;

    #[test]
    fn test_vertices_are_connected() {
        let mut graph = Graph::new(1);
        graph.create_vertex_with_data(1, 2);
        graph.create_vertex_with_data(2, 2);
        graph.create_vertex_with_data(3, 4);
        graph.create_vertex_with_data(4, 2);
        graph.vertices.get_mut(0).unwrap().push(1, 0);
        graph.vertices.get_mut(0).unwrap().push(2, 0);
        graph.vertices.get_mut(1).unwrap().push(2, 0);
        graph.vertices.get_mut(1).unwrap().push(3, 0);
        graph.vertices.get_mut(3).unwrap().push(2, 0);
        let result = vertices_are_connected(&graph.vertices.iter().collect());
        assert!(result);
        let result = vertices_are_connected(&vec![
            graph.vertices.first().unwrap(),
            graph.vertices.get(1).unwrap(),
            graph.vertices.get(3).unwrap(),
        ]);
        assert!(result);
        let result = vertices_are_connected(&vec![
            graph.vertices.first().unwrap(),
            graph.vertices.get(3).unwrap(),
        ]);
        assert!(!result);
    }

    #[test]
    fn test_vertex_vector() {
        let mut graph = Graph::new(1);
        graph.create_vertex_with_data(1, 2);
        graph.create_vertex_with_data(2, 2);
        graph.create_vertex_with_data(3, 4);
        graph.create_vertex_with_data(2, 2);
        graph.vertices.get_mut(0).unwrap().push(1, 0);
        graph.vertices.get_mut(0).unwrap().push(2, 0);
        graph.vertices.get_mut(1).unwrap().push(2, 0);
        graph.vertices.get_mut(1).unwrap().push(3, 0);
        graph.vertices.get_mut(3).unwrap().push(2, 0);
        let vertex_vector = get_vertex_vector(&graph);
        assert_eq!(vertex_vector.len(), 3);
        assert_eq!(vertex_vector[&1], 1);
        assert_eq!(vertex_vector[&2], 2);
        assert_eq!(vertex_vector[&3], 1);
    }

    #[test]
    fn test_edge_vector() {
        let mut graph = Graph::new(1);
        graph.create_vertex_with_data(1, 2);
        graph.create_vertex_with_data(2, 2);
        graph.create_vertex_with_data(3, 4);
        graph.create_vertex_with_data(2, 2);
        graph.vertices.get_mut(0).unwrap().push(1, 0);
        graph.vertices.get_mut(0).unwrap().push(2, 0);
        graph.vertices.get_mut(1).unwrap().push(2, 0);
        graph.vertices.get_mut(1).unwrap().push(3, 0);
        graph.vertices.get_mut(3).unwrap().push(2, 0);
        let edge_vector = get_edge_vector(&graph);
        assert_eq!(edge_vector.len(), 4);
        assert_eq!(edge_vector[&(1, 2, 0)], 1);
        assert_eq!(edge_vector[&(1, 3, 0)], 1);
        assert_eq!(edge_vector[&(2, 2, 0)], 1);
        assert_eq!(edge_vector[&(2, 3, 0)], 2);
    }
}
