use anyhow::{anyhow, Error};
use std::io::Read;

use intcode::{Computer, Program};
use permutohedron::Heap;

#[derive(Debug)]
struct Amplifier {
    cpu: Computer,
}

#[derive(Debug)]
struct AmplifierChain {
    amplifiers: Vec<Amplifier>,
}

impl AmplifierChain {
    fn from_program(n: usize, program: Program) -> Self {
        let mut amps = Vec::with_capacity(n);
        for _ in 0..n {
            amps.push(Amplifier {
                cpu: Computer::new(program.clone()),
            })
        }
        Self { amplifiers: amps }
    }

    fn phase(&mut self, phases: &[i32]) -> Result<(), Error> {
        if phases.len() != self.amplifiers.len() {
            return Err(anyhow!("Invalid number of phases"));
        }

        for (amp, &phase) in self.amplifiers.iter_mut().zip(phases.iter()) {
            amp.cpu.feed(phase);
        }

        Ok(())
    }

    fn run(&mut self) -> Result<i32, Error> {
        let mut signal = 0;
        for amp in self.amplifiers.iter_mut() {
            amp.cpu.feed(signal);
            signal = amp
                .cpu
                .follow()
                .nth(0)
                .ok_or(anyhow!("Amplifier returned no output!"))?;
        }
        Ok(signal)
    }
}

fn find_best_phase(n: usize, program: Program) -> Result<i32, Error> {
    let mut phases: Vec<i32> = (0..(n as i32)).collect();

    let signal = Heap::new(&mut phases)
        .map(|p| {
            let mut chain = AmplifierChain::from_program(n, program.clone());
            chain.phase(&p).unwrap();
            chain.run().unwrap()
        })
        .max_by_key(|signal| *signal);

    match signal {
        Some(s) => Ok(s),
        None => Err(anyhow!("No signal returned!")),
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let program = Program::read(input)?;

    let signal = find_best_phase(5, program)?;
    println!("Part 1: Best signal is {}", signal);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples_part1() {
        let mut ac = AmplifierChain::from_program(
            5,
            "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0"
                .parse()
                .unwrap(),
        );
        ac.phase(&[4, 3, 2, 1, 0]).unwrap();
        assert_eq!(ac.run().unwrap(), 43210);

        assert_eq!(
            find_best_phase(
                5,
                "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0"
                    .parse()
                    .unwrap()
            )
            .unwrap(),
            43210
        );
        assert_eq!(
            find_best_phase(
                5,
                "3,23,3,24,1002,24,10,24,1002,23,-1,23,
                101,5,23,23,1,24,23,23,4,23,99,0,0"
                    .parse()
                    .unwrap()
            )
            .unwrap(),
            54321
        );

        assert_eq!(
            find_best_phase(
                5,
                "3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,
                1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0"
                    .parse()
                    .unwrap()
            )
            .unwrap(),
            65210
        );
    }
}
