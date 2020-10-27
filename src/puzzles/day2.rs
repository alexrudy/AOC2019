use anyhow::Error;
use intcode::{Computer, IntMem};
use std::io::{BufRead, BufReader, Read};

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let reader = BufReader::new(input);

    let program = {
        let mut v = Vec::new();
        for line in reader.lines() {
            let elements = line?
                .split(",")
                .map(|element| element.parse::<IntMem>())
                .collect::<Result<Vec<IntMem>, std::num::ParseIntError>>()?;
            v.extend(elements)
        }
        v
    };

    let mut part1 = program.clone();
    part1[1] = 12;
    part1[2] = 2;

    let mut cpu = Computer::new(part1);
    cpu.run()?;

    let value = cpu.program().get(0).expect("Program had no value 0");
    println!("Part 1: Register 0 = {}", value);

    for noun in 0..100 {
        for verb in 0..100 {
            if trial(&program, noun, verb) == Some(19690720) {
                let input = 100 * noun + verb;
                println!("Part 2: Input = {}", input);
                return Ok(());
            }
        }
    }

    Ok(())
}

fn trial(program: &Vec<IntMem>, noun: IntMem, verb: IntMem) -> Option<IntMem> {
    let mut part2 = program.clone();
    part2[1] = noun;
    part2[2] = verb;

    let mut cpu = Computer::new(part2);
    cpu.run().ok()?;
    cpu.program().get(0)
}

#[cfg(test)]
mod test {
    use super::*;

    use intcode::Arguments;
    use intcode::IntcodeError;

    #[test]
    fn example_program_day2() {
        let program = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];

        let mut cpu = Computer::new(program);
        cpu.run().unwrap();

        let state = cpu.program();
        assert_eq!(state.get(0), Some(3500));
        assert_eq!(state.get(3), Some(70))
    }

    fn transform(program: Vec<IntMem>) -> Result<Vec<IntMem>, IntcodeError> {
        let mut cpu = Computer::new(program);
        cpu.run()?;

        Ok(cpu.program().tape())
    }

    #[test]
    fn example_cases_day2() {
        assert_eq!(
            transform(vec![1, 0, 0, 0, 99]).unwrap(),
            vec![2, 0, 0, 0, 99]
        );
        assert_eq!(
            transform(vec![2, 3, 0, 3, 99]).unwrap(),
            vec![2, 3, 0, 6, 99]
        );
        assert_eq!(
            transform(vec![2, 4, 4, 5, 99, 0]).unwrap(),
            vec![2, 4, 4, 5, 99, 9801]
        );
        assert_eq!(
            transform(vec![1, 1, 1, 4, 99, 5, 6, 0, 99]).unwrap(),
            vec![30, 1, 1, 4, 2, 5, 6, 0, 99]
        );
    }
}
