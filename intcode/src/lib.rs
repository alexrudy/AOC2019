use std::collections::HashMap;
use std::collections::VecDeque;

mod errors;
mod opcode;
mod program;

pub use crate::errors::{IntcodeError, Result};
pub use crate::opcode::OpCode;
use crate::opcode::{OpCodeResult, ParameterMode};
pub use crate::program::Program;

pub type IntMem = i64;

#[derive(Debug, Eq, PartialEq)]
pub enum ProgramState {
    Continue,
    Input,
    Halt,
}

#[derive(Debug)]
struct Memory {
    stack_pointer: IntMem,
    registers: HashMap<IntMem, IntMem>,
}

impl Memory {
    pub fn new(program: Program) -> Self {
        Self {
            stack_pointer: 0,
            registers: program
                .0
                .into_iter()
                .enumerate()
                .map(|(i, v)| ((i as IntMem), v))
                .collect::<HashMap<IntMem, IntMem>>(),
        }
    }

    pub fn get(&self, position: IntMem) -> Option<IntMem> {
        self.registers.get(&position).map(|v| *v)
    }

    pub fn tape(&self) -> Vec<IntMem> {
        let largest = *self.registers.keys().max().unwrap_or(&0);

        (0..=largest)
            .map(|i| self.registers.get(&i).map(|v| *v).unwrap_or(0))
            .collect()
    }

    fn offset(&mut self, value: IntMem) -> Result<()> {
        self.stack_pointer += value;
        Ok(())
    }

    fn argument(&self, address: IntMem) -> Result<IntMem> {
        if address < 0 {
            return Err(IntcodeError::InvalidAddress(address));
        }

        Ok(self.registers.get(&(address)).map(|v| *v).unwrap_or(0))
    }

    fn load(&mut self, address: IntMem, mode: ParameterMode) -> Result<IntMem> {
        let target = self.argument(address)?;

        match (mode, target) {
            (ParameterMode::Immediate, t) => Ok(t),
            (ParameterMode::Position, a) if a < 0 => Err(IntcodeError::InvalidAddress(a)),
            (ParameterMode::Position, a) => Ok(self.registers.get(&a).map(|v| *v).unwrap_or(0)),
            (ParameterMode::Relative, r) => Ok(self
                .registers
                .get(&(r + self.stack_pointer))
                .map(|v| *v)
                .unwrap_or(0)),
        }
    }

    fn save(&mut self, address: IntMem, mode: ParameterMode, value: IntMem) -> Result<()> {
        let target = self.argument(address)?;

        match (mode, target) {
            (ParameterMode::Immediate, _) => Err(IntcodeError::IllegalParameterMode(mode)),
            (ParameterMode::Position, a) if a < 0 => Err(IntcodeError::InvalidAddress(a)),
            (ParameterMode::Position, a) => {
                self.registers.insert(a, value);
                Ok(())
            }
            (ParameterMode::Relative, r) => {
                self.registers.insert(r + self.stack_pointer, value);
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct Computer {
    pc: IntMem,
    memory: Memory,
    inputs: VecDeque<IntMem>,
    outputs: VecDeque<IntMem>,
}

impl Computer {
    pub fn new<P: Into<Program>>(program: P) -> Self {
        Computer {
            pc: 0,
            memory: Memory::new(program.into()),
            inputs: VecDeque::new(),
            outputs: VecDeque::new(),
        }
    }

    pub fn run(&mut self) -> Result<ProgramState> {
        while let Ok(ProgramState::Continue) = self.op() {}

        // Return the last opcode result
        self.op()
    }

    pub fn get(&self, position: IntMem) -> Option<IntMem> {
        self.memory.get(position)
    }

    pub fn feed(&mut self, value: IntMem) -> () {
        self.inputs.push_back(value);
    }

    pub fn read(&mut self) -> Option<IntMem> {
        self.outputs.pop_front()
    }

    pub fn tape(&self) -> Vec<IntMem> {
        self.memory.tape()
    }

    pub fn follow<'c>(&'c mut self) -> Follower<'c> {
        Follower { cpu: self }
    }

    pub fn op(&mut self) -> Result<ProgramState> {
        let opcode = OpCode::new(self.memory.argument(self.pc)?)?;

        let result = opcode.operate(self);

        match result {
            Ok(OpCodeResult::Advance(n)) => {
                self.pc += n;
                Ok(ProgramState::Continue)
            }
            Ok(OpCodeResult::Jump(t)) => {
                self.pc = t;
                Ok(ProgramState::Continue)
            }
            Ok(OpCodeResult::Halt) => Ok(ProgramState::Halt),
            Err(IntcodeError::NoInput) => Ok(ProgramState::Input),
            Err(e) => Err(e),
        }
    }

    fn offset(&mut self, offset: IntMem) -> Result<()> {
        self.memory.offset(offset)
    }

    fn load(&mut self, opcode: &OpCode, parameter: u32) -> Result<IntMem> {
        self.memory
            .load(self.pc + (parameter as IntMem), opcode.mode(parameter)?)
    }

    fn save(&mut self, opcode: &OpCode, parameter: u32, value: IntMem) -> Result<()> {
        self.memory.save(
            self.pc + (parameter as IntMem),
            opcode.mode(parameter)?,
            value,
        )?;

        Ok(())
    }
}

pub struct Follower<'c> {
    cpu: &'c mut Computer,
}

impl<'c> Iterator for Follower<'c> {
    type Item = IntMem;

    fn next(&mut self) -> Option<Self::Item> {
        while let Ok(ProgramState::Continue) = self.cpu.op() {
            let output = self.cpu.read();
            if output.is_some() {
                return output;
            }
        }
        return None;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn example_program_day_2() {
        let program = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];

        let mut cpu = Computer::new(program);
        cpu.run().unwrap();

        assert_eq!(cpu.get(0), Some(3500));
        assert_eq!(cpu.get(3), Some(70))
    }

    fn transform(program: Vec<IntMem>) -> Result<Vec<IntMem>> {
        let mut cpu = Computer::new(program);
        cpu.run()?;

        Ok(cpu.tape())
    }

    #[test]
    fn example_cases_day_2() {
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

    #[test]
    fn example_case_day_5() {
        let mut cpu = Computer::new(vec![3, 0, 4, 0, 99]);
        cpu.feed(5);
        cpu.run().unwrap();
        assert_eq!(cpu.read(), Some(5));
    }

    #[test]
    fn example_case_day9() {
        let program = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];
        let mut cpu = Computer::new(program.clone());
        cpu.run().unwrap();

        assert_eq!(cpu.outputs.into_iter().collect::<Vec<IntMem>>(), program);
    }

    #[test]
    fn example_case_day9_bignum() {
        let program = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];

        let mut cpu = Computer::new(program.clone());
        cpu.run().unwrap();

        let value = cpu.read().unwrap();

        assert_eq!(format!("{}", value).len(), 16)
    }

    #[test]
    fn example_case_day9_bignum_simple() {
        let program = vec![104, 1125899906842624, 99];

        let mut cpu = Computer::new(program.clone());
        cpu.run().unwrap();

        let value = cpu.read().unwrap();

        assert_eq!(value, 1125899906842624)
    }
}
