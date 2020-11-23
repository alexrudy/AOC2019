//! Moudle for pathfinding in two dimensions
use std::convert::{From, Into};
use std::ops::Deref;

use super::{Direction, Point};

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
    pub fn new(origin: Point) -> Self {
        let mut steps = Vec::with_capacity(1);
        steps.push(origin);
        Path { steps }
    }

    pub fn reversed(&self) -> Self {
        let mut steps = self.steps.clone();
        steps.reverse();
        Path { steps: steps }
    }

    pub fn step(&self, direction: Direction) -> Self {
        let mut steps = self.steps.clone();
        steps.push(self.destination().step(direction));
        Path { steps: steps }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Point> {
        self.steps.iter()
    }

    pub fn origin(&self) -> &Point {
        self.steps.first().unwrap()
    }

    pub fn destination(&self) -> &Point {
        self.steps.last().unwrap()
    }

    pub fn distance(&self) -> usize {
        self.steps.len() - 1
    }

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
