use std::collections::HashMap;
use std::collections::VecDeque;

mod errors;
mod opcode;
mod program;

pub use crate::errors::{IntcodeError, Result};
pub use crate::opcode::OpCode;
use crate::opcode::{Op, ParameterMode};
pub use crate::program::Program;

#[derive(Debug)]
pub enum ProgramState {
    Continue(i32), // Contains current program counter
    Halt,
}

#[derive(Debug)]
enum OpCodeResult {
    Advance(i32),
    Jump(i32),
    Halt,
}

#[derive(Debug)]
struct Memory {
    registers: HashMap<i32, i32>,
}

impl Memory {
    pub fn new(program: Program) -> Self {
        Self {
            registers: program
                .0
                .into_iter()
                .enumerate()
                .map(|(i, v)| ((i as i32), v))
                .collect::<HashMap<i32, i32>>(),
        }
    }

    pub fn get(&self, position: i32) -> Option<i32> {
        self.registers.get(&position).map(|v| *v)
    }

    pub fn tape(&self) -> Vec<i32> {
        let largest = *self.registers.keys().max().unwrap_or(&0);

        (0..=largest)
            .map(|i| self.registers.get(&i).map(|v| *v).unwrap_or(0))
            .collect()
    }

    fn argument(&self, address: i32) -> Result<i32> {
        self.registers
            .get(&(address))
            .ok_or(IntcodeError::UninitializedMemory(address))
            .map(|v| *v)
    }

    fn load(&mut self, address: i32, mode: ParameterMode) -> Result<i32> {
        let target = self.argument(address)?;

        if mode == ParameterMode::Immediate {
            return Ok(target);
        }

        self.registers
            .get(&target)
            .ok_or(IntcodeError::UninitializedMemory(target))
            .map(|v| *v)
    }

    fn save(&mut self, address: i32, value: i32) -> Result<()> {
        let target = self.argument(address)?;
        self.registers.insert(target, value);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Computer {
    pc: i32,
    memory: Memory,
    inputs: VecDeque<i32>,
    outputs: VecDeque<i32>,
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

    pub fn run(&mut self) -> Result<()> {
        while let Ok(ProgramState::Continue(_)) = self.op() {}

        Ok(())
    }

    pub fn get(&self, position: i32) -> Option<i32> {
        self.memory.get(position)
    }

    pub fn feed(&mut self, value: i32) -> () {
        self.inputs.push_back(value);
    }

    pub fn read(&mut self) -> Option<i32> {
        self.outputs.pop_front()
    }

    pub fn tape(&self) -> Vec<i32> {
        self.memory.tape()
    }

    pub fn follow<'c>(&'c mut self) -> Follower<'c> {
        Follower { cpu: self }
    }

    pub fn op(&mut self) -> Result<ProgramState> {
        let opcode = OpCode::new(self.memory.argument(self.pc)?)?;

        let result = match opcode.op() {
            Op::Add => self.add(opcode),
            Op::Mul => self.mul(opcode),
            Op::Input => self.input(opcode),
            Op::Output => self.output(opcode),
            Op::Halt => Ok(OpCodeResult::Halt),
        };

        match result {
            Ok(OpCodeResult::Advance(n)) => {
                self.pc += n;
                Ok(ProgramState::Continue(self.pc))
            }
            Ok(OpCodeResult::Jump(t)) => {
                self.pc = t;
                Ok(ProgramState::Continue(self.pc))
            }
            Ok(OpCodeResult::Halt) => Ok(ProgramState::Halt),
            Err(e) => Err(e),
        }
    }

    fn load(&mut self, opcode: OpCode, parameter: u32) -> Result<i32> {
        self.memory
            .load(self.pc + (parameter as i32), opcode.mode(parameter)?)
    }

    fn save(&mut self, opcode: OpCode, parameter: u32, value: i32) -> Result<()> {
        if let ParameterMode::Immediate = opcode.mode(parameter)? {
            return Err(IntcodeError::IllegalParameterMode(
                ParameterMode::Immediate,
                parameter,
                opcode,
            ));
        }

        self.memory.save(self.pc + (parameter as i32), value)?;

        Ok(())
    }

    fn add(&mut self, opcode: OpCode) -> Result<OpCodeResult> {
        let left = self.load(opcode, 1)?;
        let right = self.load(opcode, 2)?;
        self.save(opcode, 3, left + right)?;

        Ok(OpCodeResult::Advance(4))
    }

    fn mul(&mut self, opcode: OpCode) -> Result<OpCodeResult> {
        let left = self.load(opcode, 1)?;
        let right = self.load(opcode, 2)?;
        self.save(opcode, 3, left * right)?;

        Ok(OpCodeResult::Advance(opcode.n_arguments() as i32))
    }

    fn input(&mut self, opcode: OpCode) -> Result<OpCodeResult> {
        let value = self.inputs.pop_front().ok_or(IntcodeError::NoInput)?;
        self.save(opcode, 1, value)?;

        Ok(OpCodeResult::Advance(2))
    }

    fn output(&mut self, opcode: OpCode) -> Result<OpCodeResult> {
        let value = self.load(opcode, 1)?;
        self.outputs.push_back(value);
        Ok(OpCodeResult::Advance(opcode.n_arguments() as i32))
    }
}

pub struct Follower<'c> {
    cpu: &'c mut Computer,
}

impl<'c> Iterator for Follower<'c> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        while let Ok(ProgramState::Continue(_)) = self.cpu.op() {
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

    fn transform(program: Vec<i32>) -> Result<Vec<i32>> {
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
}
