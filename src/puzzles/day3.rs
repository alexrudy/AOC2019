use anyhow::{anyhow, Error};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;
use thiserror::Error;

use crate::cartesian::{Direction, Point};

#[derive(Error, Debug)]
pub(crate) enum WirePathParsingError {
    #[error("Malformed Path: {0}")]
    MalformedPath(String),

    #[error("Invalid direction {0}")]
    InvalidDirection(String),

    #[error("Invalid distance: Parsing Error")]
    InvalidDistance(#[from] std::num::ParseIntError),

    #[error("Invalid distance: {0}")]
    NegativeDistance(i32),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct WirePathElement {
    distance: i32,
    direction: Direction,
}

impl FromStr for WirePathElement {
    type Err = WirePathParsingError;

    fn from_str(element: &str) -> Result<WirePathElement, WirePathParsingError> {
        if element.len() < 2 {
            return Err(WirePathParsingError::MalformedPath(element.to_owned()));
        }

        let direction = {
            let d = element
                .chars()
                .nth(0)
                .ok_or(WirePathParsingError::MalformedPath(element.to_owned()))?;
            match d {
                'U' => Ok(Direction::Up),
                'D' => Ok(Direction::Down),
                'L' => Ok(Direction::Left),
                'R' => Ok(Direction::Right),
                _ => Err(WirePathParsingError::InvalidDirection(d.to_string())),
            }
        }?;
        let distance = element[1..].parse::<i32>()?;
        if distance < 0 {
            return Err(WirePathParsingError::NegativeDistance(distance));
        }

        Ok(WirePathElement {
            distance: distance,
            direction: direction,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
struct WirePath(Vec<WirePathElement>);

impl FromStr for WirePath {
    type Err = WirePathParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s
            .split(',')
            .map(|element| element.parse::<WirePathElement>())
            .collect::<Result<Vec<WirePathElement>, WirePathParsingError>>()?;
        Ok(WirePath(elements))
    }
}

impl<'a> WirePath {
    pub fn iter(&'a self) -> WirePathIterator<'a> {
        WirePathIterator {
            path: &self,
            step: 0,
            distance: 0,
            position: Some(Point::origin()),
        }
    }
}

#[derive(Debug)]
struct WirePathIterator<'a> {
    path: &'a WirePath,
    step: usize,
    distance: i32,
    position: Option<Point>,
}

impl<'a> Iterator for WirePathIterator<'a> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step >= self.path.0.len() {
            return self.position.take();
        }

        {
            let step = &self.path.0[self.step];
            if self.distance == step.distance {
                // We are at the end of this step, advance to the next one.
                self.step += 1;
                self.distance = 0;
            }
        }
        if self.step >= self.path.0.len() {
            return self.position.take();
        }

        {
            let step = &self.path.0[self.step];
            if self.distance < step.distance {
                self.distance += 1;
                let next_position = self.position.map(|p| p.step(&step.direction));
                std::mem::replace(&mut self.position, next_position)
            } else {
                unreachable!("Exceeded step!")
            }
        }
    }
}

#[derive(Debug, Default)]
struct Breadboard {
    // Maps positions to counts of wires
    wires: usize,
    pegs: HashMap<Point, HashMap<usize, i32>>,
}

impl Breadboard {
    fn add_wire(&mut self, wire: &WirePath) -> () {
        let wire_number = self.wires + 1;
        for (steps, point) in wire.iter().enumerate() {
            let wires = self.pegs.entry(point).or_insert(HashMap::new());
            (*wires).entry(wire_number).or_insert(steps as i32);
        }
        self.wires = wire_number;
    }

    fn collisions(&self) -> HashSet<Point> {
        self.pegs
            .iter()
            .filter(|&(&point, wires)| (wires.len() > 1 && point != Point::origin()))
            .map(|(&point, _)| point)
            .collect::<HashSet<_>>()
    }

    fn earliest_collision(&self) -> Option<(Point, i32)> {
        self.pegs
            .iter()
            .filter(|&(&point, wires)| (wires.len() > 1 && point != Point::origin()))
            .map(|(&point, wires)| (point, wires.values().sum()))
            .min_by_key(|&(_, delay)| delay)
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let mut breadboard = Breadboard::default();
    let reader = BufReader::new(input);

    for line in reader.lines() {
        let path: WirePath = line?.parse()?;
        breadboard.add_wire(&path)
    }

    let closest_collision = breadboard
        .collisions()
        .into_iter()
        .min_by_key(|&point| point.manhattan(&Point::origin()))
        .ok_or(anyhow!("No collisions found!"))?;

    println!(
        "Part 1: Closest Collision is at {}, distance = {}",
        closest_collision,
        closest_collision.manhattan(&Point::origin())
    );

    let (point, delay) = breadboard
        .earliest_collision()
        .ok_or(anyhow!("No collision found"))?;
    println!(
        "Part 2: Earliest collision is at {} after {} total steps",
        point, delay
    );

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_path() {
        let path: WirePath = "U4,D3".parse().unwrap();

        assert_eq!(
            path,
            WirePath(vec![
                WirePathElement {
                    distance: 4,
                    direction: Direction::Up
                },
                WirePathElement {
                    distance: 3,
                    direction: Direction::Down
                }
            ])
        )
    }

    #[test]
    fn walk_path() {
        let path: WirePath = "U2,L1".parse().unwrap();

        let steps = path.iter().take(5).collect::<Vec<Point>>();

        assert_eq!(
            steps,
            vec![
                Point { x: 0, y: 0 },
                Point { x: 0, y: 1 },
                Point { x: 0, y: 2 },
                Point { x: -1, y: 2 }
            ]
        )
    }

    #[test]
    fn examples_part1() {
        let mut breadboard = Breadboard::default();
        let wire1: WirePath = "R8,U5,L5,D3".parse().unwrap();
        let wire2: WirePath = "U7,R6,D4,L4".parse().unwrap();

        breadboard.add_wire(&wire1);
        breadboard.add_wire(&wire2);

        assert_eq!(
            breadboard.collisions(),
            vec![Point { x: 3, y: 3 }, Point { x: 6, y: 5 }]
                .into_iter()
                .collect::<HashSet<Point>>()
        );

        assert_eq!(
            example_case_part1(
                "R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83"
            ),
            159
        );

        assert_eq!(
            example_case_part1(
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7"
            ),
            135
        );
    }

    fn example_case_part1(input: &str) -> i32 {
        let mut breadboard = Breadboard::default();
        for line in input.lines() {
            let wire: WirePath = line.parse().unwrap();
            breadboard.add_wire(&wire);
        }
        dbg!(breadboard.collisions());
        breadboard
            .collisions()
            .into_iter()
            .min_by_key(|&point| point.manhattan(&Point::origin()))
            .expect("No collisions found!")
            .manhattan(&Point::origin())
    }

    fn example_case_part2(input: &str) -> i32 {
        let mut breadboard = Breadboard::default();
        for line in input.lines() {
            let wire: WirePath = line.parse().unwrap();
            breadboard.add_wire(&wire);
        }
        let (_, delay) = breadboard.earliest_collision().unwrap();
        delay
    }

    #[test]
    fn examples_part2() {
        assert_eq!(example_case_part2("R8,U5,L5,D3\nU7,R6,D4,L4"), 30);
        assert_eq!(
            example_case_part2(
                "R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83"
            ),
            610
        );

        assert_eq!(
            example_case_part2(
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7"
            ),
            410
        );
    }
}
