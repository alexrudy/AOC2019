use anyhow::{anyhow, Error};
use geometry::coord2d::graph;
use geometry::coord2d::pathfinder;
use geometry::coord2d::Point;
use searcher::{self, Score, SearchCandidate, SearchScore, SearchState};

use std::cmp::{Eq, PartialEq};

use super::map;
use super::KeyPath;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct MultiSpelunkState(map::KeyRing, [Point; 4]);

impl MultiSpelunkState {
    pub(crate) fn new(keys: map::KeyRing, robots: [Point; 4]) -> Self {
        MultiSpelunkState(keys, robots)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MultiSpelunkPath {
    /// Full set of keys collected.
    keys: map::KeyRing,
    /// Order of keys collected
    path: Vec<char>,

    /// Current node for each robot.
    pub(crate) locations: [Point; 4],
    /// Total distance traveled.
    distance: usize,
    /// Target robot to move next. If None, try all robots.
    pub(crate) target_robot: Option<usize>,
}

impl MultiSpelunkPath {
    pub(crate) fn start(origins: [Point; 4]) -> Self {
        Self {
            keys: map::KeyRing::default(),
            path: Vec::new(),
            locations: origins,
            distance: 0,
            target_robot: None,
        }
    }

    pub(crate) fn found_key(&mut self, key: char) {
        if self.keys.insert(key) {
            self.path.push(key);
        }
    }

    pub(crate) fn path_to(
        &self,
        robot: usize,
        tile: Option<map::Tile>,
        gp: &pathfinder::Path,
        destination: &Point,
    ) -> Option<Self> {
        let mut path = match tile {
            Some(map::Tile::Key(ref c)) if !self.keys.contains(c) => {
                let mut path = self.clone();
                path.found_key(*c);
                path.target_robot = None;
                path
            }
            Some(map::Tile::Door(ref c)) if !self.keys.contains(c) => {
                return None;
            }
            None => {
                return None;
            }
            _ => {
                let mut p = self.clone();
                p.target_robot = Some(robot);
                p
            }
        };

        path.locations[robot] = *destination;
        path.distance += gp.distance();

        Some(path)
    }

    pub(crate) fn distance(&self) -> usize {
        self.distance
    }

    pub(crate) fn keyring(&self) -> &map::KeyRing {
        &self.keys
    }

    pub(crate) fn keys(&self) -> KeyPath {
        KeyPath(self.path.clone())
    }
}

#[derive(Debug, Clone)]
struct MultiSpelunker<'m> {
    map: &'m map::MultiMap,
    graphs: [&'m graph::Graph<'m, map::MultiMap>; 4],
    path: MultiSpelunkPath,
}

impl<'m> SearchCandidate for MultiSpelunker<'m> {
    fn is_complete(&self) -> bool {
        self.path.keys.len() == self.map.n_keys()
    }

    fn children(&self) -> Vec<Self> {
        self.candidates()
    }
}

impl<'m> SearchScore for MultiSpelunker<'m> {
    type Score = usize;

    fn score(&self) -> usize {
        self.distance()
    }
}

impl<'m> SearchState for MultiSpelunker<'m> {
    type State = MultiSpelunkState;

    fn state(&self) -> MultiSpelunkState {
        MultiSpelunkState(self.path.keys.clone(), self.path.locations)
    }
}

impl<'m> MultiSpelunker<'m> {
    fn new(
        map: &'m map::MultiMap,
        graphs: [&'m graph::Graph<'m, map::MultiMap>; 4],
        origins: [Point; 4],
    ) -> Self {
        Self {
            map,
            graphs: graphs,
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
        self.path
            .path_to(robot, tile, path, destination)
            .map(|p| MultiSpelunker {
                path: p,
                map: self.map,
                graphs: self.graphs,
            })
    }

    fn candidates_for_robot(
        &self,
        robot: usize,
        location: &Point,
        graph: &'m graph::Graph<'m, map::MultiMap>,
    ) -> Vec<MultiSpelunker<'m>> {
        let mut candidates = Vec::new();

        for (point, path) in graph.edges(*location) {
            if let Some(c) = self.travel_to(robot, self.map.get(*point), path, point) {
                candidates.push(c);
            }
        }

        candidates
    }

    fn candidates(&self) -> Vec<MultiSpelunker<'m>> {
        let mut candidates = Vec::new();

        if let Some(i) = self.path.target_robot {
            candidates.extend(self.candidates_for_robot(i, &self.path.locations[i], self.graphs[i]))
        } else {
            for (i, (location, graph)) in self
                .path
                .locations
                .iter()
                .zip(self.graphs.iter())
                .enumerate()
            {
                candidates.extend(self.candidates_for_robot(i, location, graph));
            }
        }

        candidates
    }

    pub(crate) fn distance(&self) -> usize {
        self.path.distance
    }
}

#[allow(dead_code)]
pub(crate) fn search<'m>(map: &'m map::MultiMap) -> Result<MultiSpelunkPath, Error> {
    use geometry::coord2d::graph::Graphable;
    use searcher::SearchOptions;

    use std::convert::TryInto;

    let entrances = map.entrances();

    let graphs: Vec<_> = entrances.iter().map(|e| map.graph(*e)).collect();

    {
        let grefs: [&graph::Graph<map::MultiMap>; 4] = graphs
            .iter()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| anyhow!("Can't form graph ref"))?;

        let origin: Score<MultiSpelunker> =
            MultiSpelunker::new(map, grefs, entrances.clone()).into();

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
}
