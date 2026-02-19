use crate::utf8::UnicodeCodepoint;
use std::sync::atomic::{AtomicUsize, Ordering};

static GRAPH_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct Graph {
    nodes: Vec<Node>,
    id: usize,
}

#[derive(Clone, Debug, Default)]
struct Node {
    is_final: bool,
    edges: Vec<(usize, UnicodeCodepoint)>,
    epsilon_edges: Vec<usize>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NodeRef {
    graph_id: usize,
    index: usize,
}

impl PartialEq for Graph {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Graph {}

impl Clone for Graph {
    fn clone(&self) -> Self {
        Graph {
            nodes: self.nodes.clone(),
            id: GRAPH_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            nodes: vec![Node::default()],
            id: GRAPH_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    fn owns_node(&self, x: NodeRef) -> bool {
        self.id == x.graph_id
    }

    fn get_node(&self, x: NodeRef) -> &Node {
        assert!(self.owns_node(x));
        &self.nodes[x.index]
    }

    fn get_node_mut(&mut self, x: NodeRef) -> &mut Node {
        assert!(self.owns_node(x));
        &mut self.nodes[x.index]
    }

    pub fn get_initial_node(&self) -> NodeRef {
        NodeRef {
            graph_id: self.id,
            index: 0,
        }
    }

    pub fn add_node(&mut self) -> NodeRef {
        self.nodes.push(Node::default());
        NodeRef {
            graph_id: self.id,
            index: self.nodes.len() - 1,
        }
    }

    /// Panics if `x` or `y` doesn't belong to `self`
    pub fn connect(&mut self, x: NodeRef, y: NodeRef, token: UnicodeCodepoint) {
        assert!(self.owns_node(y));
        self.get_node_mut(x).edges.push((y.index, token));
    }

    /// Panics if `x` or `y` doesn't belong to `self`
    pub fn connect_epsilon(&mut self, x: NodeRef, y: NodeRef) {
        assert!(self.owns_node(y));
        self.get_node_mut(x).epsilon_edges.push(y.index);
    }

    /// Panics if `x` doesn't belong to `self`
    pub fn get_connections(&self, x: NodeRef) -> impl Iterator<Item = NodeRef> {
        self.get_node(x).edges.iter().map(|(e, _)| NodeRef {
            graph_id: self.id,
            index: *e,
        })
    }

    /// Panics if `x` doesn't belong to `self`
    pub fn get_epsilon_connections(
        &self,
        x: NodeRef,
    ) -> impl Iterator<Item = NodeRef> {
        self.get_node(x).epsilon_edges.iter().map(|e| NodeRef {
            graph_id: self.id,
            index: *e,
        })
    }

    /// Panics if `x` doesn't belong to `self`
    pub fn is_final(&self, x: NodeRef) -> bool {
        self.get_node(x).is_final
    }

    /// Panics if `x` doesn't belong to `self`
    pub fn set_final(&mut self, x: NodeRef) {
        self.get_node_mut(x).is_final = true;
    }

    pub fn collapse_epsilons(&mut self) {
        for a in 0..self.nodes.len() {
            while let Some(b) = self.nodes[a].epsilon_edges.pop() {
                if a == b {
                    continue;
                }
                if self.nodes[b].is_final {
                    self.nodes[a].is_final = true;
                }
                for i in 0..self.nodes[b].edges.len() {
                    let c = self.nodes[b].edges[i];
                    self.nodes[a].edges.push(c);
                }
                for i in 0..self.nodes[b].epsilon_edges.len() {
                    let c = self.nodes[b].epsilon_edges[i];
                    self.nodes[a].epsilon_edges.push(c);
                }
            }
        }
    }

    pub fn debug_string(&self) -> String {
        let mut s = String::new();
        for (a_node, a) in self.nodes.iter().zip(0..) {
            for (b, token) in &a_node.edges {
                s.push_str(&format!("{} {} {}\n", a, b, char::from(*token)));
            }
            for b in &a_node.epsilon_edges {
                s.push_str(&format!("{} {} ε\n", a, b));
            }
        }
        s
    }
}
