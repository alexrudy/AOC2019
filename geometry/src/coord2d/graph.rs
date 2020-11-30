//! Graph decomposition for fast pathfinding
//!
//! For complex 2D maps, sometimes it is helpful
//! to decompose the map into a graph, where each
//! node is either a point of interest or a decision
//! point, where path finding would have to make a turn.

use std::fmt::Debug;

use super::map::Map;
use super::path::Path;
use super::{Direction, Point};
use graphedge::GPath;
use searcher::graph;

pub use graphinterest::GraphWithInterest;

mod graphedge {
    use std::cmp;

    use super::Path;
    use super::Point;
    use searcher::graph::Edge;

    #[derive(Debug, Clone)]
    pub struct GPath {
        pub(crate) path: Path,
    }

    impl cmp::Ord for GPath {
        fn cmp(&self, other: &Self) -> cmp::Ordering {
            self.path.distance().cmp(&other.path.distance())
        }
    }

    impl cmp::PartialOrd for GPath {
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl cmp::PartialEq for GPath {
        fn eq(&self, other: &Self) -> bool {
            self.path.distance().eq(&other.path.distance())
        }
    }

    impl cmp::Eq for GPath {}

    impl From<Path> for GPath {
        fn from(path: Path) -> Self {
            GPath { path }
        }
    }

    impl Edge for GPath {
        type Weight = usize;
        type Node = Point;

        fn weight(&self) -> Self::Weight {
            self.path.distance()
        }

        fn new(node: Self::Node) -> Self {
            Self {
                path: Path::new(node),
            }
        }

        fn origin(&self) -> &Self::Node {
            self.path.origin()
        }

        fn destination(&self) -> &Self::Node {
            self.path.destination()
        }

        fn is_empty(&self) -> bool {
            self.path.is_empty()
        }

        fn reverse(&self) -> Self {
            Self {
                path: self.path.reversed(),
            }
        }

        #[allow(unused_variables)]
        fn step(&self, node: Self::Node, edge: Self) -> Self {
            self.path.step(edge.path.last_direction().unwrap()).into()
        }
    }
}

mod graphdecomp {

    use searcher::graph::{builder, Edge};
    use searcher::graph::{Graph, Graphable};

    use super::graphedge;
    use super::Graphable as MapGraphable;
    use super::Path;
    use super::{Direction, Point};

    pub(crate) struct GraphableMap<'m, M>
    where
        M: MapGraphable,
    {
        map: &'m M,
    }

    impl<'m, M> GraphableMap<'m, M>
    where
        M: MapGraphable,
    {
        pub(crate) fn new(map: &'m M) -> Self {
            Self { map }
        }
    }

    impl<'m, M> Graphable for GraphableMap<'m, M>
    where
        M: MapGraphable,
    {
        type Edge = graphedge::GPath;

        fn is_node(&self, node: &<Self::Edge as Edge>::Node) -> bool {
            self.map.is_node(node)
        }

        fn neighbors(
            &self,
            node: &<Self::Edge as Edge>::Node,
        ) -> Vec<(<Self::Edge as Edge>::Node, Self::Edge)> {
            let origin = *node;

            let iter = Direction::all().filter_map(move |d| {
                let n = origin.step(d);
                if self.map.is_traversable(n) {
                    let e = Path::new(origin).step(d);
                    Some((*e.destination(), e.into()))
                } else {
                    None
                }
            });
            iter.collect()
        }
    }

    pub(crate) fn build<M>(map: &M, origin: Point) -> Graph<graphedge::GPath>
    where
        M: MapGraphable,
    {
        let gm = GraphableMap::new(map);
        let mut b = builder(&gm);
        eprintln!("Set up builder");
        b.explore(origin);
        eprintln!("Explored {:?}", origin);
        b.build()
    }
}

mod graphinterest {

    use std::collections::HashSet;
    use std::fmt::Debug;

    use super::graphdecomp::GraphableMap;
    use super::Graphable as MapGraphable;
    use super::Map;
    use super::Point;

    /// Wrapper structure for graphs which add points
    /// of interest as nodes along with junctions from
    /// the standard graph.
    #[derive(Debug)]
    pub struct GraphWithInterest<M>
    where
        M: MapGraphable,
    {
        map: M,
        interest_points: HashSet<Point>,
    }

