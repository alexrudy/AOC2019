use anyhow::Error;
use aoc2019::get_default_input;
use aoc2019::puzzles::day18::debug_method;

fn main() -> Result<(), Error> {
    let reader = get_default_input(18)?;

    debug_method(reader)?;

    Ok(())
}
