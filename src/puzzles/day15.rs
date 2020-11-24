use anyhow::anyhow;
use anyhow::Error;
use geometry::coord2d::{pathfinder, BoundingBox, Direction, Point};
use geometry::Position;
use intcode::{CPUState, Computer, IntMem, Program};

use std::collections::{HashMap, VecDeque};
use std::convert::{TryFrom, TryInto};
use std::default::Default;
use std::fmt::{self, Debug};
use std::io::Read;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Empty,
    Wall,
    OxygenSystem,
}

impl TryFrom<IntMem> for Tile {
    type Error = Error;

    fn try_from(value: IntMem) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Tile::Wall),
            1 => Ok(Tile::Empty),
            2 => Ok(Tile::OxygenSystem),
            _ => Err(anyhow!("Unknown robot observation: {}", value)),
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Empty => write!(f, "."),
            Tile::Wall => write!(f, "#"),
            Tile::OxygenSystem => write!(f, "O"),
        }
    }
}

trait RemoteDroid: Debug {
    fn command(&mut self, direction: Direction) -> Result<Tile, Error>;
}

#[derive(Debug)]
struct Droid {
    location: Point,
    controller: Box<dyn RemoteDroid + 'static>,
}

impl Droid {
    fn new<C>(controller: C) -> Self
    where
        C: RemoteDroid + 'static,
    {
        Self {
            location: Point::origin(),
            controller: Box::new(controller),
        }
    }

    fn location(&self) -> Point {
        self.location
    }

    fn step(&mut self, direction: Direction) -> Result<Tile, Error> {
        let tile = self.controller.command(direction)?;

        match tile {
            Tile::Empty => {
                self.location = self.location.step(direction);
            }
            Tile::OxygenSystem => {
                self.location = self.location.step(direction);
            }
            Tile::Wall => {}
        };

        Ok(tile)
    }
}

#[derive(Debug, Default, Clone)]
struct Map {
    tiles: HashMap<Point, Tile>,
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = Map::default();
        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                if let Some(tile) = match c {
                    '.' => Some(Tile::Empty),
                    'D' => Some(Tile::Empty),
                    '#' => Some(Tile::Wall),
                    'O' => Some(Tile::OxygenSystem),
                    ' ' => None,
                    _ => return Err(anyhow!("Unknwon map tile: {}", c)),
                } {
                    map.insert((x as Position, y as Position).into(), tile);
                }
            }
        }
        Ok(map)
    }
}

impl pathfinder::Map for Map {
    fn is_traversable(&self, location: Point) -> bool {
        // We can traverse unknown squares -- the robot will
        // correctly back out of a collision.
        self.check(location)
            .map(|t| t != Tile::Wall)
            .unwrap_or(true)
    }
}

impl Map {
    /// Mark the location of a tile.
    fn insert(&mut self, point: Point, tile: Tile) {
        self.tiles.insert(point, tile);
    }

    /// Check what tile is at a given location
    fn check(&self, point: Point) -> Option<Tile> {
        self.tiles.get(&point).copied()
    }

    /// Iterate over points of a given tile type
    fn locate(&self, tile: Tile) -> impl Iterator<Item = Point> {
        self.tiles
            .clone()
            .into_iter()
            .filter(move |&(_, t)| t == tile)
            .map(|(p, _)| p)
    }

    /// A pathfinidng object for realized paths
    fn realized(&self) -> Realized {
        Realized { map: self }
    }

    fn bbox(&self) -> BoundingBox {
        BoundingBox::from_points(self.tiles.keys())
    }
}

/// Implement pathfinding over only explored / realized
/// tiles, excluding any unexplored point.
#[derive(Debug)]
struct Realized<'m> {
    map: &'m Map,
}

impl<'m> pathfinder::Map for Realized<'m> {
    fn is_traversable(&self, location: Point) -> bool {
        // We can traverse unknown squares -- the robot will
        // correctly back out of a collision.
        self.map
            .check(location)
            .map(|t| t != Tile::Wall)
            .unwrap_or(false)
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bbox = self.bbox().margin(1);

        bbox.printer(f, |f, p| {
            if let Some(tile) = self.check(*p) {
                write!(f, "{}", tile)
            } else {
                write!(f, " ")
            }
        })
    }
}

#[derive(Debug)]
struct ShipSection {
    droid: Droid,
    map: Map,
}

impl ShipSection {
    fn new(droid: Droid) -> Self {
        let mut map = Map::default();
        map.insert(droid.location(), Tile::Empty);
        Self {
            droid: droid,
            map: map,
        }
    }

    fn from_program(program: Program) -> Self {
        ShipSection::new(Droid::new(Controller::new(program)))
    }

    fn walk(&mut self, direction: Direction) -> Result<Tile, Error> {
        let position = self.droid.location().step(direction);
        let tile = self.droid.step(direction)?;
        self.map.insert(position, tile);
        Ok(tile)
    }

