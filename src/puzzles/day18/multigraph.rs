use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::Error;

use geometry::coord2d::graph;
use geometry::coord2d::map::Map;
use geometry::coord2d::pathfinder;
use geometry::coord2d::Point;
use searcher::graph::{GraphPath, Graphable};
use searcher::Score;
use searcher::SearchCandidate;
use searcher::SearchScore;
use searcher::SearchState;

use super::map;
use super::multi::{MultiSpelunkPath, MultiSpelunkState};

#[derive(Debug, Clone)]
struct KeyGraphable<'m> {
    map: &'m map::MultiMap,
    base: &'m graph::RawGraph,
    keys: &'m map::KeyRing,
    location: Point,
}

impl<'m> KeyGraphable<'m> {
    fn new(
        map: &'m map::MultiMap,
        base: &'m graph::RawGraph,
        keys: &'m map::KeyRing,
        location: Point,
    ) -> Self {
        KeyGraphable {
            map,
            base,
            keys,
            location,
        }
    }
}

impl<'m> Graphable for KeyGraphable<'m> {
    type Edge = GraphPath<Point, graph::GPath>;

    fn is_node(&self, node: &Point) -> bool {
        if *node == self.location {
            return true;
        }
        match self.map.get(*node) {
            Some(map::Tile::Key(_)) => true,
            Some(map::Tile::Entrance) => true,
            _ => false,
        }
    }

    fn neighbors(&self, node: &Point) -> Vec<(Point, GraphPath<Point, graph::GPath>)> {
        let here = GraphPath::new(*node);
        self.base
            .edges(node)
            .filter(|(&d, _)| self.is_traversable(d))
            .map(|(d, p)| {
                let gp: graph::GPath = p.clone().into();
                (*d, here.step_one(*d, gp))
            })
            .collect()
    }
}

impl<'m> Map for KeyGraphable<'m> {
    fn is_traversable(&self, location: Point) -> bool {
        match self.map.get(location) {
            Some(map::Tile::Door(ref c)) if !self.keys.contains(c) => false,
            Some(_) => true,
            None => false,
        }
    }
}

impl<'m> graph::Graphable for KeyGraphable<'m> {
    fn is_node(&self, point: &Point) -> bool {
        if *point == self.location {
            return true;
        }
        match self.map.get(*point) {
            Some(map::Tile::Key(_)) => true,
            Some(map::Tile::Entrance) => true,
            _ => false,
        }
    }
}

type MultiGraph = graph::RawGraph;

#[derive(Debug)]
struct MultiGraphs<'m> {
    map: &'m map::MultiMap,
    basegraph: graph::RawGraph,
    graphs: RefCell<HashMap<map::KeyRing, Rc<MultiGraph>>>,
}

impl<'m> MultiGraphs<'m> {
    pub(crate) fn graph(&'m self, keys: &map::KeyRing, origin: Point) -> Rc<MultiGraph> {
        {
            let cache = self.graphs.borrow();
            if let Some(g) = cache.get(keys) {
                return g.clone();
            }
        }

        {
            use geometry::coord2d::graph::Graphable;

            let kg = KeyGraphable::new(self.map, &self.basegraph, keys, origin);

            let rg = kg.grapher(self.map.entrances().iter()).raw();
            let mut gs = self.graphs.borrow_mut();
            gs.insert(keys.clone(), Rc::new(rg));
            if gs.len() % 100 == 0 {
                eprintln!("G{}", gs.len());
            }
        }

        self.graph(keys, origin)
    }

    pub(crate) fn new(map: &'m map::MultiMap) -> Self {
        use geometry::coord2d::graph::Graphable;

        let g = map.grapher(map.entrances().iter()).raw();

        Self {
            map,
            basegraph: g,
            graphs: RefCell::new(HashMap::new()),
        }
    }
}

#[derive(Debug, Clone)]
struct MultiGraphSpelunker<'m> {
    map: &'m map::MultiMap,
    path: MultiSpelunkPath,
    graphs: &'m MultiGraphs<'m>,
}

impl<'m> MultiGraphSpelunker<'m> {
    fn new(map: &'m map::MultiMap, graphs: &'m MultiGraphs<'m>, origins: [Point; 4]) -> Self {
        Self {
            map,
            graphs,
            path: MultiSpelunkPath::start(origins),
        }
    }

    fn travel_to(
        &self,
        robot: usize,
        tile: Option<map::Tile>,
        path: &pathfinder::Path,
        destination: &Point,
    ) -> Option<Self> {
        self.path.path_to(robot, tile, path, destination).map(|p| {
            let mut s = self.clone();
            s.path = p;
            s
        })
    }

    fn candidates_for_robot(
        &self,
        robot: usize,
        location: &Point,
        graph: &graph::RawGraph,
    ) -> Vec<MultiGraphSpelunker<'m>> {
        let mut candidates = Vec::new();

        if graph.contains(location) {
            for (point, path) in graph.edges(location) {
                if let Some(c) = self.travel_to(robot, self.map.get(*point), path, point) {
                    candidates.push(c);
                }
            }
        }

        candidates
    }

    fn candidates(&self) -> Vec<MultiGraphSpelunker<'m>> {
        let mut candidates = Vec::new();

        if let Some(i) = self.path.target_robot {
            candidates.extend(
                self.candidates_for_robot(
                    i,
                    &self.path.locations[i],
                    &self
                        .graphs
                        .graph(self.path.keyring(), self.path.locations[i]),
                ),
            )
        } else {
            for (i, location) in self.path.locations.iter().enumerate() {
                candidates.extend(self.candidates_for_robot(
                    i,
                    location,
                    &self.graphs.graph(self.path.keyring(), *location),
                ));
            }
        }

        candidates
    }

    pub(crate) fn distance(&self) -> usize {
        self.path.distance()
    }
}

impl<'m> SearchScore for MultiGraphSpelunker<'m> {
    type Score = usize;
    fn score(&self) -> Self::Score {
        self.distance()
    }
}

impl<'m> SearchCandidate for MultiGraphSpelunker<'m> {
    fn is_complete(&self) -> bool {
        self.path.keyring().len() == self.map.n_keys()
    }

    fn children(&self) -> Vec<Self> {
        self.candidates()
    }
}

impl<'m> SearchState for MultiGraphSpelunker<'m> {
    type State = MultiSpelunkState;

    fn state(&self) -> MultiSpelunkState {
        MultiSpelunkState::new(self.path.keyring().clone(), self.path.locations)
    }
}

pub(crate) fn search<'m>(map: &'m map::MultiMap) -> Result<MultiSpelunkPath, Error> {
    use searcher::SearchOptions;

    {
        let graphs = MultiGraphs::new(map);
        let entrances = map.entrances();

        let origin: Score<MultiGraphSpelunker> =
            MultiGraphSpelunker::new(map, &graphs, entrances.clone()).into();

        let options = {
            let mut o = SearchOptions::default();
            o.verbose = Some(10_000);
            o.exhaustive = true;
            o
        };

        Ok(searcher::dijkstra::build(origin)
            .with_options(options)
            .run()
            .map(|c| c.unwrap().path)?)
    }
}
