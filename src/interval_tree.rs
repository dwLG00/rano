extern crate petgraph;
use petgraph::{Graph, Directed};
use petgraph::graph::{EdgeIndex, NodeIndex};

pub struct Interval<T> {
    lrange: usize,
    rrange: usize,
    max: usize,
    val: T
}

pub struct IntervalTree<T> {
    graph: Graph<Interval<T>, bool, Directed>,
    graph_head: Option<NodeIndex<u32>>
}

impl<T> IntervalTree<T> {
    pub fn new() -> IntervalTree<T> {
        let graph = Graph::<Interval<T>, bool, Directed>::new();
        IntervalTree { graph: graph, graph_head: None }
    }

    pub fn insert(&mut self, interval: Interval<T>) {
        // Inserts an interval
        match self.graph_head {
            None => {
                let head = self.graph.add_node(interval);
                self.graph_head = Some(head);
            },
            Some(head) => {
                let head_interval = self.graph.node_weight(head).unwrap();
            }
        }
    }
}