    /// Find the shortest path to a given type of tile (usually OxygenSystem)
    /// on a partially explored map.
    fn find_path_to_tile(&mut self, target: Tile) -> Result<pathfinder::Path, Error> {
        use geometry::coord2d::pathfinder::Map;

        let droid_origin = self.droid().location();
        let mut queue: VecDeque<Point> = {
            let mut queue = VecDeque::with_capacity(4);
            for candidate in Direction::all()
                .map(|d| self.droid().location().step(d))
                .filter(|&p| self.map.check(p).is_none())
            {
                queue.push_front(candidate);
            }
            queue
        };

        let mut candidates: Vec<pathfinder::Path> = self
            .map
            .locate(target)
            .flat_map(|p| self.map.path(droid_origin, p))
            .collect();

        while let Some(to_explore) = queue.pop_front() {
            if self.map.check(to_explore).is_some() {
                continue;
            }

            // How do we get to the point to explore
            let path = self
                .map
                .path(self.droid().location(), to_explore)
                .ok_or(anyhow!(
                    "No path from droid at {:?} to {:?}: \n {}",
                    self.droid().location(),
                    to_explore,
                    self.map
                ))?;

            let candidate_distance = self
                .map
                .realized()
                .path(droid_origin, to_explore)
                .map(|p| p.distance())
                .unwrap_or(0);

            // Go there, but only if it could be better than
            // all of our other options.
            if candidate_distance
                < candidates
                    .iter()
                    .map(|p| p.distance())
                    .max()
                    .unwrap_or(usize::MAX)
            {
                // Walk along the path to our destination
                for step in path.iter().skip(1) {
                    let tile = self.walk(self.droid().location().direction(*step).unwrap())?;

                    if tile != Tile::Wall {
                        // If we stumble upon the target along the way,
                        // add a candidate to our list of paths
                        if target == tile {
                            candidates.push(
                                self.map
                                    .realized()
                                    .path(droid_origin, self.droid().location())
                                    .ok_or(anyhow!("No path found to visible target!"))?,
                            );
                        }
                    } else {
                        queue.push_front(to_explore);
                        break;
                    }
                }
            }

            // From here, where else can we go? Check all neighbors
            // of the current space, and put them on the front of the queue
            // becasue they are close by.
            for candidate in Direction::all()
                .map(|d| self.droid().location().step(d))
                .filter(|&p| self.map.check(p).is_none())
            {
                queue.push_front(candidate);
            }
        }

        candidates
            .iter()
            .min_by_key(|&p| p.distance())
            .ok_or(anyhow!("No paths found to visible target!"))
            .cloned()
    }

    fn time_to_oxygenate(&mut self) -> Result<usize, Error> {
        use geometry::coord2d::pathfinder::Map;
        use std::collections::HashSet;

        let mut oxygenated = HashSet::new();
        let mut edges: VecDeque<_> = self.map.locate(Tile::OxygenSystem).take(1).collect();
        let mut duration = 0;

        loop {
            let mut new_edges = VecDeque::with_capacity(edges.len());
            while let Some(edge) = edges.pop_front() {
                // TODO: Maybe the droid needs to walk to this edge to explore it?
                oxygenated.insert(edge);
                for next_edge in Direction::all()
                    .map(|d| edge.step(d))
                    .filter(|e| !oxygenated.contains(e))
                {
                    if self.map.realized().is_traversable(next_edge) {
                        new_edges.push_back(next_edge);
                    } else if self.map.check(next_edge).is_none() {
                        panic!("Map contains unexplored spaces")
                    }
                }
            }
            if new_edges.is_empty() {
                break;
            }
            edges = new_edges;
            duration += 1;
        }

        Ok(duration)
    }

    fn droid(&self) -> &Droid {
        &self.droid
    }
}

impl fmt::Display for ShipSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bbox = self.map.bbox().margin(1);
        let droid = self.droid().location();

        bbox.printer(f, |f, point| {
            if *point == droid {
                write!(f, "D")
            } else {
                if let Some(tile) = self.map.check(*point) {
                    write!(f, "{}", tile)
                } else {
                    write!(f, " ")
                }
            }
        })
    }
}

#[derive(Debug)]
struct Controller {
    cpu: Computer,
}

impl Controller {
    fn new(program: Program) -> Self {
        Self {
            cpu: Computer::new(program),
        }
    }
}

