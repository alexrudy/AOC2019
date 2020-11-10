use anyhow::anyhow;
use anyhow::Error;
use geometry::coord2d::Point;

use std::collections::HashSet;
use std::collections::VecDeque;
use std::ops::Deref;

use super::map::{Map, Robot};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Step {
    Forward,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub(crate) struct Path(Vec<Step>);

impl Path {
    pub fn steps(&self) -> impl Iterator<Item = &Step> {
        self.0.iter()
    }
}

impl Deref for Path {
    type Target = [Step];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub(crate) struct Pathfinder<'m> {
    map: &'m Map,
    visited: HashSet<Point>,
    robot: Robot,
    queue: VecDeque<Step>,
    poisoned: bool,
}

impl<'m> Pathfinder<'m> {
    pub(crate) fn new(map: &'m Map) -> Result<Self, Error> {
        let robot = map.robot().ok_or(anyhow!("No robot present!"))?;
        let mut visited = HashSet::new();
        visited.insert(robot.location());

        Ok(Pathfinder {
            map: map,
            visited: visited,
            robot: robot,
            queue: VecDeque::new(),
            poisoned: false,
        })
    }

    pub(crate) fn path(self) -> Result<Path, Error> {
        self.collect::<Result<Vec<Step>, Error>>().map(|s| Path(s))
    }
}

impl<'m> Iterator for Pathfinder<'m> {
    type Item = Result<Step, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.poisoned {
            return None;
        }
        if let Some(step) = self.queue.pop_front() {
            return Some(Ok(step));
        }

        if self.visited.len() == self.map.len() {
            return None;
        }

        let forward = self.robot.forward();
        let left = self.robot.left();
        let right = self.robot.right();
        if self.map.contains(forward.location()) {
            self.queue.push_back(Step::Forward);
            self.visited.insert(forward.location());
            self.robot = forward;
        } else if self.map.contains(left.location()) && !self.visited.contains(&left.location()) {
            self.queue.push_back(Step::Left);
            self.queue.push_back(Step::Forward);
            self.visited.insert(left.location());
            self.robot = left;
        } else if self.map.contains(right.location()) && !self.visited.contains(&right.location()) {
            self.queue.push_back(Step::Right);
            self.queue.push_back(Step::Forward);
            self.visited.insert(right.location());
            self.robot = right;
        } else {
            self.poisoned = true;
            return Some(Err(anyhow!("No movement options remain!")));
        };

        self.queue.pop_front().map(|s| Ok(s))
    }
}
