use crate::data::{graph::Graph, utils::vertices_are_connected, vertex::Vertex};
use itertools::Itertools;
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Instant,
};
#[derive(Debug)]
struct GraphIdGenerator(Mutex<usize>);

impl GraphIdGenerator {
    fn new() -> Self {
        GraphIdGenerator(Mutex::new(0))
    }

    fn next_id(&self) -> usize {
        let mut data = self.0.lock().unwrap();
        *data += 1;
        *data
    }
}
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
        let graph_id_generator = Arc::new(GraphIdGenerator::new());
        let now = Instant::now();
        let candidates = match self {
            AlgoCandidateGeneration::FullyConnected {
                activity_vertex_type,
                object_vertex_types,
                min_number_of_activity_vertices,
                max_number_of_activity_vertices,
            } => get_fully_connected_candidates(
                graphs,
                activity_vertex_type,
                object_vertex_types,
                min_number_of_activity_vertices,
                max_number_of_activity_vertices,
                graph_id_generator,
            ),
        };
        let delta = now.elapsed().as_millis();
        let all_candidates: Vec<_> = candidates.iter().flatten().collect();
        println!(
            "Found candidates {}, took {}ms",
            all_candidates.len(),
            delta
        );
        candidates
    }
}

fn get_fully_connected_candidates(
    graphs: &Vec<Graph>,
    activity_vertex_type: &usize,
    object_vertex_types: &[usize],
    min_number_of_activity_vertices: &usize,
    max_number_of_activity_vertices: &usize,
    graph_id_generator: Arc<GraphIdGenerator>,
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
                Arc::clone(&graph_id_generator),
            )
        })
        .collect()
}

