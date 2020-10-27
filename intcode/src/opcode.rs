// Implement Opcodes for Intcode

use std::fmt;

use crate::errors::{IntcodeError, Result};
use crate::{Computer, IntMem};

#[derive(Debug)]
pub(crate) struct OpCodeOutputInfo {
    pub(crate) output: IntMem,
    pub(crate) advance: IntMem,
}

#[derive(Debug)]
pub(crate) enum OpCodeResult {
    Advance(IntMem),
    Output(OpCodeOutputInfo),
    Jump(IntMem),
    Halt,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Op {
    Add,
    Mul,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    EqualTo,
    MoveStack,
    Halt,
}

impl Op {
    fn from_code(code: IntMem) -> Result<Self> {
        match code % 100 {
            1 => Ok(Op::Add),
            2 => Ok(Op::Mul),
            3 => Ok(Op::Input),
            4 => Ok(Op::Output),
            5 => Ok(Op::JumpIfTrue),
            6 => Ok(Op::JumpIfFalse),
            7 => Ok(Op::LessThan),
            8 => Ok(Op::EqualTo),
            9 => Ok(Op::MoveStack),
            99 => Ok(Op::Halt),
            _ => Err(IntcodeError::UnknownOpcode(code)),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => write!(f, "Add"),
            Op::Mul => write!(f, "Mul"),
            Op::Input => write!(f, "Inp"),
            Op::Output => write!(f, "Out"),
            Op::JumpIfTrue => write!(f, "Jit"),
            Op::JumpIfFalse => write!(f, "Jif"),
            Op::LessThan => write!(f, "Clt"),
            Op::EqualTo => write!(f, "Ceq"),
            Op::MoveStack => write!(f, "Msp"),
            Op::Halt => write!(f, "Hlt"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OpCode(IntMem);

impl OpCode {
    pub(crate) fn new(code: IntMem) -> Result<Self> {
        Op::from_code(code)?;
        Ok(OpCode(code))
    }

    pub(crate) fn op(&self) -> Op {
        Op::from_code(self.0).unwrap()
    }

    pub(crate) fn mode(&self, parameter: u32) -> Result<ParameterMode> {
        let modulo = 10 * (10i64.pow(parameter));
        match (self.0 / modulo) % 10 {
            0 => Ok(ParameterMode::Position),
            1 => Ok(ParameterMode::Immediate),
            2 => Ok(ParameterMode::Relative),
            _ => Err(IntcodeError::InvalidParameterMode(self.0, parameter)),
        }
    }

    pub(crate) fn n_arguments(&self) -> u32 {
        match self.op() {
            Op::Add => 4,
            Op::Mul => 4,
            Op::Input => 2,
            Op::Output => 2,
            Op::JumpIfTrue => 3,
            Op::JumpIfFalse => 3,
            Op::LessThan => 4,
            Op::EqualTo => 4,
            Op::MoveStack => 2,
            Op::Halt => 1,
        }
    }

    pub(crate) fn operate(&self, cpu: &mut Computer) -> Result<OpCodeResult> {
        match self.op() {
            Op::Add => self.add(cpu),
            Op::Mul => self.mul(cpu),
            Op::Input => self.input(cpu),
            Op::Output => self.output(cpu),
            Op::JumpIfTrue => self.jump_if_true(cpu),
            Op::JumpIfFalse => self.jump_if_false(cpu),
            Op::LessThan => self.less_than(cpu),
            Op::EqualTo => self.equal_to(cpu),
            Op::MoveStack => self.move_stack(cpu),
            Op::Halt => Ok(OpCodeResult::Halt),
        }
    }

    fn add(&self, cpu: &mut Computer) -> Result<OpCodeResult> {
        let left = cpu.load(self, 1)?;
        let right = cpu.load(self, 2)?;
        cpu.save(self, 3, left + right)?;

        Ok(OpCodeResult::Advance(self.n_arguments() as IntMem))
    }

    fn mul(&self, cpu: &mut Computer) -> Result<OpCodeResult> {
        let left = cpu.load(self, 1)?;
        let right = cpu.load(self, 2)?;
        cpu.save(self, 3, left * right)?;

        Ok(OpCodeResult::Advance(self.n_arguments() as IntMem))
    }

    fn input(&self, cpu: &mut Computer) -> Result<OpCodeResult> {
        let value = cpu.input.take().ok_or(IntcodeError::NoInput)?;
        cpu.save(self, 1, value)?;

        Ok(OpCodeResult::Advance(self.n_arguments() as IntMem))
    }

    fn output(&self, cpu: &mut Computer) -> Result<OpCodeResult> {
        let value = cpu.load(self, 1)?;

        Ok(OpCodeResult::Output(OpCodeOutputInfo {
            advance: self.n_arguments() as IntMem,
            output: value,
        }))
    }

    fn jump_if_true(&self, cpu: &mut Computer) -> Result<OpCodeResult> {
        let value: IntMem = cpu.load(self, 1)?;
        if value != 0 {
            let target = cpu.load(self, 2)?;
            return Ok(OpCodeResult::Jump(target));
        }
        Ok(OpCodeResult::Advance(self.n_arguments() as IntMem))
    }

    fn jump_if_false(&self, cpu: &mut Computer) -> Result<OpCodeResult> {
        let value: IntMem = cpu.load(self, 1)?;
        if value == 0 {
            let target = cpu.load(self, 2)?;
            return Ok(OpCodeResult::Jump(target));
        }
        Ok(OpCodeResult::Advance(self.n_arguments() as IntMem))
    }

    fn less_than(&self, cpu: &mut Computer) -> Result<OpCodeResult> {
        let left = cpu.load(self, 1)?;
        let right = cpu.load(self, 2)?;
        cpu.save(self, 3, (left < right) as IntMem)?;

        Ok(OpCodeResult::Advance(self.n_arguments() as IntMem))
    }

    fn equal_to(&self, cpu: &mut Computer) -> Result<OpCodeResult> {
        let left = cpu.load(self, 1)?;
        let right = cpu.load(self, 2)?;
        cpu.save(self, 3, (left == right) as IntMem)?;

        Ok(OpCodeResult::Advance(self.n_arguments() as IntMem))
    }

    fn move_stack(&self, cpu: &mut Computer) -> Result<OpCodeResult> {
        let offset = cpu.load(self, 1)?;
        cpu.offset(offset)?;
        Ok(OpCodeResult::Advance(self.n_arguments() as IntMem))
    }

    pub(crate) fn modes(&self) -> Result<Vec<ParameterMode>> {
        use std::convert::TryInto;
        let n = self.n_arguments();
        let mut modes = Vec::with_capacity((n - 1).try_into().unwrap());
        for i in 1..n {
            modes.push(self.mode(i)?);
        }
        Ok(modes)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ParameterMode {
    Position,
    Immediate,
    Relative,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_modes() {
        assert_eq!(
            OpCode(1101).modes().unwrap(),
            vec![
                ParameterMode::Immediate,
                ParameterMode::Immediate,
                ParameterMode::Position,
            ]
        )
    }
}
