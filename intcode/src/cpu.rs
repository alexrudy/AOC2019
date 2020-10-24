use std::collections::HashMap;

pub use crate::errors::{IntcodeError, Result};
pub use crate::opcode::OpCode;
use crate::opcode::{OpCodeResult, ParameterMode};
pub use crate::program::Program;
use crate::IntMem;

#[derive(Debug, Eq, PartialEq)]
pub enum CPUState {
    Continue,
    Output(IntMem),
    Input,
    Halt,
}

impl CPUState {
    fn is_continue(&self) -> bool {
        !self.is_halt()
    }

    fn is_halt(&self) -> bool {
        *self == CPUState::Halt
    }
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
    pub(crate) input: Option<IntMem>,
}

impl Computer {
    pub fn new<P: Into<Program>>(program: P) -> Self {
        Computer {
            pc: 0,
            memory: Memory::new(program.into()),
            input: None,
        }
    }

    pub fn run(&mut self) -> Result<CPUState> {
        while let CPUState::Continue = self.op()? {}

        // Return the last opcode result
        self.op()
    }

    pub fn get(&self, position: IntMem) -> Option<IntMem> {
        self.memory.get(position)
    }

    pub fn feed(&mut self, value: IntMem) -> Result<()> {
        match self.input.replace(value) {
            Some(_) => Err(IntcodeError::InputAlreadyPresent),
            None => Ok(()),
        }
    }

    pub fn tape(&self) -> Vec<IntMem> {
        self.memory.tape()
    }

    pub fn follow<'c>(&'c mut self) -> Follower<'c> {
        Follower { cpu: self }
    }

    pub fn op(&mut self) -> Result<CPUState> {
        let opcode = OpCode::new(self.memory.argument(self.pc)?)?;

        let result = opcode.operate(self);

        match result {
            Ok(OpCodeResult::Advance(n)) => {
                self.pc += n;
                Ok(CPUState::Continue)
            }
            Ok(OpCodeResult::Output(o)) => {
                self.pc += o.advance;
                Ok(CPUState::Output(o.output))
            }
            Ok(OpCodeResult::Jump(t)) => {
                self.pc = t;
                Ok(CPUState::Continue)
            }
            Ok(OpCodeResult::Halt) => Ok(CPUState::Halt),
            Err(IntcodeError::NoInput) => Ok(CPUState::Input),
            Err(e) => Err(e),
        }
    }

    pub(crate) fn offset(&mut self, offset: IntMem) -> Result<()> {
        self.memory.offset(offset)
    }

    pub(crate) fn load(&mut self, opcode: &OpCode, parameter: u32) -> Result<IntMem> {
        self.memory
            .load(self.pc + (parameter as IntMem), opcode.mode(parameter)?)
    }

    pub(crate) fn save(&mut self, opcode: &OpCode, parameter: u32, value: IntMem) -> Result<()> {
        self.memory.save(
            self.pc + (parameter as IntMem),
            opcode.mode(parameter)?,
            value,
        )?;

        Ok(())
    }

    pub fn simple(&mut self, input: IntMem) -> Result<IntMem> {
        self.feed(input)?;
        self.follow().one()
    }
}

pub struct Follower<'c> {
    cpu: &'c mut Computer,
}

impl<'c> Follower<'c> {
    pub fn one(&mut self) -> Result<IntMem> {
        let mut iter = self.take(2);
        let first = iter.next().ok_or(IntcodeError::NoOutput)?;
        match iter.next() {
            Some(_) => Err(IntcodeError::UnexpectedOutput)?,
            None => {}
        };
        Ok(first)
    }
}

impl<'c> Iterator for Follower<'c> {
    type Item = IntMem;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.cpu.op() {
                Ok(CPUState::Continue) => {}
                Ok(CPUState::Output(value)) => {
                    return Some(value);
                }
                Ok(CPUState::Input) => panic!("Unexpected input in CPU Follower!"),
                Ok(CPUState::Halt) => {
                    return None;
                }
                Err(error) => panic!("Unexpected error in CPU: {}", error),
            }
        }
    }
}
