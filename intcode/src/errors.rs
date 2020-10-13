use thiserror::Error;

use crate::opcode::ParameterMode;
use crate::IntMem;

#[derive(Error, Debug)]
pub enum IntcodeError {
    #[error("Invalid program counter position: {0}")]
    InvalidPosition(IntMem),

    #[error("Unknown opcode: {0}")]
    UnknownOpcode(IntMem),

    #[error("Invalid Parameter Mode for opcode: {0}, parameter: {1}")]
    InvalidParameterMode(IntMem, u32),

    #[error("Illegal parameter mode {0:?}")]
    IllegalParameterMode(ParameterMode),

    #[error("Missing Parameters for offset {0} at position {1}")]
    MissingParameters(IntMem, IntMem),

    #[error("Invalid address {0}")]
    InvalidAddress(IntMem),

    #[error("No input avaialbe")]
    NoInput,
}

pub type Result<T> = ::std::result::Result<T, IntcodeError>;
