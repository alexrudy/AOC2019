use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntcodeError {
    #[error("Invalid program counter position: {0}")]
    InvalidPosition(i32),
    #[error("Unknown opcode: {0}")]
    UnknownOpcode(i32),

    #[error("Missing Parameters for offset {0} at position {1}")]
    MissingParameters(i32, i32),
}

#[derive(Debug)]
enum ProgramState {
    Continue,
    Halt,
}

#[derive(Debug)]
pub struct Computer {
    pc: i32,
    registers: HashMap<i32, i32>,
}

impl Computer {
    pub fn new(program: Vec<i32>) -> Self {
        Computer {
            pc: 0,
            registers: program
                .into_iter()
                .enumerate()
                .map(|(i, v)| ((i as i32), v))
                .collect::<HashMap<i32, i32>>(),
        }
    }

    pub fn run(&mut self) -> Result<(), IntcodeError> {
        while let Ok(ProgramState::Continue) = self.op() {}

        Ok(())
    }

    fn op(&mut self) -> Result<ProgramState, IntcodeError> {
        let opcode = self
            .registers
            .get(&self.pc)
            .ok_or(IntcodeError::InvalidPosition(self.pc))?;

        match opcode {
            1 => self.add().map(|_| ProgramState::Continue),
            2 => self.mul().map(|_| ProgramState::Continue),
            99 => Ok(ProgramState::Halt),
            _ => Err(IntcodeError::UnknownOpcode(*opcode)),
        }
    }

    pub fn tape(&self) -> Vec<i32> {
        let largest = *self.registers.keys().max().unwrap_or(&0);

        (0..=largest)
            .map(|i| self.registers.get(&i).map(|v| *v).unwrap_or(0))
            .collect()
    }

    pub fn get(&self, position: i32) -> Option<i32> {
        self.registers.get(&position).map(|v| *v)
    }

    fn argument(&self, offset: i32) -> Result<i32, IntcodeError> {
        self.registers
            .get(&(self.pc + offset))
            .ok_or(IntcodeError::MissingParameters(offset, self.pc))
            .map(|v| *v)
    }

    fn load(&mut self, offset: i32) -> Result<i32, IntcodeError> {
        let target = self.argument(offset)?;

        self.registers
            .get(&target)
            .ok_or(IntcodeError::InvalidPosition(target))
            .map(|v| *v)
    }

    fn save(&mut self, offset: i32, value: i32) -> Result<(), IntcodeError> {
        let target = self.argument(offset)?;
        self.registers.insert(target, value);
        Ok(())
    }

    fn add(&mut self) -> Result<(), IntcodeError> {
        let left = self.load(1)?;
        let right = self.load(2)?;
        self.save(3, left + right)?;

        self.pc += 4;
        Ok(())
    }

    fn mul(&mut self) -> Result<(), IntcodeError> {
        let left = self.load(1)?;
        let right = self.load(2)?;
        self.save(3, left * right)?;

        self.pc += 4;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn example_program_day2() {
        let program = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];

        let mut cpu = Computer::new(program);
        cpu.run().unwrap();

        assert_eq!(cpu.get(0), Some(3500));
        assert_eq!(cpu.get(3), Some(70))
    }

    fn transform(program: Vec<i32>) -> Result<Vec<i32>, IntcodeError> {
        let mut cpu = Computer::new(program);
        cpu.run()?;

        Ok(cpu.tape())
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
