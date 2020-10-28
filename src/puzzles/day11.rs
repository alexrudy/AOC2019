use anyhow::{anyhow, Error, Result};
use geometry::coord2d::{BoundingBox, Direction, Point};
use intcode::{CPUState, Computer, Program};
use std::collections::HashSet;
use std::fmt;
use std::io::Read;

#[derive(Debug)]
struct Robot {
    direction: Direction,
    location: Point,
    computer: Computer,
}

impl Robot {
    fn new(program: Program) -> Self {
        Robot {
            direction: Direction::Up,
            location: Point::origin(),
            computer: Computer::new(program),
        }
    }

    fn paint_hull(&mut self, hull: &mut Hull) -> Result<()> {
        loop {
            loop {
                match self.computer.op()? {
                    CPUState::Continue => {}
                    CPUState::Input => self.computer.feed(hull.view(self.location).to_camera())?,
                    CPUState::Output(0) => {
                        hull.paint(self.location, Panel::Black);
                        break;
                    }
                    CPUState::Output(1) => {
                        hull.paint(self.location, Panel::White);
                        break;
                    }
                    CPUState::Output(output) => {
                        Err(anyhow!("Invalid output from robot: {}", output))?
                    }
                    CPUState::Halt => {
                        return Ok(());
                    }
                }
            }
            loop {
                match self.computer.op()? {
                    CPUState::Continue => {}
                    CPUState::Input => self.computer.feed(hull.view(self.location).to_camera())?,
                    CPUState::Output(0) => {
                        self.direction = self.direction.turn_left();
                        break;
                    }
                    CPUState::Output(1) => {
                        self.direction = self.direction.turn_right();
                        break;
                    }
                    CPUState::Output(output) => {
                        Err(anyhow!("Invalid output from robot: {}", output))?
                    }
                    CPUState::Halt => {
                        return Ok(());
                    }
                }
            }

            self.location = self.location.step(self.direction);
        }
    }
}

#[derive(Debug)]
enum Panel {
    Black,
    White,
}

impl fmt::Display for Panel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Panel::Black => write!(f, " "),
            Panel::White => write!(f, "█"),
        }
    }
}

impl Panel {
    fn to_camera(&self) -> i64 {
        match self {
            Panel::Black => 0,
            Panel::White => 1,
        }
    }
}

#[derive(Debug, Default)]
struct Hull {
    // Only contains white panels
    panels: HashSet<Point>,
    painted: HashSet<Point>,
}

impl Hull {
    fn paint(&mut self, position: Point, panel: Panel) -> () {
        match panel {
            Panel::Black => self.panels.remove(&position),
            Panel::White => self.panels.insert(position),
        };
        self.painted.insert(position);
    }

    fn view(&self, position: Point) -> Panel {
        match self.panels.contains(&position) {
            true => Panel::White,
            false => Panel::Black,
        }
    }

    fn with_starting_panel() -> Self {
        let mut hull = Self::default();
        hull.panels.insert(Point::origin());
        hull
    }
}

impl fmt::Display for Hull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bbox = BoundingBox::from_points(self.panels.iter().cloned()).margin(1);

        for y in bbox.vertical() {
            for x in bbox.horizontal() {
                write!(f, "{}", self.view((x, y).into()))?;
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let controller = Program::read(input)?;
    {
        let mut hull = Hull::default();
        let mut robot = Robot::new(controller.clone());

        robot.paint_hull(&mut hull)?;
        println!("Part 1: Painted {} squares", hull.painted.len());
    }
    {
        let mut hull = Hull::with_starting_panel();
        let mut robot = Robot::new(controller.clone());

        robot.paint_hull(&mut hull)?;
        println!("Part 2: Painted {} squares", hull.painted.len());
        println!("{}", hull);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
}
