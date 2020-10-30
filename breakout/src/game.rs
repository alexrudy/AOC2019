use anyhow::{anyhow, Error, Result};
use geometry::coord2d::{BoundingBox, Point};
use intcode::{CPUState, Computer, IntcodeError, Program};

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::default::Default;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Tile {
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

#[derive(Debug, Clone)]
pub struct Screen {
    score: i64,
    pixels: HashMap<Point, Tile>,
}

impl Screen {
    fn set_score(&mut self, value: i64) {
        self.score = value;
    }

    fn set_tile(&mut self, position: Point, tile: Tile) {
        self.pixels.insert(position, tile);
    }

    pub fn count(&self, tile: Tile) -> usize {
        self.pixels.values().filter(|&&v| v == tile).count()
    }

    pub(crate) fn bbox(&self) -> BoundingBox {
        BoundingBox::from_points(self.pixels.keys().copied())
    }

    pub(crate) fn get(&self, point: &Point) -> Option<&Tile> {
        self.pixels.get(point)
    }

    pub(crate) fn score(&self) -> i64 {
        self.score
    }
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            score: 0,
            pixels: HashMap::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum State {
    Halt,
    Input,
}

#[derive(Debug)]
pub struct Breakout {
    computer: Computer,
    screen: Screen,
}

impl Breakout {
    pub fn new(program: Program) -> Self {
        Breakout {
            computer: Computer::new(program),
            screen: Screen::default(),
        }
    }

    pub fn new_with_coins(mut program: Program) -> Self {
        program.insert(0, 2).unwrap();
        Breakout::new(program)
    }

    pub(crate) fn feed(&mut self, value: i64) {
        self.computer.feed(value);
    }

    pub(crate) fn step(&mut self) -> Result<State> {
        let mut command = Vec::with_capacity(3);
        loop {
            match self.computer.op()? {
                CPUState::Continue => {}
                CPUState::Output(o) => {
                    command.push(o);
                    if command.len() == 3 {
                        let x: i32 = command[0].try_into()?;
                        let y: i32 = command[1].try_into()?;

                        if (x, y) == (-1, 0) {
                            self.screen.set_score(command[2].try_into()?);
                        } else {
                            let tile: Tile = command[2].try_into()?;
                            self.screen.set_tile((x, y).into(), tile);
                        }
                        command.clear();
                    }
                }
                CPUState::Input => return Ok(State::Input),
                CPUState::Halt => return Ok(State::Halt),
            }
        }
    }

    pub fn next(&mut self) -> Result<&Screen> {
        self.step()?;
        Ok(&self.screen)
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
    }
}
