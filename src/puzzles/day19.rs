use anyhow::Error;
use itertools::chain;

use geometry::coord2d::{BoundingBox, Point};
use geometry::Position;
use intcode::{CPUState, Computer, Program};

use std::{collections::HashSet, io::Read};
use std::{
    collections::{HashMap, VecDeque},
    default::Default,
    fmt,
};

#[derive(Debug, Default, Clone)]
struct BeamBounds(HashMap<Position, (Option<Position>, Option<Position>)>);

impl BeamBounds {
    fn include(&mut self, major: Position, minor: Position) {
        let bounds = self.0.entry(major).or_insert((None, None));

        if bounds.0.map(|l| l > minor).unwrap_or(true) {
            bounds.0 = Some(minor);
        }

        if bounds.1.map(|h| h < minor).unwrap_or(false) {
            bounds.1 = None;
        }
    }

    fn exclude(&mut self, major: Position, minor: Position) {
        let bounds = self.0.entry(major).or_insert((None, None));

        if bounds.0.map(|l| l > minor).unwrap_or(false) {
            return;
        }

        if bounds.1.map(|h| h > minor).unwrap_or(true) {
            bounds.1 = Some(minor);
        }
    }

    fn contains(&self, major: Position, minor: Position) -> bool {
        self.0
            .get(&major)
            .map(|(l, h)| {
                if l.is_none() {
                    false
                } else {
                    l.unwrap_or(0) <= minor && minor < h.unwrap_or(Position::MAX)
                }
            })
            .unwrap_or(false)
    }

    fn min(&self, major: Position) -> Position {
        match self.0.get(&major).unwrap_or(&(None, None)) {
            (Some(l), _) => *l,
            (None, _) => 0,
        }
    }

    fn bound(&self, major: Position) -> Option<(Position, Position)> {
        self.0
            .get(&major)
            .map(|(l, h)| {
                if l.is_some() && h.is_some() {
                    Some((major, l.unwrap()))
                } else {
                    None
                }
            })
            .flatten()
    }
}

#[derive(Debug, Clone, Default)]
struct Beam {
    x: BeamBounds,
    y: BeamBounds,
    bbox: BoundingBox,
}

impl Beam {
    fn len(&self) -> usize {
        self.iter().count()
    }

    fn include(&mut self, location: Point) {
        let Point { x, y } = location;

        self.x.include(x, y);
        self.y.include(y, x);

        self.bbox.include(location);
    }

    fn exclude(&mut self, location: Point) {
        let Point { x, y } = location;

        self.x.exclude(x, y);
        self.y.exclude(y, x);
    }

    fn iter(&self) -> impl Iterator<Item = Point> + '_ {
        self.bbox.points().filter(move |p| self.contains(p))
    }

    fn contains(&self, location: &Point) -> bool {
        let Point { x, y } = *location;
        self.x.contains(x, y) && self.y.contains(y, x)
    }

    fn square(&self, size: i32) -> Option<Point> {
        chain(self.squarex(size).iter(), self.squarey(size).iter())
            .min_by_key(|&s| s.manhattan_distance(Point::origin()))
            .cloned()
    }

    fn squarechecky(&self, start: Point, size: i32) -> bool {
        let offset: Point = (1 * (size - 1), -1 * (size - 1)).into();
        self.contains(&start) && self.contains(&(start + offset))
    }
    fn squarecheckx(&self, start: Point, size: i32) -> bool {
        let offset: Point = (-1 * (size - 1), 1 * (size - 1)).into();
        self.contains(&start) && self.contains(&(start + offset))
    }

    fn squarey(&self, size: i32) -> Option<Point> {
        (0..self.bbox.top())
            .rev()
            .filter_map(|y| self.y.bound(y))
            .filter_map(|(y, x)| {
                if self.squarechecky((x, y).into(), size) {
                    let start: Point = (x, y - size + 1).into();
                    Some(start)
                } else {
                    None
                }
            })
            .min_by_key(|s| s.manhattan_distance(Point::origin()))
    }

    fn squarex(&self, size: i32) -> Option<Point> {
        (0..self.bbox.right())
            .rev()
            .filter_map(|x| self.x.bound(x))
            .filter_map(|(x, y)| {
                if self.squarecheckx((x, y).into(), size) {
                    let start: Point = (x - size + 1, y).into();
                    Some(start)
                } else {
                    None
                }
            })
            .min_by_key(|s| s.manhattan_distance(Point::origin()))
    }
}

