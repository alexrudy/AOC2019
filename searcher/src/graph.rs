//! Graph decomposition and datastructures.

use std::cmp::{Ord, PartialEq};
use std::collections::{BinaryHeap, HashMap};
use std::convert::Into;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::Sum;

mod edge;
mod path;
mod traits;

pub use edge::Edge;
use edge::WeightedEdge;
pub use path::GraphPath;
pub use traits::Graphable;

type Nodes<E> = HashMap<<E as Edge>::Node, HashMap<<E as Edge>::Node, WeightedEdge<E>>>;

#[derive(Debug)]
pub struct GraphBuilder<'g, E, G>
where
    E: Edge,
    G: Graphable + ?Sized,
{
    nodes: Nodes<E>,
    graphable: &'g G,
}

impl<'g, N, E, G, W> GraphBuilder<'g, E, G>
where
    N: Debug + Clone + Hash + Eq + PartialEq,
    E: Edge<Weight = W, Node = N>,
    W: Clone + Ord + Sum,
    G: Graphable<Edge = E>,
{
    pub fn insert(&mut self, edge: (N, E, N)) -> bool {
        if edge.0 == edge.2 {
            return false;
        }

        // Left to right
        let wedge: WeightedEdge<E> = edge.1.into();

        let connections = self.nodes.entry(edge.0.clone()).or_insert(HashMap::new());
        connections
            .entry(edge.2.clone())
            .and_modify(|e| {
                if *e < wedge {
                    *e = wedge.clone();
                }
            })
            .or_insert_with(|| wedge.clone());

        // Right to left
        let connections = self.nodes.entry(edge.2.clone()).or_insert(HashMap::new());
        connections
            .entry(edge.0.clone())
            .and_modify(|e| {
                if *e < wedge {
                    *e = wedge.reverse();
                }
            })
            .or_insert_with(|| wedge.reverse());

        // TODO: This should reflect whether we actually did the insert
        true
    }

    /// Explore a map starting at the given point,
    /// adding appropriate edges to the graph.
    pub fn explore(&mut self, origin: N) {
        let mut queue: BinaryHeap<WeightedEdge<E>> = BinaryHeap::new();
        let mut visited = HashMap::new();

        queue.push(E::new(origin.clone()).into());

        while let Some(WeightedEdge { edge: path, .. }) = queue.pop() {
            if (self.graphable.is_node(path.destination()) || path.destination() == &origin)
                && !path.is_empty()
            {
                // We've found a node, stick an edge in both directions.
                let o = path.origin().clone();
                let d = path.destination().clone();

                self.insert((o, path.clone(), d));

                let key = (path.origin().clone(), path.destination().clone());
                if let Some(w) = visited.get(&key) {
                    if w <= &path.weight() {
                        continue;
                    }
                }
                visited.insert(key, path.weight());

                let stub = E::new(path.destination().clone());

                for (n, e) in self.graphable.neighbors(path.destination()) {
                    queue.push(stub.step(n.clone(), e.clone()).into())
                }
            } else {
                let key = (path.origin().clone(), path.destination().clone());
                if let Some(w) = visited.get(&key) {
                    if w <= &path.weight() {
                        continue;
                    }
                }
                visited.insert(key, path.weight());

                for (n, e) in self.graphable.neighbors(path.destination()) {
                    queue.push(path.step(n.clone(), e.clone()).into())
                }
            }
        }
    }

    /// Create an empty graph
    pub fn new(graphable: &'g G) -> Self {
        Self {
            nodes: HashMap::new(),
            graphable,
        }
    }

    pub fn build(self) -> Graph<E> {
        Graph { nodes: self.nodes }
    }
}

pub fn builder<W, E, G>(g: &G) -> GraphBuilder<E, G>
where
    G: Graphable<Edge = E>,
    E: Edge<Weight = W>,
    W: Ord + Sum + Clone,
{
    GraphBuilder::new(g)
}

#[derive(Debug)]
pub struct Graph<E>
where
    E: Edge,
{
    nodes: Nodes<E>,
}

impl<N, E> Graph<E>
where
    N: Debug + Clone + Hash + Eq + PartialEq + 'static,
    E: Edge<Node = N>,
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

