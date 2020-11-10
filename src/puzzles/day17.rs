use anyhow::{anyhow, Error};
use intcode::{CPUState, Computer, Program};

use std::convert::TryInto;
use std::io::Read;

use self::map::Map;
use self::movement::MovementPrograms;

mod map;
mod movement;
mod path;

#[derive(Debug)]
struct Camera {
    cpu: Computer,
}

impl Camera {
    fn new(program: Program) -> Self {
        Camera {
            cpu: Computer::new(program),
        }
    }

    fn new_activated(mut program: Program) -> Self {
        program.insert(0, 2).unwrap();
        Camera::new(program)
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
                    return Err(anyhow!("Unexpected input during image capture"));
                }
                CPUState::Halt => break,
            }
        }

        image.parse()
    }

    fn run(&mut self, movement: String) -> Result<i64, Error> {
        for c in format!("{}\nn\n", movement).bytes() {
            loop {
                match self.cpu.op()? {
                    CPUState::Output(_) => {}
                    CPUState::Continue => {}
                    CPUState::Halt => {
                        return Err(anyhow!("Unexpected halt from CPU during program load!"));
                    }
                    CPUState::Input => {
                        self.cpu.feed(c as i64)?;
                        break;
                    }
                }
            }
        }

        let mut result = None;
        loop {
            match self.cpu.op()? {
                CPUState::Output(v) => {
                    result.replace(v);
                }
                CPUState::Continue => {}
                CPUState::Halt => {
                    break;
                }
                CPUState::Input => {
                    return Err(anyhow!("Unexpected input from CPU!"));
                }
            }
        }

        result.ok_or(anyhow!("No output recieved from CPU!"))
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let program = Program::read(input)?;

    let mut camera = Camera::new(program.clone());
    let map = camera.capture()?;

    println!("Part 1: {}", map.alignment_parameter());

    let mut camera = Camera::new_activated(program.clone());
    let movement_program = MovementPrograms::compile(map.path()?)?.to_string();
    println!("Part 2: {}", camera.run(movement_program)?);

    Ok(())
}

#[cfg(test)]
mod test {
    use geometry::coord2d::Direction;

    use super::map::Tile;
    use super::movement::{MovementProgram, MovementPrograms};
    use super::path::Step;
    use super::*;
    use crate::get_default_input;

    #[test]
    fn parse_tile() {
        assert_eq!(".".parse::<Tile>().unwrap(), Tile::Open);
        assert_eq!("#".parse::<Tile>().unwrap(), Tile::Scaffold);
        assert_eq!("^".parse::<Tile>().unwrap(), Tile::Robot(Direction::Up));
        assert_eq!("<".parse::<Tile>().unwrap(), Tile::Robot(Direction::Left));
        assert_eq!(">".parse::<Tile>().unwrap(), Tile::Robot(Direction::Right));
        assert_eq!("v".parse::<Tile>().unwrap(), Tile::Robot(Direction::Down));
    }

    fn example_map() -> Map {
        include_str!("../../puzzles/17/example_a_map.txt")
            .parse()
            .unwrap()
    }

    #[test]
    fn examples_part1() {
        let map = example_map();
        assert_eq!(map.get((3, 2).into()), Tile::Scaffold);
        assert_eq!(map.get((0, 0).into()), Tile::Open);

        assert_eq!(map.is_intersection((2, 2).into()), true);
        assert_eq!(map.is_intersection((3, 2).into()), false);

        assert_eq!(map.intersections().len(), 4);
        assert_eq!(map.alignment_parameter(), 76);
    }

    #[test]
    fn answer_part1() {
        let program = Program::read(get_default_input(17).unwrap()).unwrap();
        let mut camera = Camera::new(program);
        let map = camera.capture().unwrap();
        assert_eq!(map.alignment_parameter(), 7720);
    }

    #[test]
    fn examples_part2a() {
        let map = example_map();
        let mut pf = map.pathfinder().unwrap();

        assert_eq!(pf.next().unwrap().unwrap(), Step::Forward);
    }

    #[test]
    fn examples_part2b() {
        let map: Map = include_str!("../../puzzles/17/example_b_map.txt")
            .parse()
            .unwrap();

        let movement: MovementProgram = map.path().unwrap().into();
        let commands = movement.to_string();
        assert_eq!(
            commands,
            "R,8,R,8,R,4,R,4,R,8,L,6,L,2,R,4,R,4,R,8,R,8,R,8,L,6,L,2"
        );

        let cprog = MovementPrograms::compile(movement).unwrap();

        assert_eq!(cprog.expand().to_string(), commands);

        for line in cprog.to_string().lines() {
            assert!(line.trim().len() <= 20);
        }
    }

    #[test]
    fn answer_part2() {}
}
