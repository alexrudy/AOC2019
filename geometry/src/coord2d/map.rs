//! Trait to define a map suitable for pathfinding
//! on a 2D coordinate grid.
use std::fmt;

use super::path::Path;
use super::pathfinder::Pathfinder;
use super::Point;

/// Defines a map of locations on a coordinate grid.
///
/// The storage of the map is left to the implementing
/// structure, this trait simply requires a map to
/// return whether a given location is traversable.
///
/// Maps assume that traversal happens one square at
/// a time in 2-D space.
pub trait Map: Sized + fmt::Debug {
    /// Can the sprite step on this location on the path?
    fn is_traversable(&self, location: Point) -> bool;

    /// Build a re-usable pathfinder for this map
    fn pathfinder(&self) -> Pathfinder<Self> {
        Pathfinder::new(self)
    }

    /// Build a path on this map
    fn path(&self, origin: Point, destination: Point) -> Option<Path> {
        self.pathfinder().find_path(origin, destination)
    }
}

#[cfg(test)]
pub(crate) mod helpers {
    use std::collections::HashSet;
    use std::str::FromStr;

    use super::super::Point;
    use super::Map;
    use crate::Position;

    #[derive(Debug, Default, Clone)]
    pub(crate) struct SimpleMap {
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
    pub(crate) struct OpenMap {
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
}