impl fmt::Display for Beam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.bbox.printer(f, |f, p| {
            write!(f, "{}", if self.contains(p) { '#' } else { '.' })
        })
    }
}

struct BeamViewer(Beam, HashSet<Point>);

impl fmt::Display for BeamViewer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.bbox.printer(f, |f, p| {
            write!(
                f,
                "{}",
                if self.1.contains(p) {
                    if self.0.contains(p) {
                        'X'
                    } else {
                        '!'
                    }
                } else if self.0.contains(p) {
                    '#'
                } else {
                    '.'
                }
            )
        })
    }
}

trait Scanner {
    fn scan(&self, location: &Point) -> bool;
}

#[derive(Debug)]
struct IntScanner {
    program: Program,
}

impl IntScanner {
    fn new(program: Program) -> Self {
        Self { program: program }
    }
}

impl Scanner for IntScanner {
    fn scan(&self, location: &Point) -> bool {
        let mut cpu = Computer::new(self.program.clone());

        cpu.feed(location.x as i64).unwrap();
        if !matches!(cpu.run().unwrap(), CPUState::Input) {
            panic!("Expected input position!");
        };
        cpu.feed(location.y as i64).unwrap();
        match cpu.run().unwrap() {
            CPUState::Output(v) => v == 1,
            s => panic!("Unexpected CPU State: {:?}", s),
        }
    }
}

/// Scans an entire bounding box
fn scan_bbox<S: Scanner>(scanner: &S, size: i32) -> Beam {
    let mut beam = Beam::default();

    let bbox = BoundingBox::new(0, size - 1, 0, size - 1);

    for point in bbox.points() {
        if scanner.scan(&point) {
            beam.include(point);
        } else {
            beam.exclude(point);
        }
    }
    beam
}

fn part2(program: Program) -> Point {
    let s = IntScanner::new(program.clone());
    let mut beam = scan_bbox(&s, 10);
    let mut seen = HashSet::new();
    let mut queue: VecDeque<Point> = VecDeque::new();

    loop {
        let t = beam.x.min(beam.bbox.right());
        queue.push_back((beam.bbox.right(), t).into());
        queue.push_back((beam.bbox.right(), beam.bbox.bottom()).into());

        let l = beam.y.min(beam.bbox.bottom());
        queue.push_back((l, beam.bbox.bottom()).into());
        queue.push_back((beam.bbox.right(), beam.bbox.bottom()).into());

        while let Some(target) = queue.pop_front() {
            for dest in target.adjacent_diagonal() {
                if seen.insert(dest) && !beam.contains(&dest) {
                    if s.scan(&dest) {
                        beam.include(dest);
                    } else {
                        beam.exclude(dest);
                    }
                }
            }
        }

        if let Some(start) = beam.square(100) {
            // let bbox = BoundingBox::new(start.x, start.x + 4, start.y, start.y + 4);
            // let viewer = BeamViewer(beam, bbox.points().collect());
            // println!("{}", viewer);
            return start;
        }
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let program = Program::read(input)?;

    {
        let b = scan_bbox(&IntScanner::new(program.clone()), 50);
        println!("Part 1: {}", b.len());
    }
    {
        let s = part2(program.clone());
        println!("Square at {}", s);
        println!("Part 2: {}", s.x * 10000 + s.y)
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::get_default_input;

    #[test]
    fn answer_part1() {
        let program = Program::read(get_default_input(19).unwrap()).unwrap();
        let b = scan_bbox(&IntScanner::new(program), 50);

        assert_eq!(b.len(), 223);
    }

    #[test]
    fn answer_part2() {
        let program = Program::read(get_default_input(19).unwrap()).unwrap();

        assert_eq!(part2(program), (948, 761).into())
    }
}