impl RemoteDroid for Controller {
    fn command(&mut self, direction: Direction) -> Result<Tile, Error> {
        let cmd = match direction {
            Direction::Up => 1,
            Direction::Down => 2,
            Direction::Left => 3,
            Direction::Right => 4,
        };
        self.cpu.feed(cmd)?;
        loop {
            match self.cpu.op()? {
                CPUState::Output(t) => {
                    return t.try_into();
                }
                CPUState::Continue => {}
                CPUState::Input => {
                    return Err(anyhow!("Computer expected input!"));
                }
                CPUState::Halt => {
                    return Err(anyhow!("Comptuer halted!"));
                }
            }
        }
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let program = Program::read(input)?;

    let mut ship = ShipSection::from_program(program.clone());

    let path = ship.find_path_to_tile(Tile::OxygenSystem)?;
    println!("Part 1: {} steps to the oxygen system", path.distance());

    let duration = ship.time_to_oxygenate()?;
    println!("Part 2: {} minutes to oxyngenate the system", duration);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::get_default_input;

    #[derive(Debug)]
    struct UniformRoom(Tile);

    impl UniformRoom {
        fn new(tile: Tile) -> Self {
            UniformRoom(tile)
        }
    }

    impl RemoteDroid for UniformRoom {
        fn command(&mut self, _: Direction) -> Result<Tile, Error> {
            Ok(self.0)
        }
    }

    #[derive(Debug)]
    struct MappedRoom {
        map: Map,
        position: Point,
    }

    impl MappedRoom {
        fn new(map: Map, start: Point) -> Self {
            assert_eq!(map.check(start).unwrap(), Tile::Empty);

            Self {
                map,
                position: start,
            }
        }
    }

    impl RemoteDroid for MappedRoom {
        fn command(&mut self, direction: Direction) -> Result<Tile, Error> {
            let location = self.position.step(direction);
            let tile = self
                .map
                .check(location)
                .ok_or(anyhow!("Walked off the map!"))?;
            if Tile::Wall != tile {
                self.position = location;
            }
            Ok(tile)
        }
    }

    #[test]
    fn droid_movement() {
        let mut ship = ShipSection::new(Droid::new(UniformRoom::new(Tile::Empty)));

        let cmds = vec![
            Direction::Left,
            Direction::Left,
            Direction::Left,
            Direction::Left,
            Direction::Right,
            Direction::Right,
            Direction::Right,
            Direction::Right,
        ];

        for cmd in cmds {
            ship.walk(cmd).unwrap();
        }
        assert_eq!(ship.droid().location(), Point::origin());
    }
    #[test]
    fn droid_collision() {
        let mut ship = ShipSection::new(Droid::new(UniformRoom::new(Tile::Wall)));

        ship.walk(Direction::Up).unwrap();
        assert_eq!(ship.droid().location(), Point::origin());
    }

    #[test]
    fn example_map() {
        let map: Map = "      \n   ## \n  #..#\n  D.# \n   #  ".parse().unwrap();
        assert_eq!(map.check(Point::origin()), None);
        assert_eq!(map.check((2, 2).into()), Some(Tile::Wall));
    }

    #[test]
    fn check_ship() {
        let map: Map = include_str!("../../geometry/examples/pathfinding_multi.txt")
            .parse()
            .unwrap();
        let droid = Droid::new(MappedRoom::new(map.clone(), (1, 1).into()));
        let ship = ShipSection::new(droid);
        assert!(ship.map.check(ship.droid.location).is_some());
    }

    #[test]
    fn explore_empty_map() {
        let map: Map = include_str!("../../geometry/examples/pathfinding_multi.txt")
            .parse()
            .unwrap();
        let droid = Droid::new(MappedRoom::new(map.clone(), (1, 1).into()));
        let mut ship = ShipSection::new(droid);
        assert!(ship.find_path_to_tile(Tile::OxygenSystem).is_err());
    }

    #[test]
    fn explore_simple_map() {
        let mut map: Map = include_str!("../../geometry/examples/pathfinding_multi.txt")
            .parse()
            .unwrap();
        map.insert((1, 12).into(), Tile::OxygenSystem);

        let droid = Droid::new(MappedRoom::new(map.clone(), (1, 1).into()));
        let mut ship = ShipSection::new(droid);
        eprintln!("Finding path on {}", map);
        let path = ship.find_path_to_tile(Tile::OxygenSystem).unwrap();
        assert_eq!(path.distance(), 19);
    }

    #[test]
    fn answers() {
        let program = Program::read(get_default_input(15).unwrap()).unwrap();
        let mut ship = ShipSection::from_program(program.clone());
        let path = ship.find_path_to_tile(Tile::OxygenSystem).unwrap();
        assert_eq!(path.distance(), 282);
        assert_eq!(ship.time_to_oxygenate().unwrap(), 286);
    }

    #[test]
    fn example_part2() {
        let map: Map = "
######
#..###
#.#..#
#.O.##
######
        "
        .parse()
        .unwrap();

        let droid = Droid::new(MappedRoom::new(map.clone(), (1, 2).into()));
        let mut ship = ShipSection::new(droid);
        eprintln!("Finding path on {}", map);
        let path = ship.find_path_to_tile(Tile::OxygenSystem).unwrap();
        assert_eq!(path.distance(), 3);
        eprintln!("Oxygenating {}", ship);
        assert_eq!(ship.time_to_oxygenate().unwrap(), 4);
    }
}
