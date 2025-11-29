use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct Edge {
    pub id: usize,
    pub from: usize,
    pub to: usize,
    pub e_label: usize,
}

impl Edge {
    pub fn new(from: usize, to: usize, e_label: usize) -> Edge {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        Edge {
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
            from,
            to,
            e_label,
        }
    }
    pub fn to_str_repr(&self) -> String {
        vec![
            "e".to_string(),
            self.from.to_string(),
            self.to.to_string(),
            self.e_label.to_string(),
        ]
        .join(" ")
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.e_label == other.e_label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_edge() {
        let edge1 = Edge::new(1, 2, 3);
        let edge2 = Edge::new(1, 2, 3);
        let edge3 = Edge::new(2, 1, 3);

        assert_eq!(edge1, edge2);
        assert_ne!(edge1, edge3);
        assert_ne!(edge2, edge3);
    }
}