impl<N, E> Graph<E>
where
    N: Debug + Clone + Hash + Eq + PartialEq,
    E: Edge<Weight = usize, Node = N>,
{
    /// Find a path within the graph.
    ///
    /// Returns None when no path can be found, or when origin or destination
    /// are not nodes in the graph.
    pub fn find_path(&self, origin: N, destination: N) -> Option<GraphPath<N, E>> {
        use crate::dijkstra;
        use crate::SearchOptions;

        // Chech that start and endpoints are nodes.
        // TODO: Could dynamically add nodes to the graph as new options appear?
        if !(self.nodes.contains_key(&origin) && self.nodes.contains_key(&destination)) {
            return None;
        }

        let options = {
            let mut o = SearchOptions::default();
            o.verbose = Some(1);
            o.exhaustive = true;
            o
        };

        let c = graphsearch::GraphPathCandidate::start(origin, &destination, &self);
        dijkstra::build(c)
            .with_options(options)
            .run()
            .ok()
            .map(|c| c.path)
    }
}

mod graphsearch {

    use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
    use std::fmt::Debug;
    use std::hash::Hash;
    use std::iter::Sum;

    use super::path::GraphPath;
    use super::{Edge, Graph};
    use crate::{SearchCacher, SearchCandidate, SearchState};

    #[derive(Debug, Clone)]
    pub(crate) struct GraphPathCandidate<'m, N, E>
    where
        N: Debug + Clone,
        E: Edge<Node = N>,
    {
        pub(crate) path: GraphPath<N, E>,
        destination: &'m N,
        graph: &'m Graph<E>,
    }

    impl<'m, N, E, W> GraphPathCandidate<'m, N, E>
    where
        N: Debug + Hash + Clone + Eq,
        E: Edge<Weight = W, Node = N>,
        W: Sum + Ord + Eq + Debug + Copy,
    {
        pub(crate) fn start(origin: N, destination: &'m N, graph: &'m Graph<E>) -> Self {
            Self {
                path: GraphPath::new(origin),
                destination: destination,
                graph: graph,
            }
        }

        fn step(&self, node: N, edge: E) -> Self {
            let ne = GraphPath::new(self.path.destination().clone())
                .step_one(node.clone(), edge.clone());
            Self {
                path: self.path.step(node, ne),
                destination: self.destination,
                graph: self.graph,
            }
        }
    }

    impl<'m, N, E> SearchCandidate for GraphPathCandidate<'m, N, E>
    where
        N: Debug + Clone + Hash + Eq + PartialEq,
        E: Edge<Weight = usize, Node = N>,
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
        N: Debug + Clone + Hash + Eq + PartialEq,
        E: Edge<Weight = usize, Node = N>,
    {
        type State = N;

        fn state(&self) -> Self::State {
            self.path.destination().clone()
        }
    }

    impl<'m, N, E> SearchCacher for GraphPathCandidate<'m, N, E>
    where
        N: Debug + Clone + Hash + Eq + PartialEq,
        E: Edge<Weight = usize, Node = N>,
    {
        type Value = usize;

        fn value(&self) -> Self::Value {
            self.path.weight()
        }
    }

    impl<'m, N, E> Ord for GraphPathCandidate<'m, N, E>
    where
        N: Debug + Clone + Hash + Eq + PartialEq,
        E: Edge<Weight = usize, Node = N>,
    {
        fn cmp(&self, other: &Self) -> Ordering {
            self.path.weight().cmp(&other.path.weight()).reverse()
        }
    }

    impl<'m, N, E> PartialOrd for GraphPathCandidate<'m, N, E>
    where
        N: Debug + Clone + Hash + Eq + PartialEq,
        E: Edge<Weight = usize, Node = N>,
    {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<'m, N, E> PartialEq for GraphPathCandidate<'m, N, E>
    where
        N: Debug + Clone + Hash + Eq + PartialEq,
        E: Edge<Weight = usize, Node = N>,
    {
        fn eq(&self, other: &Self) -> bool {
            self.path.weight().eq(&other.path.weight())
        }
    }

    impl<'m, N, E> Eq for GraphPathCandidate<'m, N, E>
    where
        N: Debug + Clone + Hash + Eq + PartialEq,
        E: Edge<Weight = usize, Node = N>,
    {
    }
}
