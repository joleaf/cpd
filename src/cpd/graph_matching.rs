use crate::data::graph::Graph;
use petgraph::algo::isomorphism::is_isomorphic_matching;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

/// Defines the available algorithms for comparing two graphs and determining
/// how similar they are.
///
/// Currently, the only implemented method is:
///
/// - `CosineSimilarity { alpha, matching_threshold }`  
///   Computes similarity based on the cosine similarity of vertex- and edge-frequency
///   vectors extracted from both graphs.  
///
/// # Parameters
///
/// * `alpha` – Weight (0–1) determining the contribution of vertex similarity
///   vs. edge similarity.  
///   - `1.0` → only vertex similarity  
///   - `0.0` → only edge similarity  
/// * `matching_threshold` – Minimum similarity required to classify two graphs
///   as a `RelaxedMatch`.
///
/// # Matching Semantics
///
/// After computing the similarity `sim ∈ [0, 1]`:
///
/// - `sim == 1.0` → `ExactMatch` or `RelaxedMatch` (Important: This does not mean that the graphs are IDENDICAL!)
///   The ExactMatch is only returned if the VF2IsomorphismTest also returns `ExactMatch`
/// - `sim >= matching_threshold` → `RelaxedMatch`  
/// - otherwise → `NoMatch`
///
/// # Example
///
/// ```rust
/// use crate::graph_matching::{AlgoGraphMatching, MatchingResult};
/// use crate::data::graph::Graph;
///
/// let mut g1 = Graph::new(1);
/// g1.create_vertex_with_data(1, 2);
///
/// let mut g2 = Graph::new(2);
/// g2.create_vertex_with_data(1, 2);
///
/// let algo = AlgoGraphMatching::CosineSimilarity {
///     alpha: 0.5,
///     matching_threshold: 0.8,
/// };
///
/// let result = algo.match_graphs(&g1, &g2);
///
/// assert!(matches!(result, MatchingResult::ExactMatch | MatchingResult::RelaxedMatch | MatchingResult::NoMatch));
/// ```
#[derive(Debug)]
pub enum AlgoGraphMatching {
    CosineSimilarity { alpha: f32, matching_threshold: f32 },
    VF2IsomorphismTest,
}

/// Result of comparing two graphs with the selected graph-matching algorithm.
///
/// # Variants
/// - `ExactMatch` – Graphs are identical under the similarity metric (similarity = 1.0).  
/// - `RelaxedMatch` – Graphs are sufficiently similar but not identical.  
/// - `NoMatch` – Graphs do not meet the relaxed similarity threshold.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MatchingResult {
    ExactMatch,
    RelaxedMatch,
    NoMatch,
}

impl AlgoGraphMatching {
    /// Computes a similarity score between two graphs.
    ///
    /// # Returns
    ///
    /// A floating-point similarity value in the range `[0.0, 1.0]`, where:
    ///
    /// - `1.0` represents very similar graphs (based on vertex & edge vectors) (but not necessary
    ///   identical graphs)
    /// - `0.0` represents complete dissimilarity
    ///
    /// The meaning of the distance depends on the specific matching algorithm:
    ///
    /// For `CosineSimilarity`, the final score is:
    ///
    /// ```
    /// score = alpha * vertex_similarity + (1 - alpha) * edge_similarity
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// let algo = AlgoGraphMatching::CosineSimilarity {
    ///     alpha: 0.4,
    ///     matching_threshold: 0.7,
    /// };
    ///
    /// let distance = algo.calc_distance(&g1, &g2);
    ///
    /// println!("Similarity = {}", distance);
    /// ```
    pub fn calc_distance(&self, one_graph: &Graph, other_graph: &Graph) -> f32 {
        match self {
            AlgoGraphMatching::CosineSimilarity {
                alpha,
                matching_threshold: _,
            } => graph_cosine_similarity(one_graph, other_graph, alpha),
            AlgoGraphMatching::VF2IsomorphismTest => graph_vf2_isomorphism(one_graph, other_graph),
        }
    }

