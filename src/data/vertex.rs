use std::isize;

use crate::data::edge::Edge;

#[derive(Debug)]
pub struct Vertex {
    pub id: usize,
    pub label: isize,
    pub v_type: isize,
    pub edges: Vec<Edge>,
}

impl Vertex {
    pub fn new(id: usize, label: Option<isize>, v_type: Option<isize>) -> Vertex {
        Vertex {
            id,
            label: match label {
                None => 0,
                Some(label) => label,
            },
            v_type: v_type.unwrap_or(0),
            edges: Vec::with_capacity(8),
        }
    }

    pub fn push(&mut self, to: usize, e_label: usize) {
        self.edges.push(Edge::new(self.id, to, e_label));
    }

    pub fn to_str_repr(&self) -> String {
        vec![
            "v".to_string(),
            self.id.to_string(),
            self.label.to_string(),
            self.v_type.to_string(),
        ]
        .join(" ")
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.v_type == other.v_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vertex() {
        let v1 = Vertex::new(1, None, None);
        assert_eq!(v1.id, 1);
    }

    #[test]
    fn test_add_edge() {
        let mut v1 = Vertex::new(1, Some(2), Some(3));
        assert_eq!(v1.edges.len(), 0);
        assert_eq!(v1.label, 2);
        assert_eq!(v1.v_type, 3);
        v1.push(2, 2);
        assert_eq!(v1.edges.len(), 1);
        let e = v1.edges.pop().unwrap();
        assert_eq!(v1.edges.len(), 0);
        assert_eq!(e.to, 2);
        assert_eq!(e.from, 1);
        assert_eq!(e.e_label, 2);
    }
}
