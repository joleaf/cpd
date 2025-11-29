use std::collections::HashSet;

use crate::data::edge::Edge;

use super::vertex::Vertex;

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
        if to_remove.len() == 0 {
            break;
        } else {
            edges.retain(|e| !to_remove.contains(&e.id));
            to_remove.clear();
        }
    }
    v_ids.len() == visited_v.len()
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
        assert_eq!(result, true);
        let result = vertices_are_connected(&vec![
            graph.vertices.get(0).unwrap(),
            graph.vertices.get(1).unwrap(),
            graph.vertices.get(3).unwrap(),
        ]);
        assert_eq!(result, true);
        let result = vertices_are_connected(&vec![
            graph.vertices.get(0).unwrap(),
            graph.vertices.get(3).unwrap(),
        ]);
        assert_eq!(result, false);
    }
}
