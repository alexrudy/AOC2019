use anyhow::{anyhow, Error};
use geometry::coord2d::Point;
use intcode::{CPUState, Computer, Program};

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io::Read;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl TryFrom<i64> for Tile {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Tile::Empty),
            1 => Ok(Tile::Wall),
            2 => Ok(Tile::Block),
            3 => Ok(Tile::Paddle),
            4 => Ok(Tile::Ball),
            _ => Err(anyhow!("Unexpected tile value: {}", value)),
        }
    }
}

#[derive(Debug, Default)]
struct Screen {
    score: i64,
    pixels: HashMap<Point, Tile>,
}

impl Screen {
    fn score(&mut self, value: i64) {
        self.score = value;
    }

    fn add(&mut self, position: Point, tile: Tile) {
        self.pixels.insert(position, tile);
    }

    fn count(&self, tile: Tile) -> usize {
        self.pixels.values().filter(|&&v| v == tile).count()
    }
}

fn run_game(program: Program) -> Result<Screen, Error> {
    let mut command = Vec::with_capacity(3);
    let mut computer = Computer::new(program);
    let mut screen = Screen::default();

    loop {
        match computer.op()? {
            CPUState::Continue => {}
            CPUState::Output(o) => {
                command.push(o);
                if command.len() == 3 {
                    let x: i32 = command[0].try_into()?;
                    let y: i32 = command[1].try_into()?;

                    if (x, y) == (-1, 0) {
                        screen.score(command[2].try_into()?);
                    } else {
                        let tile: Tile = command[2].try_into()?;
                        screen.add((x, y).into(), tile);
                    }
                    command.clear();
                }
            }
            CPUState::Input => return Err(anyhow!("Unexpected input!")),
            CPUState::Halt => return Ok(screen),
        }
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let arcade = Program::read(input)?;

    let screen = run_game(arcade)?;

    println!("Part 1: {} block tiles", screen.count(Tile::Block));

    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::get_default_input;

    #[test]
    fn solution_day13_part1() {
        let arcade = Program::read(get_default_input(13).unwrap()).unwrap();

        let screen = run_game(arcade).unwrap();
        assert_eq!(screen.count(Tile::Block), 251);
    }
}
