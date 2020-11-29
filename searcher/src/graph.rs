//! Graph decomposition and datastructures.

use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::convert::{From, Into};
use std::fmt::Debug;
use std::hash::Hash;

mod edge;

pub use edge::Edge;
use edge::WeightedEdge;

pub trait Node: Debug + PartialEq + Eq + Hash + Clone {}

type Nodes<N, E> = HashMap<N, HashMap<N, WeightedEdge<E>>>;

#[derive(Debug)]
pub struct GraphBuilder<N, E>
where
    N: Node,
    E: Edge,
{
    nodes: Nodes<N, E>,
}

impl<N, E> GraphBuilder<N, E>
where
    N: Node,
    E: Edge,
{
    pub fn insert(&mut self, edge: (N, E, N)) -> bool {
        // Left to right
        let wedge: WeightedEdge<E> = edge.1.into();

        let connections = self.nodes.entry(edge.0.clone()).or_insert(HashMap::new());
        connections
            .entry(edge.2.clone())
            .and_modify(|e| {
                if *e > wedge {
                    *e = wedge.clone();
                }
            })
            .or_insert_with(|| wedge.clone());

        // Right to left
        let connections = self.nodes.entry(edge.2.clone()).or_insert(HashMap::new());
        connections
            .entry(edge.0.clone())
            .and_modify(|e| {
                if *e > wedge {
                    *e = wedge.clone();
                }
            })
            .or_insert_with(|| wedge.clone());

        // TODO: This should reflect whether we actually did the insert
        true
    }

    /// Create an empty graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn build(self) -> Graph<N, E> {
        Graph { nodes: self.nodes }
    }
}
#[derive(Debug)]
pub struct Graph<N, E>
where
    N: Node,
    E: Edge,
{
    nodes: Nodes<N, E>,
}

impl<N, E> Graph<N, E>
where
    N: Node,
    E: Edge,
{
    pub fn contains_node(&self, node: &N) -> bool {
        self.nodes.contains_key(node)
    }

    /// Number of nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn nodes(&self) -> impl Iterator<Item = &N> {
        self.nodes.keys()
    }

    /// Iterate through the edges of a graph which connect to this node.
    pub fn edges(&self, node: N) -> impl Iterator<Item = (&N, &E)> {
        self.nodes
            .get(&node)
            .expect(&format!("{:?} is not a node", node))
            .iter()
            .map(|(n, w)| (n, &w.edge))
    }
}

impl<N, E> Graph<N, E>
where
    N: Node,
    E: Edge<Weight = usize>,
{
    /// Find a path within the graph.
    ///
    /// Returns None when no path can be found, or when origin or destination
    /// are not nodes in the graph.
    pub fn find_path(&self, origin: N, destination: N) -> Option<graphsearch::GraphPath<N, E>> {
        use crate::dijkstra;

        // Chech that start and endpoints are nodes.
        // TODO: Could dynamically add nodes to the graph as new options appear?
        if !(self.nodes.contains_key(&origin) && self.nodes.contains_key(&destination)) {
            return None;
        }

        let c = graphsearch::GraphPathCandidate::start(origin, &destination, &self);
        dijkstra::run(c).ok().map(|c| c.path)
    }
}

mod graphsearch {

    use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

    use crate::{SearchCacher, SearchCandidate, SearchState};

    use super::{Edge, Graph, Node};

    #[derive(Debug, Clone)]
    pub struct GraphPath<N, E>
    where
        N: Node,
        E: Edge,
    {
        pub(crate) nodes: Vec<N>,
        pub(crate) edges: Vec<E>,
    }

    impl<N, E> GraphPath<N, E>
    where
        N: Node,
        E: Edge,
    {
        fn new(origin: N) -> Self {
            Self {
                nodes: vec![origin],
                edges: Vec::new(),
            }
        }

        fn step(&self, node: N, edge: E) -> Self {
            let mut nextpath = self.clone();
            nextpath.nodes.push(node);
            nextpath.edges.push(edge);
            nextpath
        }

        fn destination(&self) -> &N {
            self.nodes.last().unwrap()
        }

        fn penultimate(&self) -> Option<&N> {
            let n = self.nodes.len();
            if n > 1 {
                Some(&self.nodes[n - 2])
            } else {
                None
            }
        }
    }

    impl<N, E> GraphPath<N, E>
    where
        N: Node,
        E: Edge<Weight = usize>,
    {
        fn weight(&self) -> usize {
            self.edges.iter().map(|e| e.weight()).sum()
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct GraphPathCandidate<'m, N, E>
    where
        N: Node,
        E: Edge,
    {
        pub(crate) path: GraphPath<N, E>,
        destination: &'m N,
        graph: &'m Graph<N, E>,
    }

    impl<'m, N, E> GraphPathCandidate<'m, N, E>
    where
        N: Node,
        E: Edge,
    {
        pub(crate) fn start(origin: N, destination: &'m N, graph: &'m Graph<N, E>) -> Self {
            Self {
                path: GraphPath::new(origin),
                destination: destination,
                graph: graph,
            }
        }

        fn step(&self, node: N, edge: E) -> Self {
            Self {
                path: self.path.step(node, edge),
                destination: self.destination,
                graph: self.graph,
            }
        }
    }

    impl<'m, N, E> SearchCandidate for GraphPathCandidate<'m, N, E>
    where
        N: Node,
        E: Edge<Weight = usize>,
    {
        fn is_complete(&self) -> bool {
            self.destination == self.path.destination()
        }

        fn children(&self) -> Vec<Self> {
            let mut paths = Vec::new();
            let node = self.path.destination();
            let backtrack = self.path.penultimate();

            for (destination, path) in self.graph.nodes.get(&node).unwrap() {
                if Some(destination) != backtrack {
                    paths.push(self.step(destination.clone(), path.edge.clone()))
                }
            }

            paths
        }
    }

    impl<'m, N, E> SearchState for GraphPathCandidate<'m, N, E>
    where
        N: Node,
        E: Edge<Weight = usize>,
    {
        type State = N;

        fn state(&self) -> Self::State {
            self.path.destination().clone()
        }
    }

    impl<'m, N, E> SearchCacher for GraphPathCandidate<'m, N, E>
    where
        N: Node,
        E: Edge<Weight = usize>,
    {
        type Value = usize;

        fn value(&self) -> Self::Value {
            self.path.weight()
        }
    }

    impl<'m, N, E> Ord for GraphPathCandidate<'m, N, E>
    where
        N: Node,
        E: Edge<Weight = usize>,
    {
        fn cmp(&self, other: &Self) -> Ordering {
            self.path.weight().cmp(&other.path.weight())
        }
    }

    impl<'m, N, E> PartialOrd for GraphPathCandidate<'m, N, E>
    where
        N: Node,
        E: Edge<Weight = usize>,
    {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<'m, N, E> PartialEq for GraphPathCandidate<'m, N, E>
    where
        N: Node,
        E: Edge<Weight = usize>,
    {
        fn eq(&self, other: &Self) -> bool {
            self.path.weight().eq(&other.path.weight())
        }
    }

    impl<'m, N, E> Eq for GraphPathCandidate<'m, N, E>
    where
        N: Node,
        E: Edge<Weight = usize>,
    {
    }
}
