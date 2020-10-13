use thiserror::Error;

use crate::opcode::{OpCode, ParameterMode};

#[derive(Error, Debug)]
pub enum IntcodeError {
    #[error("Invalid program counter position: {0}")]
    InvalidPosition(i32),

    #[error("Unknown opcode: {0}")]
    UnknownOpcode(i32),

    #[error("Invalid Parameter Mode for opcode: {0}, parameter: {1}")]
    InvalidParameterMode(i32, u32),

    #[error("Illegal parameter mode {0:?} for argument {1} to opcode {2:?}")]
    IllegalParameterMode(ParameterMode, u32, OpCode),

    #[error("Missing Parameters for offset {0} at position {1}")]
    MissingParameters(i32, i32),

    #[error("Invalid address {0}")]
    InvalidAddress(i32),

    #[error("No input avaialbe")]
    NoInput,
}

pub type Result<T> = ::std::result::Result<T, IntcodeError>;
