use std::collections::{HashMap, HashSet};

use searcher::dijkstra;

use super::map::Map;
use super::path::Path;
use super::{Direction, Point};

pub trait Graphable: Map {
    fn is_node(&self, point: &Point) -> bool {
        let options = self.movement_options(point);
        options == 1 || options > 2
    }

    fn movement_options(&self, point: &Point) -> usize {
        Direction::all()
            .filter(|d| self.is_traversable(point.step(*d)))
            .count()
    }

    fn graph(&self, origin: Point) -> Graph<Self> {
        Graph::new(self, origin)
    }
}

#[derive(Debug)]
pub struct GraphWithInterest<M>
where
    M: Graphable,
{
    map: M,
    interest_points: HashSet<Point>,
}

impl<M> GraphWithInterest<M>
where
    M: Graphable,
{
    pub fn new(map: M) -> Self {
        Self {
            map,
            interest_points: HashSet::new(),
        }
    }

    pub fn insert(&mut self, point: Point) -> bool {
        self.interest_points.insert(point)
    }
}

impl<M> Map for GraphWithInterest<M>
where
    M: Graphable,
{
    fn is_traversable(&self, location: Point) -> bool {
        self.map.is_traversable(location)
    }
}

impl<M> Graphable for GraphWithInterest<M>
where
    M: Graphable,
{
    fn is_node(&self, point: &Point) -> bool {
        self.map.is_node(point) || self.interest_points.contains(point)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct Node(Point);

#[derive(Debug, Clone)]
pub struct Graph<'m, M>
where
    M: Graphable,
{
    nodes: HashMap<Point, HashMap<Point, Path>>,
    map: &'m M,
}

impl<'m, M> Graph<'m, M>
where
    M: Graphable,
{
    pub fn contains(&self, point: &Point) -> bool {
        self.nodes.contains_key(point)
    }

    fn add_edge(&mut self, path: &Path) {
        self.nodes
            .entry(*path.origin())
            .or_insert(HashMap::new())
            .entry(*path.destination())
            .and_modify(|p| {
                if p.distance() > path.distance() {
                    *p = path.clone();
                }
            })
            .or_insert_with(|| path.clone());

        self.nodes
            .entry(*path.destination())
            .or_insert(HashMap::new())
            .entry(*path.origin())
            .and_modify(|p| {
                if p.distance() > path.distance() {
                    *p = path.reversed();
                }
            })
            .or_insert_with(|| path.reversed());
    }

    fn empty(map: &'m M) -> Self {
        Self {
            map: map,
            nodes: HashMap::new(),
        }
    }

    pub fn new(map: &'m M, origin: Point) -> Self {
        let mut graph = Self::empty(map);
        let mut queue = Vec::new();
        let mut visited = HashSet::new();

        queue.push(Path::new(origin));

        while let Some(path) = queue.pop() {
            if map.is_node(&path.destination()) && path.distance() > 0 {
                // We've found a node, stick an edge in both directions.
                graph.add_edge(&path);

                if !visited.insert(*path.destination()) {
                    continue;
                }

                let stub = Path::new(*path.destination());

                for d in Direction::all() {
                    if Some(d) != path.last_direction().map(|ld| ld.reverse()) {
                        let next = path.destination().step(d);
                        if map.is_traversable(next) {
                            queue.push(stub.step(d))
                        }
                    }
                }
            } else {
                if !visited.insert(*path.destination()) {
                    continue;
                }

                for d in Direction::all() {
                    if Some(d) != path.last_direction().map(|ld| ld.reverse()) {
                        let next = path.destination().step(d);
                        if map.is_traversable(next) {
                            queue.push(path.step(d))
                        }
                    }
                }
            }
        }

        graph
    }

    pub fn edges(&self, location: Point) -> impl Iterator<Item = (&Point, &Path)> {
        self.nodes.get(&location).unwrap().iter()
    }

    pub fn find_path(&self, origin: Point, destination: Point) -> Option<Path> {
        // Chech that start and endpoints are nodes.
        // TODO: Could dynamically add nodes to the graph as new options appear?
        if !(self.nodes.contains_key(&origin) && self.nodes.contains_key(&destination)) {
            return None;
        }

        let c = graphsearch::GraphPathCandidate::start(origin, &destination, &self);
        dijkstra(c).run().ok().map(|c| self.expand_path(&c.path))
    }

    fn expand_path(&self, graphpath: &graphsearch::GraphPath) -> Path {
        let mut path = Vec::new();
        let mut location = *graphpath.nodes.first().unwrap();
        path.push(location);

        for node in graphpath.nodes.iter().skip(1) {
            let nodepath = self.nodes.get(&location).unwrap().get(node).unwrap();
            path.extend(nodepath.iter().skip(1));
            location = *node;
        }

        path.into()
    }
}

mod graphsearch {

    use searcher::{SearchCacher, SearchCandidate};

    use super::{Graph, Graphable, Point};

    #[derive(Debug)]
    pub(crate) struct GraphPath {
        pub(crate) nodes: Vec<Point>,
        distance: usize,
    }

    impl GraphPath {
        fn new(origin: Point) -> Self {
            Self {
                nodes: vec![origin],
                distance: 0,
            }
        }

        fn step(&self, node: Point, distance: usize) -> Self {
            let mut nodes = self.nodes.clone();
            nodes.push(node);
            Self {
                nodes: nodes,
                distance: self.distance + distance,
            }
        }

        fn destination(&self) -> &Point {
            self.nodes.last().unwrap()
        }

        fn penultimate(&self) -> Option<&Point> {
            let n = self.nodes.len();
            if n > 1 {
                Some(&self.nodes[n - 2])
            } else {
                None
            }
        }
    }

    #[derive(Debug)]
    pub(crate) struct GraphPathCandidate<'m, M>
    where
        M: Graphable,
    {
        pub(crate) path: GraphPath,
        destination: &'m Point,
        graph: &'m Graph<'m, M>,
    }

    impl<'m, M> GraphPathCandidate<'m, M>
    where
        M: Graphable,
    {
        pub(crate) fn start(
            origin: Point,
            destination: &'m Point,
            graph: &'m Graph<'m, M>,
        ) -> Self {
            Self {
                path: GraphPath::new(origin),
                destination: destination,
                graph: graph,
            }
        }

        fn step(&self, node: Point, distance: usize) -> Self {
            Self {
                path: self.path.step(node, distance),
                destination: self.destination,
                graph: self.graph,
            }
        }
    }

    impl<'m, M> SearchCandidate for GraphPathCandidate<'m, M>
    where
        M: Graphable,
    {
        fn score(&self) -> usize {
            self.path.distance
        }

        fn is_complete(&self) -> bool {
            self.destination == self.path.destination()
        }

        fn children(&self) -> Vec<Self> {
            let mut paths = Vec::new();
            let node = self.path.destination();
            let backtrack = self.path.penultimate();

            for (destination, path) in self.graph.nodes.get(&node).unwrap() {
                if Some(destination) != backtrack {
                    paths.push(self.step(*destination, path.distance()))
                }
            }

            paths
        }
    }

    impl<'m, M> SearchCacher for GraphPathCandidate<'m, M>
    where
        M: Graphable,
    {
        type State = Point;

        fn state(&self) -> Self::State {
            *self.path.destination()
        }
    }
}

#[cfg(test)]
mod test {

    use super::super::map::helpers::*;
    use super::*;

    impl Graphable for SimpleMap {}

    #[test]
    fn simple() {
        let map: SimpleMap = vec![(0, 0).into()].into();

        let graph = Graph::new(&map, (0, 0).into());
        assert_eq!(graph.nodes.len(), 0);
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
        assert_eq!(graph.nodes.len(), 16);

        let path = graph.find_path((1, 1).into(), (1, 12).into()).unwrap();
        assert_eq!(path.distance(), 19);
    }
}