fn _get_fully_connected_candidates_of_graph(
    graph: &Graph,
    activity_vertex_type: &usize,
    object_vertex_types: &[usize],
    min_number_of_activity_vertices: &usize,
    max_number_of_activity_vertices: &usize,
    graph_id_generator: Arc<GraphIdGenerator>,
) -> Vec<Graph> {
    let activity_vertices = graph.get_vertices_by_type(*activity_vertex_type);
    let mut candidates = Vec::new();

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
                let mut new_candidate = Graph::new(graph_id_generator.next_id());
                let mut vertex_id_mapping: HashMap<usize, usize> = HashMap::new();

                let activity_v_ids: HashSet<usize> = comb_ref.iter().map(|v| v.id).collect();
                // Create the activity vertices
                for activity_vertex in comb_ref.iter() {
                    let new_activity_vertex = new_candidate.create_vertex_with_data(
                        activity_vertex.label,
                        activity_vertex.vertex_type,
                    );
                    vertex_id_mapping.insert(activity_vertex.id, new_activity_vertex.id);
                }
                // Create object vertices and edges
                for activity_vertex in comb_ref.iter() {
                    for edge in activity_vertex.edges.iter() {
                        let to_vertex: &Vertex = graph.vertices.get(edge.to).unwrap();
                        if object_vertex_types.contains(&to_vertex.vertex_type) {
                            let new_object_vertex_id = match vertex_id_mapping.get(&to_vertex.id) {
                                Some(id) => *id,
                                None => {
                                    let v = new_candidate.create_vertex_with_data(
                                        to_vertex.label,
                                        to_vertex.vertex_type,
                                    );
                                    v.id
                                }
                            };
                            new_candidate
                                .vertices
                                .get_mut(*vertex_id_mapping.get(&activity_vertex.id).unwrap())
                                .unwrap()
                                .push(new_object_vertex_id, edge.e_label);
                        } else if activity_v_ids.contains(&to_vertex.id) {
                            new_candidate
                                .vertices
                                .get_mut(*vertex_id_mapping.get(&activity_vertex.id).unwrap())
                                .unwrap()
                                .push(to_vertex.id, edge.e_label);
                        }
                    }
                }
                candidates.push(new_candidate);
            }
        }
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::graph::Graph;

    fn make_basic_graph() -> Graph {
        // Vertex types:
        // type 2 = activity
        // type 4 = object
        let mut g = Graph::new(1);

        // Activity vertices
        g.create_vertex_with_data(1, 2); // id 0
        g.create_vertex_with_data(2, 2); // id 1
        g.create_vertex_with_data(3, 2); // id 2

        // Object vertex
        g.create_vertex_with_data(4, 4); // id 3

        // Edges creating connectivity among activities
        g.vertices.get_mut(0).unwrap().push(1, 10);
        g.vertices.get_mut(1).unwrap().push(2, 10);

        // Objects connected all activity vertices
        g.vertices.get_mut(0).unwrap().push(3, 20);
        g.vertices.get_mut(1).unwrap().push(3, 20);
        g.vertices.get_mut(2).unwrap().push(3, 20);

        g
    }

    #[test]
    fn test_single_graph_single_candidate() {
        let g = make_basic_graph();

        let algo = AlgoCandidateGeneration::FullyConnected {
            activity_vertex_type: 2,
            object_vertex_types: vec![4],
            min_number_of_activity_vertices: 2,
            max_number_of_activity_vertices: 2,
        };

        let result = algo.get_candidates(&vec![g]);

        assert_eq!(result.len(), 1); // one input graph → one Vec<Graph>
        assert_eq!(result[0].len(), 2); // possible pairs among 3 activities = 3C2 = 3

        // All pairs are connected in the constructed graph
    }

    #[test]
    fn test_non_connected_activity_vertices_are_rejected() {
        let mut g = Graph::new(1);

        // Two activities with no edge
        g.create_vertex_with_data(1, 2); // id 0
        g.create_vertex_with_data(2, 2); // id 1

        let algo = AlgoCandidateGeneration::FullyConnected {
            activity_vertex_type: 2,
            object_vertex_types: vec![],
            min_number_of_activity_vertices: 2,
            max_number_of_activity_vertices: 2,
        };

        let result = algo.get_candidates(&vec![g]);
        assert_eq!(
            result[0].len(),
            0,
            "Disconnected activities should not produce candidates"
        );
    }

    #[test]
    fn test_object_vertices_are_included() {
        let g = make_basic_graph();

        let algo = AlgoCandidateGeneration::FullyConnected {
            activity_vertex_type: 2,
            object_vertex_types: vec![4],
            min_number_of_activity_vertices: 2,
            max_number_of_activity_vertices: 2,
        };

        let result = algo.get_candidates(&vec![g]);
        let candidates = &result[0];

        // find the candidate that includes vertex original id 1 (the one that links to object 3)
        let candidate: &Graph = candidates
            .iter()
            .find(|c| c.vertices.iter().any(|v| v.label == 2))
            .unwrap();

        // Ensure object vertex exists
        assert!(
            candidate.vertices.iter().any(|v| v.vertex_type == 4),
            "Candidate must include object vertices connected to selected activities"
        );
    }

    #[test]
    fn test_min_max_activity_vertex_limits() {
        let g = make_basic_graph();

        let algo = AlgoCandidateGeneration::FullyConnected {
            activity_vertex_type: 2,
            object_vertex_types: vec![],
            min_number_of_activity_vertices: 3,
            max_number_of_activity_vertices: 3,
        };

        let result = algo.get_candidates(&vec![g]);

        // Only one possible combination of size 3
        assert_eq!(result[0].len(), 1);
    }

    #[test]
    fn test_multiple_input_graphs() {
        let g1 = make_basic_graph();
        let g2 = make_basic_graph();

        let algo = AlgoCandidateGeneration::FullyConnected {
            activity_vertex_type: 2,
            object_vertex_types: vec![4],
            min_number_of_activity_vertices: 2,
            max_number_of_activity_vertices: 2,
        };

        let result = algo.get_candidates(&vec![g1, g2]);

        // one vec per graph
        assert_eq!(result.len(), 2);

        // each graph should yield 3 candidates (3C2)
        assert_eq!(result[0].len(), 2);
        assert_eq!(result[1].len(), 2);
    }

    #[test]
    fn test_graph_id_generation_increments() {
        let g = make_basic_graph();

        let algo = AlgoCandidateGeneration::FullyConnected {
            activity_vertex_type: 2,
            object_vertex_types: vec![4],
            min_number_of_activity_vertices: 2,
            max_number_of_activity_vertices: 2,
        };

        let result = algo.get_candidates(&vec![g]);

        let ids: Vec<_> = result[0].iter().map(|c| c.id).collect();
        let mut sorted = ids.clone();
        sorted.sort();

        assert_eq!(ids, sorted, "Graph IDs should increase monotonically");
        assert!(
            ids.windows(2).all(|w| w[1] > w[0]),
            "Each ID must be unique and increasing"
        );
    }
}
