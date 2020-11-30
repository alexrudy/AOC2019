use std::cell::Cell;
use std::cmp::{Eq, PartialEq};

use anyhow::{anyhow, Error};

use geometry::coord2d::graph;
use geometry::coord2d::pathfinder;
use geometry::coord2d::Point;
use searcher::{self, Score, SearchCandidate, SearchHeuristic, SearchScore, SearchState};

use super::map;
use super::KeyPath;

#[derive(Debug)]
struct NoDoorMap<'m>(&'m map::Map);

impl<'m> pathfinder::Map for NoDoorMap<'m> {
    fn is_traversable(&self, location: Point) -> bool {
        self.0.get(location).is_some()
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct SpelunkState(map::KeyRing, Point);

#[derive(Debug, Clone)]
pub(crate) struct SpelunkPath {
    keys: map::KeyRing,
    path: Vec<char>,
    location: Point,
    distance: usize,
}

impl SpelunkPath {
    fn start(origin: Point) -> Self {
        Self {
            keys: map::KeyRing::default(),
            path: Vec::new(),
            location: origin,
            distance: 0,
        }
    }

    fn found_key(&mut self, key: char) {
        if self.keys.insert(key) {
            self.path.push(key);
        }
    }

    pub(crate) fn distance(&self) -> usize {
        self.distance
    }

    pub(crate) fn keys(&self) -> KeyPath {
        KeyPath(self.path.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Spelunker<'m> {
    map: &'m map::Map,
    graph: &'m graph::Graph<'m, map::Map>,
    path: SpelunkPath,
    heuristic: Cell<Option<usize>>,
}

impl<'m> SearchCandidate for Spelunker<'m> {
    fn is_complete(&self) -> bool {
        self.path.keys.len() == self.map.n_keys()
    }

    fn children(&self) -> Vec<Self> {
        self.candidates().unwrap()
    }
}

impl<'m> SearchScore for Spelunker<'m> {
    type Score = usize;
    fn score(&self) -> Self::Score {
        self.distance()
    }
}

impl<'m> SearchState for Spelunker<'m> {
    type State = SpelunkState;

    fn state(&self) -> SpelunkState {
        SpelunkState(self.path.keys.clone(), self.location().unwrap())
    }
}

impl<'m> SearchHeuristic for Spelunker<'m> {
    type Hueristic = usize;

    fn heuristic(&self) -> usize {
        if let Some(h) = self.heuristic.get() {
            return h;
        }

        use pathfinder::Map;
        let mut here = self.location().unwrap();
        let mut h = 0;

        for key in self.map.keys() {
            if self.path.keys.contains(&key.door) {
                continue;
            }

            let p = NoDoorMap(self.map).path(here, key.location).unwrap();
            h += p.distance();
            here = *p.destination();
        }

        let total_heuristic = h + self.distance();

        self.heuristic.set(Some(total_heuristic));
        total_heuristic
    }
}

impl<'m> Spelunker<'m> {
    fn new(map: &'m map::Map, graph: &'m graph::Graph<'m, map::Map>) -> Self {
        Self {
            map,
            graph: graph,
            path: SpelunkPath::start(map.entrance().unwrap()),
            heuristic: Cell::new(None),
        }
    }

    fn location(&self) -> Result<Point, Error> {
        Ok(self.path.location)
    }

    fn candidates(&self) -> Result<Vec<Spelunker<'m>>, Error> {
        let mut candidates = Vec::with_capacity(4);

        for (point, path) in self.graph.edges(&self.location()?) {
            match self.map.get(*point) {
                Some(map::Tile::Key(c)) => {
                    let mut newsp = self.clone();
                    newsp.path.found_key(c);
                    newsp.path.location = *point;
                    newsp.path.distance += path.distance();
                    candidates.push(newsp);
                }
                Some(map::Tile::Door(c)) if self.path.keys.contains(&c) => {
                    let mut newsp = self.clone();
                    newsp.path.location = *point;
                    newsp.path.distance += path.distance();
                    candidates.push(newsp);
                }
                Some(map::Tile::Entrance) => {
                    let mut newsp = self.clone();
                    newsp.path.location = *point;
                    newsp.path.distance += path.distance();
                    candidates.push(newsp);
                }
                Some(map::Tile::Door(_)) => {}
                Some(map::Tile::Hall) => {
                    let mut newsp = self.clone();
                    newsp.path.location = *point;
                    newsp.path.distance += path.distance();
                    candidates.push(newsp);
                }
                None => {}
            }
        }

        Ok(candidates)
    }

    pub(crate) fn distance(&self) -> usize {
        self.path.distance
    }
}

pub(crate) fn search<'m>(map: &'m map::Map) -> Result<SpelunkPath, Error> {
    use geometry::coord2d::graph::Graphable;
    use searcher::SearchOptions;

    let graph = map.graph(map.entrance().ok_or(anyhow!("No entrance?"))?);
    let origin: Score<Spelunker> = Spelunker::new(map, &graph).into();

    let options = {
        let mut o = SearchOptions::default();
        o.verbose = Some(10_000);
        o
    };

    Ok(searcher::dijkstra::build(origin)
        .with_options(options)
        .run()
        .map(|c| c.unwrap().path)?)
}
