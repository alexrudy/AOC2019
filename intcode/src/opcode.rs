// Implement Opcodes for Intcode

use crate::errors::{IntcodeError, Result};

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
    Halt,
}

impl Op {
    fn from_code(code: i32) -> Result<Self> {
        match code % 100 {
            1 => Ok(Op::Add),
            2 => Ok(Op::Mul),
            3 => Ok(Op::Input),
            4 => Ok(Op::Output),
            5 => Ok(Op::JumpIfTrue),
            6 => Ok(Op::JumpIfFalse),
            7 => Ok(Op::LessThan),
            8 => Ok(Op::EqualTo),
            99 => Ok(Op::Halt),
            _ => Err(IntcodeError::UnknownOpcode(code)),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OpCode(i32);

impl OpCode {
    pub(crate) fn new(code: i32) -> Result<Self> {
        Op::from_code(code)?;
        Ok(OpCode(code))
    }

    pub(crate) fn op(&self) -> Op {
        Op::from_code(self.0).unwrap()
    }

    pub(crate) fn mode(&self, parameter: u32) -> Result<ParameterMode> {
        let modulo = 10 * (10i32.pow(parameter));
        match (self.0 / modulo) % 10 {
            0 => Ok(ParameterMode::Position),
            1 => Ok(ParameterMode::Immediate),
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
            Op::Halt => 1,
        }
    }

    #[cfg(test)]
    fn modes(&self) -> Result<Vec<ParameterMode>> {
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