    /// Determines the match type between two graphs: exact, relaxed, or no match.
    ///
    /// # Rules
    ///
    /// Let `sim = calc_distance(one_graph, other_graph)`:
    ///
    /// - If `sim == 1.0` → `MatchingResult::ExactMatch` iff VF2IsomorphismTest also returns 1.0
    ///   Else `MatchingResult::RelaxedMatch`:w
    ///
    /// - Else if `sim >= matching_threshold` → `MatchingResult::RelaxedMatch`
    /// - Else → `MatchingResult::NoMatch`
    ///
    /// For VF2IsomorphismTest, the result is either `ExactMatch` or `NoMatch`.
    ///
    /// # Parameters
    ///
    /// * `one_graph` – First graph in the comparison  
    /// * `other_graph` – Second graph  
    ///
    /// # Returns
    ///
    /// A `MatchingResult` enum describing how similar the graphs are.
    ///
    /// # Example
    ///
    ///
    /// ```rust
    /// let algo = AlgoGraphMatching::CosineSimilarity {
    ///     alpha: 0.5,
    ///     matching_threshold: 0.6,
    /// };
    ///
    /// let r = algo.match_graphs(&graph_a, &graph_b);
    ///
    /// match r {
    ///     MatchingResult::ExactMatch => println!("Identical patterns"),
    ///     MatchingResult::RelaxedMatch => println!("Similar patterns"),
    ///     MatchingResult::NoMatch => println!("Not similar"),
    /// }
    /// ```
    pub fn match_graphs(&self, one_graph: &Graph, other_graph: &Graph) -> MatchingResult {
        let distance = self.calc_distance(one_graph, other_graph);
        println!("{distance}");
        match self {
            AlgoGraphMatching::CosineSimilarity {
                alpha: _,
                matching_threshold,
            } => {
                if distance == 1.0 {
                    if AlgoGraphMatching::VF2IsomorphismTest.match_graphs(one_graph, other_graph)
                        == MatchingResult::ExactMatch
                    {
                        MatchingResult::ExactMatch
                    } else {
                        MatchingResult::RelaxedMatch
                    }
                } else if distance >= *matching_threshold {
                    MatchingResult::RelaxedMatch
                } else {
                    MatchingResult::NoMatch
                }
            }
            AlgoGraphMatching::VF2IsomorphismTest => {
                if distance == 1.0 {
                    MatchingResult::ExactMatch
                } else {
                    MatchingResult::NoMatch
                }
            }
        }
    }
}