    impl<M> GraphWithInterest<M>
    where
        M: MapGraphable,
    {
        /// Create a new graph with points of interest from a map.
        pub fn new(map: M) -> Self {
            Self {
                map,
                interest_points: HashSet::new(),
            }
        }

        pub(crate) fn grapher(&self) -> GraphableMap<Self> {
            GraphableMap::new(self)
        }

        /// Add a point of interest to a map, which will be used as
        /// a node.
        pub fn insert(&mut self, point: Point) -> bool {
            self.interest_points.insert(point)
        }
    }

    impl<M> Map for GraphWithInterest<M>
    where
        M: MapGraphable,
    {
        fn is_traversable(&self, location: Point) -> bool {
            self.map.is_traversable(location)
        }
    }

    impl<M> MapGraphable for GraphWithInterest<M>
    where
        M: MapGraphable,
    {
        fn is_node(&self, point: &Point) -> bool {
            self.interest_points.contains(point) || self.map.is_node(point)
        }
    }
}

#[derive(Debug)]
pub struct RawGraph(graph::Graph<GPath>);

impl RawGraph {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn find_path(&self, origin: Point, destination: Point) -> Option<Path> {
        self.0.find_path(origin, destination).map(|p| {
            eprintln!("{:?}", p);
            let mut prev = None;
            p.edges()
                .iter()
                .flat_map(|e| e.path.iter().copied())
                .filter_map(|p| {
                    if Some(p) == prev {
                        None
                    } else {
                        prev = Some(p);
                        Some(p)
                    }
                })
                .collect::<Vec<Point>>()
                .into()
        })
    }

    pub fn edges(&self, node: &Point) -> impl Iterator<Item = (&Point, &Path)> {
        self.0.edges(*node).map(|(n, g)| (n, &g.path))
    }

    pub fn contains(&self, node: &Point) -> bool {
        self.0.contains_node(node)
    }
}

#[derive(Debug)]
pub struct Graph<'m, M>(RawGraph, &'m M);

impl<'m, M> Graph<'m, M> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn find_path(&self, origin: Point, destination: Point) -> Option<Path> {
        self.0.find_path(origin, destination)
    }

    pub fn edges(&self, node: &Point) -> impl Iterator<Item = (&Point, &Path)> {
        self.0.edges(node)
    }

    pub fn raw(self) -> RawGraph {
        self.0
    }
}

pub trait Graphable: Map {
    fn is_node(&self, location: &Point) -> bool;

    fn graph(&self, origin: Point) -> Graph<Self> {
        Graph(RawGraph(graphdecomp::build(self, origin)), &self)
    }

    fn grapher<'a, I>(&self, points: I) -> Graph<Self>
    where
        I: Iterator<Item = &'a Point>,
    {
        let gm = graphdecomp::GraphableMap::new(self);
        let mut b = graph::builder(&gm);
        for point in points {
            b.explore(*point);
        }
        Graph(RawGraph(b.build()), &self)
    }
}

#[cfg(test)]
mod test {

    use super::super::map::helpers::*;
    use super::*;

    impl Graphable for SimpleMap {
        fn is_node(&self, location: &Point) -> bool {
            let options = Direction::all()
                .filter(|d| self.is_traversable(location.step(*d)))
                .count();

            options != 2
        }
    }

    #[test]
    fn simple() {
        let map: SimpleMap = vec![(0, 0).into()].into();

        let graph = graphdecomp::build(&map, (0, 0).into());

        assert_eq!(graph.len(), 0);
    }

    #[test]
    fn shortest() {
        let map: SimpleMap = include_str!("../../examples/pathfinding_multi.txt")
            .parse()
            .unwrap();

        let mut poi = GraphWithInterest::new(map);
        poi.insert((1, 1).into());
        poi.insert((1, 12).into());

        let graph = poi.graph((1, 1).into());

        assert_eq!(graph.len(), 16);

        let path = graph.find_path((1, 1).into(), (1, 12).into()).unwrap();
        eprintln!("{:?}", path);
        assert_eq!(path.distance(), 19);
    }
}
