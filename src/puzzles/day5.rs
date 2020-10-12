use anyhow::Error;
use std::io::Read;

use intcode::{Computer, Program};

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let program = Program::read(input)?;

    {
        let mut cpu = Computer::new(program.clone());

        // Set to air conditioner ID
        cpu.feed(1);

        // Find non-zero outputs
        let outputs = cpu.follow().skip_while(|e| *e == 0).collect::<Vec<i32>>();
        if outputs.len() != 1 {
            eprintln!("Unexpected Outputs: {:?}", outputs);
        }
        println!("Part 1: Diagonstic Code = {}", outputs[0]);
    }

    {
        let mut cpu = Computer::new(program.clone());

        // Set to air conditioner ID
        cpu.feed(5);

        // Find non-zero outputs
        let outputs = cpu.follow().collect::<Vec<i32>>();
        if outputs.len() != 1 {
            eprintln!("Unexpected Outputs: {:?}", outputs);
        }
        println!("Part 2: Diagonstic Code = {}", outputs[0]);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn one_to_one(program: &str, input: i32) -> Result<i32, Error> {
        let prog: Program = program.parse()?;
        let mut cpu = Computer::new(prog);

        cpu.feed(input);
        Ok(cpu.follow().next().unwrap())
    }

    #[test]
    fn example_day5_part2() {
        assert_eq!(one_to_one("3,9,8,9,10,9,4,9,99,-1,8", 1).unwrap(), 0);
        assert_eq!(one_to_one("3,9,8,9,10,9,4,9,99,-1,8", 8).unwrap(), 1);

        assert_eq!(one_to_one("3,9,7,9,10,9,4,9,99,-1,8", 1).unwrap(), 1);
        assert_eq!(one_to_one("3,9,7,9,10,9,4,9,99,-1,8", 8).unwrap(), 0);

        assert_eq!(one_to_one("3,3,1108,-1,8,3,4,3,99", 1).unwrap(), 0);
        assert_eq!(one_to_one("3,3,1108,-1,8,3,4,3,99", 8).unwrap(), 1);

        assert_eq!(one_to_one("3,3,1107,-1,8,3,4,3,99", 1).unwrap(), 1);
        assert_eq!(one_to_one("3,3,1107,-1,8,3,4,3,99", 8).unwrap(), 0);

        assert_eq!(
            one_to_one("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9", 0).unwrap(),
            0
        );
        assert_eq!(
            one_to_one("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9", 5).unwrap(),
            1
        );

        assert_eq!(
            one_to_one("3,3,1105,-1,9,1101,0,0,12,4,12,99,1", 0).unwrap(),
            0
        );
        assert_eq!(
            one_to_one("3,3,1105,-1,9,1101,0,0,12,4,12,99,1", 5).unwrap(),
            1
        );

        let program = "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,
        1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,
        999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99";

        assert_eq!(one_to_one(program, -4).unwrap(), 999);
        assert_eq!(one_to_one(program, 8).unwrap(), 1000);
        assert_eq!(one_to_one(program, 19).unwrap(), 1001);
    }
}
