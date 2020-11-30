use anyhow::Error;

use std::cmp;
use std::collections::{BTreeSet, BinaryHeap};

use geometry::coord2d::graph;
use geometry::coord2d::pathfinder;
use geometry::coord2d::Point;
use searcher::Score;
use searcher::SearchCandidate;
use searcher::SearchScore;
use searcher::SearchState;

use super::map;
use super::multi::{MultiSpelunkPath, MultiSpelunkState};

#[derive(Debug)]
struct MultiGraphs<'m> {
    map: &'m map::MultiMap,
    graph: graph::RawGraph,
}

impl<'m> MultiGraphs<'m> {
    pub(crate) fn new(map: &'m map::MultiMap) -> Self {
        use geometry::coord2d::graph::Graphable;

        let g = map.grapher(map.entrances().iter()).raw();

        Self { map, graph: g }
    }
}

#[derive(Debug, Clone)]
struct MultiGraphSpelunker<'m> {
    map: &'m map::MultiMap,
    path: MultiSpelunkPath,
    graphs: &'m MultiGraphs<'m>,
}

struct MGQ(usize, Point, pathfinder::Path);

impl cmp::Ord for MGQ {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

impl cmp::PartialOrd for MGQ {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::PartialEq for MGQ {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl cmp::Eq for MGQ {}

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
        self.path
            .path_to(robot, tile, path, destination)
            .map(|p| Self {
                map: self.map,
                path: p,
                graphs: self.graphs,
            })
    }

    fn candidates_for_robot(
        &self,
        robot: usize,
        location: &Point,
        graph: &graph::RawGraph,
    ) -> Vec<MultiGraphSpelunker<'m>> {
        let mut candidates = Vec::new();
        let mut queue = BinaryHeap::new();
        let mut seen = BTreeSet::new();

        seen.insert(location);
        queue.push(MGQ(0, *location, pathfinder::Path::new(*location)));

        while let Some(MGQ(_, origin, current_path)) = queue.pop() {
            if graph.contains(&origin) {
                for (destination, path) in graph.edges(&origin) {
                    let tile = self.map.get(*destination);

                    match tile {
                        Some(map::Tile::Key(ref key)) if !self.path.keyring().contains(key) => {
                            if let Some(c) = self.travel_to(
                                robot,
                                tile,
                                &current_path.follow(path).unwrap(),
                                path.destination(),
                            ) {
                                candidates.push(c);
                            }
                        }
                        Some(map::Tile::Door(ref key)) if !self.path.keyring().contains(key) => {}
                        _ => {
                            if seen.insert(destination) {
                                let new_path = current_path.follow(path).unwrap();
                                queue.push(MGQ(new_path.distance(), *destination, new_path));
                            }
                        }
                    }
                }
            }
        }

        candidates
    }

    fn candidates(&self) -> Vec<MultiGraphSpelunker<'m>> {
        let mut candidates = Vec::new();

        if let Some(i) = self.path.target_robot {
            candidates.extend(self.candidates_for_robot(
                i,
                &self.path.locations[i],
                &self.graphs.graph,
            ))
        } else {
            for (i, location) in self.path.locations.iter().enumerate() {
                candidates.extend(self.candidates_for_robot(i, location, &self.graphs.graph));
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

#[allow(dead_code)]
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
