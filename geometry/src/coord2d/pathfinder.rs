//! Moudle for pathfinding in two dimensions
use searcher::{dijkstra, SearchCacher, SearchCandidate};

pub use super::map::Map;
pub use super::path::Path;
use super::{Direction, Point};

/// Holds information about a Path while the search
/// algorithm (in searcher) runs.
#[derive(Debug)]
struct PathCandidate<'m, M> {
    path: Path,
    map: &'m M,
    target: &'m Point,
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

    /// Find a path between the origin and destination given.
    ///
    /// When no path exists and the search is exhausted, return None.
    pub fn find_path(&self, origin: Point, destination: Point) -> Option<Path> {
        if !self.map.is_traversable(origin) {
            return None;
        }
        let start = PathCandidate::start(origin, self.map, &destination);

        dijkstra(start).ok().map(|c| c.path)
    }
}

#[cfg(test)]
mod test {

    use super::super::map::helpers::*;

    use super::*;

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
