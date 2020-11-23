//! Moudle for pathfinding in two dimensions
use std::clone::Clone;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::convert::From;
use std::fmt;

use searcher::{djirkstra, SearchCacher, SearchCandidate};

pub use super::path::Path;
use super::{Direction, Point};

// Defines a map of locations
pub trait Map: Sized + fmt::Debug {
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

/// Holds information about a Path while the search
/// algorithm (in searcher) runs.
#[derive(Debug)]
struct PathCandidate<'m, M> {
    path: Path,
    map: &'m M,
    target: &'m Point,
}

impl<'m, M> Clone for PathCandidate<'m, M> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            map: self.map,
            target: self.target,
        }
    }
}

impl<'m, M> PathCandidate<'m, M>
where
    M: Map,
{
    fn start(origin: Point, map: &'m M, target: &'m Point) -> Self {
        Self {
            path: Path::new(origin),
            map: map,
            target: target,
        }
    }

    fn step(&self, direction: Direction) -> Self {
        Self {
            path: self.path.step(direction),
            map: self.map,
            target: self.target,
        }
    }
}

impl<'m, M> Ord for PathCandidate<'m, M>
where
    M: Map,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.distance().cmp(&other.path.distance()).reverse()
    }
}

impl<'m, M> PartialEq for PathCandidate<'m, M> {
    fn eq(&self, other: &Self) -> bool {
        self.path.distance().eq(&other.path.distance())
    }
}

impl<'m, M> Eq for PathCandidate<'m, M> {}

impl<'m, M> PartialOrd for PathCandidate<'m, M>
where
    M: Map,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'m, M> SearchCandidate for PathCandidate<'m, M>
where
    M: Map,
{
    fn score(&self) -> usize {
        self.path.distance()
    }

    fn is_complete(&self) -> bool {
        self.path.destination() == self.target
    }

    fn children(&self) -> Vec<Self> {
        let mut paths = Vec::new();
        let destination = self.path.destination();
        for direction in Direction::all() {
            let next_point = destination.step(direction);
            if self.map.is_traversable(next_point)
                && Some(direction.reverse()) != self.path.last_direction()
            {
                paths.push(self.step(direction));
            }
        }

        paths
    }
}

impl<'m, M> SearchCacher for PathCandidate<'m, M>
where
    M: Map,
{
    type State = Point;

    fn state(&self) -> Self::State {
        *self.path.destination()
    }
}

#[derive(Debug, Clone)]
pub struct Pathfinder<'m, M> {
    map: &'m M,
}

impl<'m, M> Pathfinder<'m, M>
where
    M: Map,
{
    /// Construct a new pathfinder with a pathifinding cache
    pub fn new(map: &'m M) -> Self {
        Self { map }
    }

    /// Find a path between the origin point given and an enemy.
    pub fn find_path(&self, origin: Point, destination: Point) -> Option<Path> {
        if !self.map.is_traversable(origin) {
            return None;
        }
        let start = PathCandidate::start(origin, self.map, &destination);

        djirkstra(start).run().ok().map(|c| c.path)
    }
}

#[cfg(test)]
mod test {

    use std::collections::HashSet;
    use std::str::FromStr;

    use super::*;
    use crate::Position;

    #[derive(Debug, Default, Clone)]
    struct SimpleMap {
        spaces: HashSet<Point>,
    }

    impl From<Vec<Point>> for SimpleMap {
        fn from(points: Vec<Point>) -> Self {
            Self {
                spaces: points.into_iter().collect(),
            }
        }
    }

    impl FromStr for SimpleMap {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut map = SimpleMap::default();
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

    impl Map for SimpleMap {
        fn is_traversable(&self, location: Point) -> bool {
            self.spaces.contains(&location)
        }
    }

    #[derive(Debug, Default, Clone)]
    struct OpenMap {
        walls: HashSet<Point>,
    }

    impl From<Vec<Point>> for OpenMap {
        fn from(points: Vec<Point>) -> Self {
            Self {
                walls: points.into_iter().collect(),
            }
        }
    }

    impl FromStr for OpenMap {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut map = OpenMap::default();
            for (y, line) in s.lines().enumerate() {
                for (x, c) in line.trim().chars().enumerate() {
                    match c {
                        '.' => {}
                        '#' => {
                            map.walls.insert((x as Position, y as Position).into());
                        }
                        _ => return Err(format!("Unexpected map character: {}", c)),
                    };
                }
            }
            Ok(map)
        }
    }

    impl Map for OpenMap {
        fn is_traversable(&self, location: Point) -> bool {
            !self.walls.contains(&location)
        }
    }

    #[test]
    fn simple() {
        let map: SimpleMap = vec![(0, 0).into()].into();

        assert_eq!(
            map.path((0, 0).into(), (0, 0).into()),
            Some(vec![(0, 0).into()].into())
        );
    }

    #[test]
    fn shortest() {
        let map: SimpleMap = include_str!("../../examples/pathfinding_multi.txt")
            .parse()
            .unwrap();

        let path = map.path((1, 1).into(), (1, 12).into()).unwrap();
        assert_eq!(path.distance(), 19);
    }

    #[test]
    fn openmap() {
        let map: OpenMap = include_str!("../../examples/pathfinding_island.txt")
            .parse()
            .unwrap();

        let path = map.path((0, 0).into(), (2, 2).into()).unwrap();
        assert_eq!(path.distance(), 10);
    }
}
