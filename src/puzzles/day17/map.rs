use anyhow::{anyhow, Error};
use geometry::coord2d::{Direction, Point};
use geometry::Position;

use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

use super::path::{Path, Pathfinder};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Tile {
    Open,
    Scaffold,
    Robot(Direction),
}

impl FromStr for Tile {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "." => Ok(Tile::Open),
            "#" => Ok(Tile::Scaffold),
            "^" => Ok(Tile::Robot(Direction::Up)),
            "<" => Ok(Tile::Robot(Direction::Left)),
            ">" => Ok(Tile::Robot(Direction::Right)),
            "v" => Ok(Tile::Robot(Direction::Down)),
            _ => Err(anyhow!("Can't parse tile {}", s)),
        }
    }
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Tile::Open),
            '#' => Ok(Tile::Scaffold),
            '^' => Ok(Tile::Robot(Direction::Up)),
            '<' => Ok(Tile::Robot(Direction::Left)),
            '>' => Ok(Tile::Robot(Direction::Right)),
            'v' => Ok(Tile::Robot(Direction::Down)),
            _ => Err(anyhow!("Can't parse tile {}", value)),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct Robot {
    location: Point,
    direction: Direction,
}

impl Robot {
    pub(crate) fn new(location: Point, direction: Direction) -> Self {
        Robot {
            location,
            direction,
        }
    }

    pub(crate) fn location(&self) -> Point {
        self.location
    }

    pub(crate) fn forward(&self) -> Robot {
        Robot::new(self.location.step(self.direction), self.direction)
    }

    pub(crate) fn left(&self) -> Robot {
        let direction = self.direction.turn_left();
        Robot::new(self.location.step(direction), direction)
    }

    pub(crate) fn right(&self) -> Robot {
        let direction = self.direction.turn_right();
        Robot::new(self.location.step(direction), direction)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Map {
    scaffold: HashSet<Point>,
    robot: Option<Robot>,
}

impl Map {
    pub(crate) fn get(&self, location: Point) -> Tile {
        if self.scaffold.contains(&location) {
            Tile::Scaffold
        } else {
            Tile::Open
        }
    }

    pub(crate) fn contains(&self, location: Point) -> bool {
        self.scaffold.contains(&location)
    }

    pub(crate) fn insert(&mut self, location: Point, tile: Tile) {
        match tile {
            Tile::Open => {}
            Tile::Scaffold => {
                self.scaffold.insert(location);
            }
            Tile::Robot(direction) => {
                self.robot = Some(Robot::new(location, direction));
                self.scaffold.insert(location);
            }
        }
    }

    pub(crate) fn is_intersection(&self, location: Point) -> bool {
        Direction::all().all(|d| self.get(location.step(d)) != Tile::Open)
    }

    pub(crate) fn intersections(&self) -> Vec<Point> {
        self.scaffold
            .iter()
            .filter(|&p| self.is_intersection(*p))
            .copied()
            .collect()
    }

    pub(crate) fn alignment_parameter(&self) -> Position {
        self.intersections().iter().map(|p| p.x * p.y).sum()
    }

    pub(crate) fn pathfinder(&self) -> Result<Pathfinder, Error> {
        Pathfinder::new(self)
    }

    pub(crate) fn path(&self) -> Result<Path, Error> {
        self.pathfinder()?.path()
    }

    pub(crate) fn robot(&self) -> Option<Robot> {
        self.robot
    }

    pub(crate) fn len(&self) -> usize {
        self.scaffold.len()
    }
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = Map::default();

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let point = (x, y).into();

                map.insert(point, c.try_into()?);
            }
        }

        Ok(map)
    }
}
