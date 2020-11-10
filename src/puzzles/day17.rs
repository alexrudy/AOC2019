use anyhow::{anyhow, Error};
use geometry::coord2d::{Direction, Point};
use geometry::Position;
use intcode::{CPUState, Computer, Program};

use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::io::Read;
use std::str::FromStr;

use self::movement::MovementPrograms;
use self::path::{Path, Pathfinder};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
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
struct Robot {
    location: Point,
    direction: Direction,
}

impl Robot {
    fn new(location: Point, direction: Direction) -> Self {
        Robot {
            location,
            direction,
        }
    }

    fn forward(&self) -> Point {
        self.location.step(self.direction)
    }

    fn left(&self) -> Point {
        self.location.step(self.direction.turn_left())
    }

    fn right(&self) -> Point {
        self.location.step(self.direction.turn_right())
    }
}

#[derive(Debug, Default)]
pub struct Map {
    scaffold: HashSet<Point>,
    robot: Option<Robot>,
}

impl Map {
    fn get(&self, location: Point) -> Tile {
        if self.scaffold.contains(&location) {
            Tile::Scaffold
        } else {
            Tile::Open
        }
    }

    fn insert(&mut self, location: Point, tile: Tile) {
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

    fn is_intersection(&self, location: Point) -> bool {
        Direction::all().all(|d| self.get(location.step(d)) != Tile::Open)
    }

    fn intersections(&self) -> Vec<Point> {
        self.scaffold
            .iter()
            .filter(|&p| self.is_intersection(*p))
            .copied()
            .collect()
    }

    fn alignment_parameter(&self) -> Position {
        self.intersections().iter().map(|p| p.x * p.y).sum()
    }

    fn pathfinder(&self) -> Result<Pathfinder, Error> {
        Pathfinder::new(self)
    }

    fn path(&self) -> Result<Path, Error> {
        self.pathfinder()?.path()
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

mod path {
    use anyhow::anyhow;
    use anyhow::Error;
    use geometry::coord2d::Point;

    use std::collections::HashSet;
    use std::collections::VecDeque;

    use super::{Map, Robot};

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Step {
        Forward,
        Left,
        Right,
    }

    #[derive(Debug, Clone)]
    pub struct Path(Vec<Step>);

    impl Path {
        pub fn steps(&self) -> impl Iterator<Item = &Step> {
            self.0.iter()
        }
    }

    #[derive(Debug)]
    pub struct Pathfinder<'m> {
        map: &'m Map,
        visited: HashSet<Point>,
        robot: Robot,
        queue: VecDeque<Step>,
        poisoned: bool,
    }

    impl<'m> Pathfinder<'m> {
        pub fn new(map: &'m Map) -> Result<Self, Error> {
            let robot = map.robot.ok_or(anyhow!("No robot present!"))?;
            let mut visited = HashSet::new();
            visited.insert(robot.location);

            Ok(Pathfinder {
                map: map,
                visited: visited,
                robot: robot,
                queue: VecDeque::new(),
                poisoned: false,
            })
        }

        pub fn path(self) -> Result<Path, Error> {
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

            if self.visited.len() == self.map.scaffold.len() {
                return None;
            }

            let forward = self.robot.forward();
            if self.map.scaffold.contains(&forward) {
                self.queue.push_back(Step::Forward);
                self.visited.insert(forward);
                self.robot.location = self.robot.forward();
            } else if self.map.scaffold.contains(&self.robot.left())
                && !self.visited.contains(&self.robot.left())
            {
                self.queue.push_back(Step::Left);
                self.queue.push_back(Step::Forward);
                self.visited.insert(self.robot.left());
                self.robot.location = self.robot.left();
                self.robot.direction = self.robot.direction.turn_left();
            } else if self.map.scaffold.contains(&self.robot.right())
                && !self.visited.contains(&self.robot.right())
            {
                self.queue.push_back(Step::Right);
                self.queue.push_back(Step::Forward);
                self.visited.insert(self.robot.right());
                self.robot.location = self.robot.right();
                self.robot.direction = self.robot.direction.turn_right();
            } else {
                self.poisoned = true;
                return Some(Err(anyhow!("No movement options remain!")));
            };

            self.queue.pop_front().map(|s| Ok(s))
        }
    }
}

mod movement {

    use anyhow::{anyhow, Error};

    use std::collections::HashMap;
    use std::convert::Into;
    use std::ops::Deref;

    use super::path::{Path, Step};

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Movement {
        Forward(usize),
        TurnLeft,
        TurnRight,
    }

    impl ToString for Movement {
        fn to_string(&self) -> String {
            match self {
                Movement::Forward(steps) => format!("{}", steps),
                Movement::TurnLeft => "L".to_string(),
                Movement::TurnRight => "R".to_string(),
            }
        }
    }

    #[derive(Debug, Default, Clone)]
    pub struct MovementProgram(Vec<Movement>);

    impl From<Path> for MovementProgram {
        fn from(path: Path) -> Self {
            let mut cmds = Vec::new();

            let mut forward = 0;

            for step in path.steps() {
                match step {
                    Step::Forward => {
                        forward += 1;
                    }
                    Step::Left => {
                        if forward > 0 {
                            cmds.push(Movement::Forward(forward));
                        }
                        forward = 0;
                        cmds.push(Movement::TurnLeft);
                    }
                    Step::Right => {
                        if forward > 0 {
                            cmds.push(Movement::Forward(forward));
                        }
                        forward = 0;
                        cmds.push(Movement::TurnRight);
                    }
                }
            }

            if forward > 0 {
                cmds.push(Movement::Forward(forward));
            }

            MovementProgram(cmds)
        }
    }

    impl From<Vec<Movement>> for MovementProgram {
        fn from(v: Vec<Movement>) -> Self {
            MovementProgram(v)
        }
    }

    impl ToString for MovementProgram {
        fn to_string(&self) -> String {
            self.0
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join(",")
        }
    }

    impl MovementProgram {
        fn starts_with(&self, other: &[Movement]) -> bool {
            if other.len() > self.len() {
                return false;
            }

            self.0.starts_with(other)
        }

        fn size(&self) -> usize {
            self.to_string().len()
        }

        fn is_small(&self) -> bool {
            self.size() <= 20
        }
    }

    impl Deref for MovementProgram {
        type Target = [Movement];
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
    enum Routine {
        A,
        B,
        C,
    }

    const ROUTINES: [Routine; 3] = [Routine::A, Routine::B, Routine::C];

    impl From<usize> for Routine {
        fn from(idx: usize) -> Self {
            match idx {
                0 => Routine::A,
                1 => Routine::B,
                2 => Routine::C,
                _ => panic!("Unexpected routine index"),
            }
        }
    }

    impl ToString for Routine {
        fn to_string(&self) -> String {
            match self {
                Routine::A => "A".to_string(),
                Routine::B => "B".to_string(),
                Routine::C => "C".to_string(),
            }
        }
    }

    #[derive(Debug, Default)]
    struct MovementRoutine(Vec<Routine>);

    impl ToString for MovementRoutine {
        fn to_string(&self) -> String {
            self.0
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join(",")
        }
    }

    #[derive(Debug, Default)]
    pub struct MovementPrograms {
        main: MovementRoutine,
        routines: HashMap<Routine, MovementProgram>,
    }

    impl ToString for MovementPrograms {
        fn to_string(&self) -> String {
            use std::fmt::Write;

            let mut buf = String::new();

            writeln!(buf, "{}", self.main.to_string()).unwrap();

            for routine in &ROUTINES {
                writeln!(
                    buf,
                    "{}",
                    self.routines
                        .get(routine)
                        .map(|m| m.to_string())
                        .unwrap_or("".to_string())
                )
                .unwrap();
            }

            buf
        }
    }

    impl MovementPrograms {
        pub fn compile<P>(program: P) -> Result<MovementPrograms, Error>
        where
            P: Into<MovementProgram>,
        {
            let p = program.into();
            for a in 1..=11 {
                for b in 1..=11 {
                    for c in 1..=11 {
                        match MovementPrograms::build(&p, a, b, c) {
                            Ok(r) => {
                                return Ok(r);
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
            Err(anyhow!("Unable to build program!"))
        }

        fn build(
            program: &MovementProgram,
            a: usize,
            b: usize,
            c: usize,
        ) -> Result<MovementPrograms, Error> {
            let r_a: MovementProgram = program.iter().take(a).copied().collect::<Vec<_>>().into();

            if !r_a.is_small() {
                return Err(anyhow!("Routine A does not fit in memory: {:?}", r_a));
            }

            let mut programs = MovementPrograms::default();
            programs.routines.insert(Routine::A, r_a);

            let mut remainder = program.clone();

            loop {
                match programs.strip(&remainder) {
                    Some((r, p)) => {
                        programs.main.0.push(r);
                        remainder = p;
                    }
                    None => {
                        if remainder.is_empty() {
                            return Ok(programs);
                        }
                        if !programs.routines.contains_key(&Routine::B) {
                            let r_b: MovementProgram =
                                remainder.iter().take(b).copied().collect::<Vec<_>>().into();
                            if !r_b.is_small() {
                                return Err(anyhow!("Routine B does not fit in memory: {:?}", r_b));
                            }

                            programs.routines.insert(Routine::B, r_b.into());
                        } else if !programs.routines.contains_key(&Routine::C) {
                            let r_c: MovementProgram =
                                remainder.iter().take(c).copied().collect::<Vec<_>>().into();
                            if !r_c.is_small() {
                                return Err(anyhow!("Routine B does not fit in memory: {:?}", r_c));
                            }
                            programs.routines.insert(Routine::C, r_c);
                        } else {
                            return Err(anyhow!(
                                "Unable to consume program: {:?} {:?}",
                                program,
                                programs
                            ));
                        }
                    }
                }

                if programs.main.0.len() >= 11 && !remainder.is_empty() {
                    return Err(anyhow!(
                        "Insufficient memory in main routine: {:?} {:?}",
                        programs,
                        remainder
                    ));
                }
            }
        }

        fn strip(&self, program: &MovementProgram) -> Option<(Routine, MovementProgram)> {
            for (i, subprogram) in self.routines.iter() {
                if program.starts_with(subprogram) {
                    let remiander: MovementProgram = program
                        .iter()
                        .skip(subprogram.len())
                        .copied()
                        .collect::<Vec<_>>()
                        .into();

                    return Some((*i, remiander));
                }
            }
            None
        }

        pub fn expand(&self) -> MovementProgram {
            let mut result = Vec::new();

            for routine in &self.main.0 {
                result.extend(self.routines.get(routine).unwrap().iter())
            }

            result.into()
        }
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
