use anyhow::Error;

use geometry::coord2d::{BoundingBox, Point};
use intcode::{CPUState, Computer, Program};

use std::collections::HashSet;
use std::io::Read;

#[derive(Debug, Default)]
struct Beam {
    cells: HashSet<Point>,
}

trait Scanner {
    fn scan(&mut self, location: &Point) -> bool;

    fn scan_bbox(&mut self, size: i32) -> Beam {
        let mut beam = Beam::default();

        let bbox = BoundingBox::new(0, size, 0, size);

        for point in bbox.points() {
            if self.scan(&point) {
                beam.cells.insert(point);
            }
        }
        beam
    }
}

#[derive(Debug)]
struct IntScanner {
    cpu: Computer,
}

impl IntScanner {
    fn new(program: Program) -> Self {
        Self {
            cpu: Computer::new(program),
        }
    }
}

impl Scanner for IntScanner {
    fn scan(&mut self, location: &Point) -> bool {
        eprintln!("Checking {:?}", location);
        self.cpu.feed(location.x as i64).unwrap();
        if !matches!(self.cpu.run().unwrap(), CPUState::Input) {
            panic!("Expected input position!");
        };
        self.cpu.feed(location.y as i64).unwrap();
        match self.cpu.run().unwrap() {
            CPUState::Output(v) => v == 1,
            s => panic!("Unexpected CPU State: {:?}", s),
        }
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let program = Program::read(input)?;

    {
        let b = IntScanner::new(program).scan_bbox(50);
        println!("Part 1: {}", b.cells.len());
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples_part1() {}

    #[test]
    fn answer_part1() {}

    #[test]
    fn examples_part2() {}

    #[test]
    fn answer_part2() {}
}
