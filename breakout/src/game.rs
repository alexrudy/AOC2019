use anyhow::{anyhow, Error, Result};
use geometry::coord2d::{BoundingBox, Point};
use intcode::{CPUState, Computer, Program};

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::default::Default;
use std::fmt::Debug;

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
    paddle: Option<Point>,
    ball: Option<Point>,
}

impl Screen {
    fn set_score(&mut self, value: i64) {
        self.score = value;
    }

    fn set_tile(&mut self, position: Point, tile: Tile) {
        self.pixels.insert(position, tile);
        match tile {
            Tile::Paddle => {
                self.paddle = Some(position);
            }
            Tile::Ball => {
                self.ball = Some(position);
            }
            _ => {}
        }
    }

    pub fn paddle(&self) -> Option<Point> {
        self.paddle
    }

    pub fn ball(&self) -> Option<Point> {
        self.ball
    }

    pub fn count(&self, tile: Tile) -> usize {
        self.pixels.values().filter(|&&v| v == tile).count()
    }

    pub(crate) fn bbox(&self) -> BoundingBox {
        BoundingBox::from_points(self.pixels.keys())
    }

    pub(crate) fn get(&self, point: &Point) -> Option<&Tile> {
        self.pixels.get(point)
    }

    pub fn score(&self) -> i64 {
        self.score
    }
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            score: 0,
            pixels: HashMap::default(),
            paddle: None,
            ball: None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Joystick {
    Left,
    Neutral,
    Right,
}

pub trait Controller: Debug {
    fn control(&self, screen: &Screen) -> Joystick;
}

type BoxedController = Box<dyn Controller + Send + 'static>;

/// Simple controller provides a basic version of AI which
/// can follow the ball around.
#[derive(Debug)]
pub struct SimpleController {}

impl SimpleController {
    pub fn new() -> Self {
        SimpleController {}
    }
}

impl Controller for SimpleController {
    fn control(&self, screen: &Screen) -> Joystick {
        let pos = screen
            .paddle()
            .and_then(|p| screen.ball().map(|b| (b.x - p.x).signum()));

        match pos {
            Some(-1) => Joystick::Left,
            Some(0) => Joystick::Neutral,
            Some(1) => Joystick::Right,
            Some(_) => panic!("Invalid value!"),
            None => Joystick::Neutral,
        }
    }
}

#[derive(Debug)]
pub struct NeutralController {}

impl NeutralController {
    pub fn new() -> Self {
        NeutralController {}
    }
}

impl Controller for NeutralController {
    fn control(&self, _: &Screen) -> Joystick {
        Joystick::Neutral
    }
}

#[derive(Debug)]
pub(crate) enum State {
    Halt,
    Step,
}

#[derive(Debug)]
pub struct Breakout {
    computer: Computer,
    screen: Screen,
    controller: BoxedController,
}

impl Breakout {
    pub fn new(program: Program, controller: BoxedController) -> Self {
        Breakout {
            computer: Computer::new(program),
            screen: Screen::default(),
            controller: controller,
        }
    }

    pub fn new_without_controller(program: Program) -> Self {
        Breakout::new(program, Box::new(NeutralController::new()))
    }

    pub fn new_with_coins(mut program: Program, controller: BoxedController) -> Self {
        program.insert(0, 2).unwrap();
        Breakout::new(program, controller)
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
                CPUState::Input => {
                    let _ = match self.controller.control(&self.screen) {
                        Joystick::Left => self.computer.feed(-1),
                        Joystick::Neutral => self.computer.feed(0),
                        Joystick::Right => self.computer.feed(1),
                    };
                    return Ok(State::Step);
                }
                CPUState::Halt => return Ok(State::Halt),
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            match self.step()? {
                State::Step => {}
                State::Halt => break,
            }
        }
        Ok(())
    }

    pub fn next(&mut self) -> Result<&Screen> {
        self.step()?;
        Ok(&self.screen)
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
    }
}
