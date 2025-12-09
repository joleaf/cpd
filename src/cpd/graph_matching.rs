use crate::data::graph::Graph;
use petgraph::visit::EdgeRef;
use petgraph::visit::NodeIndexable;
use petgraph::{algo::isomorphism::is_isomorphic_matching, graph::DiGraph};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

/// Defines the available algorithms for comparing two graphs and determining
/// how similar they are.
///
/// Currently, the implemented methods are:
/// - `CosineSimilarity { alpha, matching_threshold }`
/// - `VF2IsomorphismTest`
/// - `GEDFastHungarian { edit_costs, matching_threshold }`
#[derive(Debug)]
pub enum AlgoGraphMatching {
    /// Computes similarity based on the cosine similarity of vertex- and edge-frequency
    /// vectors extracted from both graphs.
    ///
    /// # Parameters
    /// - `alpha`: weight of vertex similarity vs. edge similarity (0–1)
    /// - `matching_threshold`: minimum similarity for `RelaxedMatch`
    CosineSimilarity { alpha: f64, matching_threshold: f64 },

    /// Determines exact graph isomorphism using the VF2 algorithm.
    /// Returns either `ExactMatch` or `NoMatch`.
    VF2IsomorphismTest,

    /// Approximate graph edit distance using bipartite matching.
    /// Considers node/edge insertions, deletions, and substitutions.
    GEDFastHungarian {
        edit_costs: GEDEditCosts,
        matching_threshold: usize,
    },
}

/// Result of comparing two graphs.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MatchingResult {
    /// Graphs are identical (distance = 0 or VF2 exact)
    ExactMatch,
    /// Graphs are sufficiently similar but not identical
    RelaxedMatch,
    /// Graphs do not meet the relaxed similarity threshold
    NoMatch,
}

