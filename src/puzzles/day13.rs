use anyhow::Error;
use breakout::{Breakout, Screen, Tile};
use intcode::Program;

use std::io::Read;

fn run_game(program: Program) -> Result<Screen, Error> {
    let mut breakout = Breakout::new(program);
    breakout.next().cloned()
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
