use anyhow::Error;
use std::io::Read;

use intcode::{Computer, IntMem, Program};

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let program = Program::read(input)?;

    {
        let mut cpu = Computer::new(program.clone());

        // Set to test mode
        cpu.feed(1)?;

        // Find non-zero outputs
        let outputs = cpu
            .follow()
            .skip_while(|e| *e == 0)
            .collect::<Vec<IntMem>>();
        if outputs.len() != 1 {
            eprintln!("Unexpected Outputs: {:?}", outputs);
        }
        println!("Part 1: Diagonstic Code = {}", outputs[0]);
    }

    {
        let mut cpu = Computer::new(program.clone());

        // Set to boost mode
        cpu.feed(2)?;

        // Find non-zero outputs
        let outputs = cpu.follow().collect::<Vec<IntMem>>();
        if outputs.len() != 1 {
            eprintln!("Unexpected Outputs: {:?}", outputs);
        }
        println!("Part 2: Coordinates = {}", outputs[0]);
    }

    Ok(())
}