impl AlgoGraphMatching {
    /// Computes a floating-point similarity/distance score between two graphs.
    ///
    /// - `1.0` → very similar / identical (depending on algorithm)
    /// - `0.0` → completely dissimilar
    /// - For the GED -> 0 → Equal graphs, values > 0 -> edit costs (distance)
    pub fn calc_distance(&self, one_graph: &Graph, other_graph: &Graph) -> f64 {
        match self {
            AlgoGraphMatching::CosineSimilarity {
                alpha,
                matching_threshold: _,
            } => graph_cosine_similarity(one_graph, other_graph, *alpha),
            AlgoGraphMatching::VF2IsomorphismTest => graph_vf2_isomorphism(one_graph, other_graph),
            AlgoGraphMatching::GEDFastHungarian {
                edit_costs,
                matching_threshold: _,
            } => fast_ged(one_graph, other_graph, edit_costs) as f64,
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
        match self {
            AlgoGraphMatching::CosineSimilarity {
                alpha: _,
                matching_threshold,
            } => {
                const EPS: f64 = 1e-8;
                if distance >= 1.0 - EPS {
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
            AlgoGraphMatching::GEDFastHungarian {
                edit_costs: _,
                matching_threshold,
            } => {
                let distance = distance.round() as usize;
                if distance == 0 {
                    if AlgoGraphMatching::VF2IsomorphismTest.match_graphs(one_graph, other_graph)
                        == MatchingResult::ExactMatch
                    {
                        MatchingResult::ExactMatch
                    } else {
                        MatchingResult::RelaxedMatch
                    }
                } else if distance <= *matching_threshold {
                    MatchingResult::RelaxedMatch
                } else {
                    MatchingResult::NoMatch
                }
            }
        }
    }
}

fn graph_cosine_similarity(one_graph: &Graph, other_graph: &Graph, alpha: f64) -> f64 {
    let one_graph_vertex_vector = one_graph.get_vertex_vector();
    let other_graph_vertex_vector = other_graph.get_vertex_vector();
    let one_graph_edge_vector = one_graph.get_edge_vector();
    let other_graph_edge_vector = other_graph.get_edge_vector();
    let sim_vertices =
        _calc_cosine_similarity(&one_graph_vertex_vector, &other_graph_vertex_vector);
    let sim_edges = _calc_cosine_similarity(&one_graph_edge_vector, &other_graph_edge_vector);
    alpha * sim_vertices + (1.0 - alpha) * sim_edges
}

fn _calc_cosine_similarity<T: Eq + Hash>(
    one_vec: &HashMap<T, usize>,
    other_vec: &HashMap<T, usize>,
) -> f64 {
    let mut keys = HashSet::new();
    keys.extend(one_vec.keys());
    keys.extend(other_vec.keys());
    let mut dot = 0.0f64;
    let mut norm_one = 0.0f64;
    let mut norm_other = 0.0f64;

    for key in keys {
        let v1 = *one_vec.get(key).unwrap_or(&0) as f64;
        let v2 = *other_vec.get(key).unwrap_or(&0) as f64;

        dot += v1 * v2;
        norm_one += v1 * v1;
        norm_other += v2 * v2;
    }

    if norm_one == 0.0 || norm_other == 0.0 {
        return 0.0;
    }

    dot / (norm_one.sqrt() * norm_other.sqrt())
}

fn graph_vf2_isomorphism(one_graph: &Graph, other_graph: &Graph) -> f64 {
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

/// Costs for Graph Edit Distance (GED)
#[derive(Clone, Debug)]
pub struct GEDEditCosts {
    pub node_sub: usize,
    pub node_ins: usize,
    pub node_del: usize,
    pub edge_sub: usize,
    pub edge_ins: usize,
    pub edge_del: usize,
}

impl Default for GEDEditCosts {
    fn default() -> Self {
        Self {
            node_sub: 1,
            node_ins: 1,
            node_del: 1,
            edge_sub: 1,
            edge_ins: 1,
            edge_del: 1,
        }
    }
}

/// Fast approximate graph edit distance using bipartite matching
fn fast_ged(one_graph: &Graph, other_graph: &Graph, edit_costs: &GEDEditCosts) -> usize {
    let g1 = one_graph.get_digraph();
    let g2 = other_graph.get_digraph();
    let n1 = g1.node_count();
    let n2 = g2.node_count();
    let max_n = n1.max(n2);
    let height = max_n;
    let width = max_n;

    let mut cost_matrix = vec![0usize; height * width];
    // Fill cost matrix with node substitution + edge differences
    for i in 0..height {
        for j in 0..width {
            let cost = if i < n1 && j < n2 {
                // real node substitution
                let (l1, t1) = g1.node_weight(g1.from_index(i)).unwrap();
                let (l2, t2) = g2.node_weight(g2.from_index(j)).unwrap();

                let mut c = if l1 == l2 && t1 == t2 {
                    0
                } else {
                    edit_costs.node_sub
                };

                c += edge_penalty(&g1, &g2, i, j, edit_costs);

                c
            } else if i < n1 {
                // row = real node in G1, col = dummy → delete node i
                edit_costs.node_del
            } else {
                // row = dummy, col = real node in G2 → insert node j
                edit_costs.node_ins
            };

            cost_matrix[i * width + j] = cost;
        }
    }

    let assignment = hungarian::minimize(&cost_matrix, height, width);
    let mut total: usize = 0;
    for (i, maybe_j) in assignment.iter().enumerate() {
        if let Some(j) = maybe_j {
            total += cost_matrix[i * width + j];
        }
    }

    total
}

/// Compare outgoing edges of node i in g1 vs node j in g2
fn edge_penalty(
    g1: &DiGraph<(usize, usize), usize>,
    g2: &DiGraph<(usize, usize), usize>,
    i: usize,
    j: usize,
    edit_costs: &GEDEditCosts,
) -> usize {
    let ni = g1.from_index(i);
    let nj = g2.from_index(j);

    let mut penalty = 0;

    // Outgoing edges
    let mut g1_edges = Vec::new();
    for e in g1.edges(ni) {
        g1_edges.push((e.target().index(), *e.weight()));
    }

    let mut g2_edges = Vec::new();
    for e in g2.edges(nj) {
        g2_edges.push((e.target().index(), *e.weight()));
    }

    // Compare counts
    let d = (g1_edges.len() as isize - g2_edges.len() as isize).unsigned_abs();
    penalty += d * edit_costs.edge_ins;

    // Compare overlapping edges
    let m = usize::min(g1_edges.len(), g2_edges.len());

    for k in 0..m {
        let (_, w1) = g1_edges[k];
        let (_, w2) = g2_edges[k];

        if w1 != w2 {
            penalty += edit_costs.edge_sub;
        }
    }

    penalty
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

    #[test]
    fn test_vf2_with_duplicate_labels() {
        // Graph 1
        let mut g1 = Graph::new(1);
        g1.create_vertex_with_data(1, 2); // node 0
        g1.create_vertex_with_data(2, 2); // node 1
        g1.create_vertex_with_data(1, 2); // node 2 (duplicate label)
        g1.create_vertex_with_data(3, 4); // node 3
        // edges
        g1.vertices.get_mut(0).unwrap().push(1, 0);
        g1.vertices.get_mut(1).unwrap().push(3, 0);
        g1.vertices.get_mut(2).unwrap().push(3, 0);

        // Graph 2 (ExactMatch)
        let mut g2 = Graph::new(1);
        g2.create_vertex_with_data(1, 2); // node 0
        g2.create_vertex_with_data(2, 2); // node 1
        g2.create_vertex_with_data(1, 2); // node 2 (duplicate label)
        g2.create_vertex_with_data(3, 4); // node 3
        // edges
        g2.vertices.get_mut(0).unwrap().push(1, 0);
        g2.vertices.get_mut(1).unwrap().push(3, 0);
        g2.vertices.get_mut(2).unwrap().push(3, 0);

        // Should be ExactMatch
        assert_eq!(
            AlgoGraphMatching::VF2IsomorphismTest.match_graphs(&g1, &g2),
            MatchingResult::ExactMatch
        );

        // Graph 3 (NoMatch: different topology)
        let mut g3 = Graph::new(1);
        g3.create_vertex_with_data(1, 2);
        g3.create_vertex_with_data(2, 2);
        g3.create_vertex_with_data(1, 2);
        g3.create_vertex_with_data(3, 4);
        // change edges so topology differs
        g3.vertices.get_mut(0).unwrap().push(2, 0);
        g3.vertices.get_mut(1).unwrap().push(0, 0);
        g3.vertices.get_mut(2).unwrap().push(3, 0);

        // Should be NoMatch
        assert_eq!(
            AlgoGraphMatching::VF2IsomorphismTest.match_graphs(&g1, &g3),
            MatchingResult::NoMatch
        );
    }

    #[test]
    fn test_ged_exact_match() {
        let mut g1 = Graph::new(1);
        g1.create_vertex_with_data(1, 2);
        g1.create_vertex_with_data(2, 3);
        g1.vertices.get_mut(0).unwrap().push(1, 0);
        g1.vertices.get_mut(1).unwrap().push(0, 0);

        let mut g2 = Graph::new(1);
        g2.create_vertex_with_data(1, 2);
        g2.create_vertex_with_data(2, 3);
        g2.vertices.get_mut(0).unwrap().push(1, 0);
        g2.vertices.get_mut(1).unwrap().push(0, 0);

        let algo = AlgoGraphMatching::GEDFastHungarian {
            edit_costs: GEDEditCosts::default(),
            matching_threshold: 1,
        };

        assert_eq!(algo.match_graphs(&g1, &g2), MatchingResult::ExactMatch);
    }

    #[test]
    fn test_ged_relaxed_match() {
        let mut g1 = Graph::new(1);
        g1.create_vertex_with_data(1, 2);
        g1.create_vertex_with_data(2, 3);

        let mut g2 = Graph::new(1);
        g2.create_vertex_with_data(1, 2);
        g2.create_vertex_with_data(5, 3); // node label changed

        let algo = AlgoGraphMatching::GEDFastHungarian {
            edit_costs: GEDEditCosts::default(),
            matching_threshold: 1,
        };

        // Substitution of 1 node → distance = 1 → within threshold
        assert_eq!(algo.match_graphs(&g1, &g2), MatchingResult::RelaxedMatch);
    }

    #[test]
    fn test_ged_no_match() {
        let mut g1 = Graph::new(1);
        g1.create_vertex_with_data(1, 2);

        let mut g2 = Graph::new(1);
        g2.create_vertex_with_data(2, 3);
        g2.create_vertex_with_data(3, 4);

        let algo = AlgoGraphMatching::GEDFastHungarian {
            edit_costs: GEDEditCosts::default(),
            matching_threshold: 1,
        };

        // More edits than threshold → NoMatch
        assert_eq!(algo.match_graphs(&g1, &g2), MatchingResult::NoMatch);
    }

    #[test]
    fn test_ged_edge_exact_match() {
        let mut g1 = Graph::new(1);
        g1.create_vertex_with_data(1, 2);
        g1.create_vertex_with_data(2, 3);
        g1.vertices.get_mut(0).unwrap().push(1, 5); // edge with weight 5
        g1.vertices.get_mut(1).unwrap().push(0, 2); // edge with weight 2

        let mut g2 = Graph::new(1);
        g2.create_vertex_with_data(2, 3);
        g2.create_vertex_with_data(1, 2);
        g2.vertices.get_mut(0).unwrap().push(1, 2);
        g2.vertices.get_mut(1).unwrap().push(0, 5);

        let algo = AlgoGraphMatching::GEDFastHungarian {
            edit_costs: GEDEditCosts::default(),
            matching_threshold: 1,
        };

        assert_eq!(algo.match_graphs(&g1, &g2), MatchingResult::ExactMatch);
    }

    #[test]
    fn test_ged_edge_substitution() {
        let mut g1 = Graph::new(1);
        g1.create_vertex_with_data(1, 2);
        g1.create_vertex_with_data(2, 3);
        g1.vertices.get_mut(0).unwrap().push(1, 5); // edge with weight 5

        let mut g2 = Graph::new(1);
        g2.create_vertex_with_data(1, 2);
        g2.create_vertex_with_data(2, 3);
        g2.vertices.get_mut(0).unwrap().push(1, 7); // weight changed

        let algo = AlgoGraphMatching::GEDFastHungarian {
            edit_costs: GEDEditCosts::default(),
            matching_threshold: 1,
        };

        // 1 edge substitution → distance = 1 → within threshold → RelaxedMatch
        assert_eq!(algo.match_graphs(&g1, &g2), MatchingResult::RelaxedMatch);
    }

    #[test]
    fn test_ged_edge_insertion_deletion() {
        let mut g1 = Graph::new(1);
        g1.create_vertex_with_data(1, 2);
        g1.create_vertex_with_data(2, 3);
        g1.vertices.get_mut(0).unwrap().push(1, 5); // g1 has 1 edge

        let mut g2 = Graph::new(1);
        g2.create_vertex_with_data(1, 2);
        g2.create_vertex_with_data(2, 3);
        // g2 has no edges → edge deletion penalty

        let algo = AlgoGraphMatching::GEDFastHungarian {
            edit_costs: GEDEditCosts::default(),
            matching_threshold: 1,
        };

        // 1 edge deleted → distance = 1 → within threshold → RelaxedMatch
        assert_eq!(algo.match_graphs(&g1, &g2), MatchingResult::RelaxedMatch);
    }

    #[test]
    fn test_ged_edge_no_match_due_to_multiple_edits() {
        let mut g1 = Graph::new(1);
        g1.create_vertex_with_data(1, 2);
        g1.create_vertex_with_data(2, 3);
        g1.create_vertex_with_data(3, 4);
        g1.vertices.get_mut(0).unwrap().push(1, 5); // edge 0→1
        g1.vertices.get_mut(1).unwrap().push(2, 2); // edge 1→2

        let mut g2 = Graph::new(1);
        g2.create_vertex_with_data(1, 2);
        g2.create_vertex_with_data(2, 3);
        g2.create_vertex_with_data(3, 4);
        g2.vertices.get_mut(0).unwrap().push(2, 7); // edge 0→2 (different target and weight)
        g2.vertices.get_mut(1).unwrap().push(0, 3); // edge 1→0 (different target and weight)

        let algo = AlgoGraphMatching::GEDFastHungarian {
            edit_costs: GEDEditCosts::default(),
            matching_threshold: 1,
        };

        // multiple edge substitutions → distance > threshold → NoMatch
        assert_eq!(algo.match_graphs(&g1, &g2), MatchingResult::NoMatch);
    }
}
