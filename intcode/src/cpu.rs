pub use crate::errors::{IntcodeError, Result};
pub use crate::opcode::OpCode;
use crate::opcode::OpCodeResult;
pub use crate::program::{Arguments, Memory, Program};
use crate::IntMem;

#[derive(Debug, Eq, PartialEq)]
pub enum CPUState {
    Continue,
    Output(IntMem),
    Input,
    Halt,
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
        loop {
            let state = self.op()?;

            if state != CPUState::Continue {
                return Ok(state);
            }
        }
    }

    pub fn program(self) -> Program {
        self.memory.program()
    }

    pub fn feed(&mut self, value: IntMem) -> Result<()> {
        match self.input.replace(value) {
            Some(_) => Err(IntcodeError::InputAlreadyPresent),
            None => Ok(()),
        }
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
