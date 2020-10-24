use anyhow::{anyhow, Error};
use std::io::Read;

use intcode::{CPUState, Computer, IntMem, Program};
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

    fn phase(&mut self, phases: &[IntMem]) -> Result<(), Error> {
        if phases.len() != self.amplifiers.len() {
            return Err(anyhow!("Invalid number of phases"));
        }

        for (amp, &phase) in self.amplifiers.iter_mut().zip(phases.iter()) {
            amp.cpu.feed(phase)?;
        }

        Ok(())
    }

    fn run(&mut self) -> Result<IntMem, Error> {
        let mut signal = 0;
        for amp in self.amplifiers.iter_mut() {
            loop {
                match amp.cpu.op()? {
                    CPUState::Input => amp.cpu.feed(signal)?,
                    CPUState::Continue => {}
                    CPUState::Output(v) => {
                        signal = v;
                        break;
                    }
                    CPUState::Halt => return Err(anyhow!("Expected output from CPU!")),
                }
            }
        }
        Ok(signal)
    }

    fn feedback_loop(&mut self) -> Result<IntMem, Error> {
        let mut last_ouput = None;
        let n = self.amplifiers.len();
        let mut input = 0;
        loop {
            for (i, amp) in self.amplifiers.iter_mut().enumerate() {
                loop {
                    match amp.cpu.op()? {
                        CPUState::Output(value) => {
                            input = value;
                            if i == n - 1 {
                                last_ouput = Some(value);
                            }
                            break;
                        }
                        CPUState::Halt => break,
                        CPUState::Continue => {}
                        CPUState::Input => {
                            amp.cpu.feed(input)?;
                        }
                    }
                }
            }
            if self
                .amplifiers
                .iter_mut()
                .all(|amp| amp.cpu.run().unwrap() == CPUState::Halt)
            {
                break;
            }
        }
        last_ouput.ok_or(anyhow!("No final output produced"))
    }
}

fn find_best_phase(n: usize, program: Program) -> Result<IntMem, Error> {
    let mut phases: Vec<IntMem> = (0..(n as IntMem)).collect();

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

fn find_best_phase_with_feedback(n: usize, program: Program) -> Result<IntMem, Error> {
    let mut phases: Vec<IntMem> = (0..(n as IntMem)).map(|i| i + 5).collect();

    let signal = Heap::new(&mut phases)
        .map(|p| {
            let mut chain = AmplifierChain::from_program(n, program.clone());
            chain.phase(&p).unwrap();
            chain.feedback_loop().unwrap()
        })
        .max_by_key(|signal| *signal);

    match signal {
        Some(s) => Ok(s),
        None => Err(anyhow!("No signal returned!")),
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let program = Program::read(input)?;

    let signal = find_best_phase(5, program.clone())?;
    println!("Part 1: Best signal is {}", signal);

    let signal = find_best_phase_with_feedback(5, program.clone())?;
    println!("Part 2: Best signal with feedback is {}", signal);

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

    #[test]
    fn examples_part2() {
        assert_eq!(
            find_best_phase_with_feedback(
                5,
                "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,
                27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5"
                    .parse()
                    .unwrap()
            )
            .unwrap(),
            139629729
        );

        assert_eq!(
            find_best_phase_with_feedback(
                5,
                "3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,
                -5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,
                53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10"
                    .parse()
                    .unwrap()
            )
            .unwrap(),
            18216
        );
    }
}
