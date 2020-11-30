//! Path data structures
//!
//! A path is a sequence of steps in a 2D geometry.

use std::convert::{From, Into};
use std::ops::Deref;

use thiserror::Error;

use super::{Direction, Point};

/// Error returned for invalid paths
#[derive(Debug, Error)]
pub enum PathError {
    /// A new step added to this path was not adjecent to the
    /// previous step.
    #[error("{0} is not adjacent to the end of the path {1}")]
    NotAdjacentSequence(Point, Point),
}

type PathResult<T> = Result<T, PathError>;

/// A sequence of steps in a 2D geometry.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Path {
    steps: Vec<Point>,
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
    /// Construct a new path which starts from this point.
    pub fn new(origin: Point) -> Self {
        let mut steps = Vec::with_capacity(1);
        steps.push(origin);
        Path { steps }
    }

    /// Return a copy of this path, but reversed.
    pub fn reversed(&self) -> Self {
        let mut steps = self.steps.clone();
        steps.reverse();
        Path { steps: steps }
    }

    /// Return a new path after taking a step in a particular direction.
    pub fn step(&self, direction: Direction) -> Self {
        let mut steps = self.steps.clone();
        steps.push(self.destination().step(direction));
        Path { steps: steps }
    }

    /// Return a new path after stepping to a particular point.
    pub fn step_to(&self, point: Point) -> PathResult<Self> {
        let mut steps = self.steps.clone();
        if !point.is_adjacent(self.destination()) {
            return Err(PathError::NotAdjacentSequence(point, *self.destination()));
        }

        steps.push(point);
        Ok(Path { steps: steps })
    }

    /// Iterate over the points in this path.
    pub fn iter(&self) -> impl Iterator<Item = &Point> {
        self.steps.iter()
    }

    /// Where this path started
    pub fn origin(&self) -> &Point {
        self.steps.first().unwrap()
    }

    /// Where this path ends
    pub fn destination(&self) -> &Point {
        self.steps.last().unwrap()
    }

    /// How long this path is.
    pub fn distance(&self) -> usize {
        self.steps.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.steps.len() < 2
    }

    /// What is the last direction in this path?
    pub fn last_direction(&self) -> Option<Direction> {
        let n = self.steps.len();
        if self.steps.len() < 2 {
            return None;
        }

        let last = self.steps[n - 1];
        let penultimate = self.steps[n - 2];

        penultimate.direction(last)
    }
}

impl Deref for Path {
    type Target = [Point];

    fn deref(&self) -> &Self::Target {
        &self.steps
    }
}
