use anyhow::{anyhow, Error};
use geometry::coord2d::{Direction, Point};
use geometry::Position;
use intcode::{CPUState, Computer, Program};

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io::Read;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Open,
    Scaffold,
    Robot,
}

impl FromStr for Tile {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "." => Ok(Tile::Open),
            "#" => Ok(Tile::Scaffold),
            "^" => Ok(Tile::Robot),
            "<" => Ok(Tile::Robot),
            ">" => Ok(Tile::Robot),
            "v" => Ok(Tile::Robot),
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
            '^' => Ok(Tile::Robot),
            '<' => Ok(Tile::Robot),
            '>' => Ok(Tile::Robot),
            'v' => Ok(Tile::Robot),
            _ => Err(anyhow!("Can't parse tile {}", value)),
        }
    }
}

#[derive(Debug, Default)]
struct Map {
    tiles: HashMap<Point, Tile>,
}

impl Map {
    fn get(&self, location: Point) -> Tile {
        self.tiles.get(&location).copied().unwrap_or(Tile::Open)
    }

    fn insert(&mut self, location: Point, tile: Tile) {
        if Tile::Open != tile {
            self.tiles.insert(location, tile);
        }
    }

    fn is_intersection(&self, location: Point) -> bool {
        Direction::all().all(|d| self.get(location.step(d)) != Tile::Open)
    }

    fn intersections(&self) -> Vec<Point> {
        self.tiles
            .keys()
            .filter(|&p| self.is_intersection(*p))
            .copied()
            .collect()
    }

    fn alignment_parameter(&self) -> Position {
        self.intersections().iter().map(|p| p.x * p.y).sum()
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

struct Camera {
    cpu: Computer,
}

impl Camera {
    fn new(program: Program) -> Self {
        Camera {
            cpu: Computer::new(program),
        }
    }

    fn capture(&mut self) -> Result<Map, Error> {
        let mut image = String::new();

        loop {
            match self.cpu.op()? {
                CPUState::Output(v) => {
                    let c: u8 = v.try_into()?;
                    image.push(c as char);
                }
                CPUState::Continue => {}
                CPUState::Input => {
                    return Err(anyhow!("Unexpected input state"));
                }
                CPUState::Halt => break,
            }
        }

        image.parse()
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let program = Program::read(input)?;

    let mut camera = Camera::new(program);
    let map = camera.capture()?;

    println!("Part 1: {}", map.alignment_parameter());

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_tile() {
        assert_eq!(".".parse::<Tile>().unwrap(), Tile::Open);
        assert_eq!("#".parse::<Tile>().unwrap(), Tile::Scaffold);
        assert_eq!("^".parse::<Tile>().unwrap(), Tile::Robot);
        assert_eq!("<".parse::<Tile>().unwrap(), Tile::Robot);
        assert_eq!(">".parse::<Tile>().unwrap(), Tile::Robot);
        assert_eq!("v".parse::<Tile>().unwrap(), Tile::Robot);
    }

    #[test]
    fn examples_part1() {
        let map: Map = include_str!("../../puzzles/17/example_a_map.txt")
            .parse()
            .unwrap();
        assert_eq!(map.get((3, 2).into()), Tile::Scaffold);
        assert_eq!(map.get((0, 0).into()), Tile::Open);

        assert_eq!(map.is_intersection((2, 2).into()), true);
        assert_eq!(map.is_intersection((3, 2).into()), false);

        assert_eq!(map.intersections().len(), 4);
        assert_eq!(map.alignment_parameter(), 76);
    }

    #[test]
    fn answer_part1() {}

    #[test]
    fn examples_part2() {}

    #[test]
    fn answer_part2() {}
}
