use anyhow::Error;
use breakout::{Breakout, Screen, SimpleController, Tile};
use intcode::Program;

use std::io::Read;

fn first_screen(program: Program) -> Result<Screen, Error> {
    let mut breakout = Breakout::new_without_controller(program);
    breakout.next().cloned()
}

fn play_simple(program: Program) -> Result<i64, Error> {
    let controller = Box::new(SimpleController::new());
    let mut breakout = Breakout::new_with_coins(program, controller);
    breakout.run()?;
    Ok(breakout.screen().score())
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let arcade = Program::read(input)?;

    let screen = first_screen(arcade.clone())?;

    println!("Part 1: {} block tiles", screen.count(Tile::Block));

    let score = play_simple(arcade)?;
    println!("Part 2: Score is {} at the end of the game.", score);

    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::get_default_input;

    #[test]
    fn solution_day13_part1() {
        let arcade = Program::read(get_default_input(13).unwrap()).unwrap();

        let screen = first_screen(arcade).unwrap();
        assert_eq!(screen.count(Tile::Block), 251);
    }

    #[test]
    fn solution_day13_part2() {
        let arcade = Program::read(get_default_input(13).unwrap()).unwrap();
        let score = play_simple(arcade).unwrap();
        assert_eq!(score, 12779);
    }
}
