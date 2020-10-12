use anyhow::Error;
use std::io::Read;

use intcode::{Computer, Program};

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let program = Program::read(input)?;

    let mut cpu = Computer::new(program);

    // Set to air conditioner ID
    cpu.feed(1);

    // Find non-zero outputs
    let outputs = cpu.follow().skip_while(|e| *e == 0).collect::<Vec<i32>>();
    if outputs.len() != 1 {
        eprintln!("Unexpected Outputs: {:?}", outputs);
    }
    println!("Part 1: Diagonstic Code = {}", outputs[0]);

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