fn graph_cosine_similarity(one_graph: &Graph, other_graph: &Graph, alpha: &f32) -> f32 {
    let one_graph_vertex_vector = one_graph.get_vertex_vector();
    let other_graph_vertex_vector = other_graph.get_vertex_vector();
    let one_graph_edge_vector = one_graph.get_edge_vector();
    let other_graph_edge_vector = other_graph.get_edge_vector();
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

fn graph_vf2_isomorphism(one_graph: &Graph, other_graph: &Graph) -> f32 {
    let one_di_graph = one_graph.get_digraph();
    let other_di_graph = other_graph.get_digraph();
    let node_match = |a: &(usize, usize), b: &(usize, usize)| -> bool { a.0 == b.0 && a.1 == b.1 };

    // Edge matcher compares edge labels (usize)
    let edge_match = |a: &usize, b: &usize| -> bool { a == b };

    // VF2 returns an iterator; if any mapping exists → isomorphic
    let iso_exists =
        is_isomorphic_matching(&*one_di_graph, &*other_di_graph, node_match, edge_match);

    if iso_exists { 1.0 } else { 0.0 }
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
        let mut other_eq_graph = Graph::new(1);
        other_eq_graph.create_vertex_with_data(1, 2);
        other_eq_graph.create_vertex_with_data(2, 2);
        other_eq_graph.create_vertex_with_data(3, 4);
        other_eq_graph.create_vertex_with_data(4, 2);
        other_eq_graph.vertices.get_mut(0).unwrap().push(1, 0);
        other_eq_graph.vertices.get_mut(0).unwrap().push(2, 0);
        other_eq_graph.vertices.get_mut(1).unwrap().push(2, 0);
        other_eq_graph.vertices.get_mut(1).unwrap().push(3, 0);
        other_eq_graph.vertices.get_mut(3).unwrap().push(2, 0);
        assert_eq!(
            AlgoGraphMatching::CosineSimilarity {
                alpha: 0.5,
                matching_threshold: 1.0
            }
            .match_graphs(&one_graph, &other_eq_graph,),
            MatchingResult::ExactMatch
        );
        assert_eq!(
            AlgoGraphMatching::CosineSimilarity {
                alpha: 0.5,
                matching_threshold: 0.5
            }
            .match_graphs(&one_graph, &other_eq_graph,),
            MatchingResult::ExactMatch
        );

        let mut other_graph = Graph::new(1);
        other_graph.create_vertex_with_data(1, 2);
        other_graph.create_vertex_with_data(2, 2);
        other_graph.create_vertex_with_data(5, 4);
        other_graph.create_vertex_with_data(4, 2);
        other_graph.vertices.get_mut(0).unwrap().push(1, 0);
        other_graph.vertices.get_mut(0).unwrap().push(2, 0);
        other_graph.vertices.get_mut(1).unwrap().push(2, 0);
        other_graph.vertices.get_mut(1).unwrap().push(3, 0);
        other_graph.vertices.get_mut(3).unwrap().push(2, 0);

        assert_eq!(
            AlgoGraphMatching::CosineSimilarity {
                alpha: 0.5,
                matching_threshold: 0.6
            }
            .match_graphs(&one_graph, &other_graph),
            MatchingResult::NoMatch
        );
        assert_eq!(
            AlgoGraphMatching::CosineSimilarity {
                alpha: 0.5,
                matching_threshold: 0.5
            }
            .match_graphs(&one_graph, &other_graph),
            MatchingResult::RelaxedMatch
        );
    }

    #[test]
    fn test_vf2() {
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
        let mut other_eq_graph = Graph::new(1);
        other_eq_graph.create_vertex_with_data(1, 2);
        other_eq_graph.create_vertex_with_data(2, 2);
        other_eq_graph.create_vertex_with_data(3, 4);
        other_eq_graph.create_vertex_with_data(4, 2);
        other_eq_graph.vertices.get_mut(0).unwrap().push(1, 0);
        other_eq_graph.vertices.get_mut(0).unwrap().push(2, 0);
        other_eq_graph.vertices.get_mut(1).unwrap().push(2, 0);
        other_eq_graph.vertices.get_mut(1).unwrap().push(3, 0);
        other_eq_graph.vertices.get_mut(3).unwrap().push(2, 0);
        assert_eq!(
            AlgoGraphMatching::VF2IsomorphismTest.match_graphs(&one_graph, &other_eq_graph,),
            MatchingResult::ExactMatch
        );

        let mut other_graph = Graph::new(1);
        other_graph.create_vertex_with_data(2, 2);
        other_graph.create_vertex_with_data(3, 2);
        other_graph.create_vertex_with_data(1, 4);
        other_graph.create_vertex_with_data(2, 2);
        other_graph.vertices.get_mut(0).unwrap().push(3, 0);
        other_graph.vertices.get_mut(0).unwrap().push(2, 0);
        other_graph.vertices.get_mut(1).unwrap().push(2, 0);
        other_graph.vertices.get_mut(1).unwrap().push(3, 0);
        other_graph.vertices.get_mut(3).unwrap().push(2, 0);
        assert_eq!(
            AlgoGraphMatching::VF2IsomorphismTest.match_graphs(&one_graph, &other_graph,),
            MatchingResult::NoMatch
        );
    }
}
