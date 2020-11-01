//! Moudle for pathfinding in two dimensions
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::{From, Into};
use std::fmt;

use super::{Direction, Point};

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Path {
    steps: VecDeque<Point>,
}

impl From<Vec<Point>> for Path {
    fn from(points: Vec<Point>) -> Self {
        assert_ne!(points.len(), 0);
        Self {
            steps: points.into(),
        }
    }
}

impl Path {
    pub fn new(origin: Point) -> Self {
        let mut steps = VecDeque::with_capacity(1);
        steps.push_back(origin);
        Path { steps }
    }

    pub fn step(&self, direction: Direction) -> Self {
        let mut steps = self.steps.clone();
        steps.push_back(self.destination().step(direction));
        Path { steps: steps }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Point> {
        self.steps.iter()
    }

    pub fn origin(&self) -> &Point {
        self.steps.front().unwrap()
    }

    pub fn destination(&self) -> &Point {
        self.steps.back().unwrap()
    }

    pub fn distance(&self) -> usize {
        self.steps.len() - 1
    }
}

// Defines a map of locations
pub trait Map: Sized {
    // Can the sprite step on this location on the path?
    fn is_traversable(&self, location: Point) -> bool;

    // Build a re-usable pathfinder for this map
    fn pathfinder(&self) -> Pathfinder<Self> {
        Pathfinder::new(self)
    }

    // Build a path on this map
    fn path(&self, origin: Point, destination: Point) -> Option<Path> {
        self.pathfinder().find_path(origin, destination)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct Query {
    origin: Point,
    destiantion: Point,
}

impl From<(Point, Point)> for Query {
    fn from(points: (Point, Point)) -> Self {
        Query {
            origin: points.0,
            destiantion: points.0,
        }
    }
}

#[derive(Clone)]
pub struct Pathfinder<'m, M> {
    map: &'m M,
    path_cache: RefCell<HashMap<Query, Option<Path>>>,
}

impl<'m, M> fmt::Debug for Pathfinder<'m, M>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pathfinder")
            .field("map", &self.map)
            .finish()
    }
}

impl<'m, M> Pathfinder<'m, M>
where
    M: Map,
{
    /// Construct a new pathfinder with a pathifinding cache
    pub fn new(map: &'m M) -> Self {
        Self {
            map,
            path_cache: RefCell::from(HashMap::new()),
        }
    }

    /// Given a path candidate, compute next step potentail paths
    fn candidate_paths(&self, candidate: Path, visited: &mut HashSet<Point>) -> Vec<Path> {
        let mut paths = Vec::new();
        for direction in Direction::all() {
            let next_point = candidate.destination().step(direction);
            if visited.insert(next_point) && self.map.is_traversable(next_point) {
                paths.push(candidate.step(direction));
            }
        }

        paths
    }

    /// Compute the shortest path between two points, given all possible candidates
    fn calculate_shortest_path(&self, origin: Point, destination: Point) -> Option<Path> {
        if !self.map.is_traversable(origin) {
            return None;
        }

        let mut visited = HashSet::new();

        // Candidate cached paths.
        let mut candidates = Vec::new();

        let mut paths: VecDeque<Path> = vec![Path::new(origin)].into();

        while !paths.is_empty() {
            let candidate = paths.pop_front().unwrap();

            // This path struck a target, stop hunting
            if candidate.destination() == &destination {
                candidates.push(candidate);
            } else if candidate.distance()
                < candidates
                    .iter()
                    .map(|p| p.distance())
                    .max()
                    .unwrap_or(usize::MAX)
            {
                // Find all children of this path, if it is shorter than our current options
                paths.extend(self.candidate_paths(candidate, &mut visited).into_iter());
            }
        }

        candidates
            .into_iter()
            .min_by_key(|c| c.distance())
            .map(|c| c.into())
    }

    /// Find a path between the origin point given and an enemy.
    pub fn find_path(&self, origin: Point, destination: Point) -> Option<Path> {
        {
            match self
                .path_cache
                .borrow_mut()
                .entry((origin, destination).into())
            {
                Entry::Occupied(e) => {
                    return e.get().clone();
                }
                Entry::Vacant(e) => {
                    e.insert(self.calculate_shortest_path(origin, destination));
                }
            }
        }

        self.find_path(origin, destination)
    }

    pub(crate) fn clear(&self) {
        self.path_cache.borrow_mut().clear();
    }
}

#[cfg(test)]
mod test {

    use std::str::FromStr;

    use super::*;
    use crate::Position;

    #[derive(Debug, Default)]
    struct TestingMap {
        spaces: HashSet<Point>,
    }

    impl From<Vec<Point>> for TestingMap {
        fn from(points: Vec<Point>) -> Self {
            Self {
                spaces: points.into_iter().collect(),
            }
        }
    }

    impl FromStr for TestingMap {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut map = TestingMap::default();
            for (y, line) in s.lines().enumerate() {
                for (x, c) in line.trim().chars().enumerate() {
                    match c {
                        '.' => {
                            map.spaces.insert((x as Position, y as Position).into());
                        }
                        '#' => {}
                        _ => return Err(format!("Unexpected map character: {}", c)),
                    };
                }
            }
            Ok(map)
        }
    }

    impl Map for TestingMap {
        fn is_traversable(&self, location: Point) -> bool {
            self.spaces.contains(&location)
        }
    }

    #[test]
    fn simple() {
        let map: TestingMap = vec![(0, 0).into()].into();

        assert_eq!(
            map.path((0, 0).into(), (0, 0).into()),
            Some(vec![(0, 0).into()].into())
        );
    }

    #[test]
    fn shortest() {
        let map: TestingMap = include_str!("../../examples/pathfinding_multi.txt")
            .parse()
            .unwrap();

        let path = map.path((1, 1).into(), (1, 12).into()).unwrap();
        assert_eq!(path.distance(), 19);
    }
}
