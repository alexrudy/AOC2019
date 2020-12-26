use anyhow::Error;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;
use thiserror::Error;

pub use crate::errors::{IntcodeError, Result};
use crate::opcode::{OpCode, ParameterMode};
use crate::IntMem;

type AnyResult<T> = std::result::Result<T, Error>;

pub trait Arguments: Sized {
    fn argument(&self, address: IntMem) -> Result<IntMem>;
    fn tape(&self) -> Vec<IntMem>;
    fn len(&self) -> IntMem;

    fn instruction(&self, address: IntMem) -> AnyResult<Instruction> {
        let op = OpCode::new(self.argument(address)?)?;
        let n = op.n_arguments();
        let mut args = Vec::with_capacity((n - 1).try_into()?);

        for i in (1 as IntMem)..(n as IntMem) {
            args.push(self.argument(address + i)?);
        }
        Ok(Instruction {
            opcode: op,
            arguments: args,
        })
    }

    fn assembly(&self) -> Assembly<Self> {
        Assembly { program: &self }
    }
}

#[derive(Debug, Clone)]
pub struct Program(HashMap<IntMem, IntMem>);

#[derive(Debug, Error)]
pub enum ParseProgramError {
    #[error("Failed to parse integer {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl FromStr for Program {
    type Err = ParseProgramError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut v = Vec::new();
        for line in s.lines() {
            let elements = line
                .trim()
                .trim_end_matches(',')
                .split(",")
                .map(|element| element.trim())
                .map(|element| element.parse::<IntMem>())
                .collect::<std::result::Result<Vec<IntMem>, std::num::ParseIntError>>()?;
            v.extend(elements)
        }
        Ok(v.into())
    }
}

impl Program {
    pub fn read(reader: Box<dyn Read + 'static>) -> AnyResult<Self> {
        let bufread = BufReader::new(reader);
        let mut v = Vec::new();
        for line in bufread.lines() {
            let elements = line?
                .trim()
                .split(",")
                .map(|element| element.trim())
                .map(|element| element.parse::<IntMem>())
                .collect::<std::result::Result<Vec<IntMem>, std::num::ParseIntError>>()?;
            v.extend(elements)
        }
        Ok(v.into())
    }

    pub fn get(&self, address: IntMem) -> Option<IntMem> {
        self.0.get(&address).copied()
    }

    pub fn insert(&mut self, address: IntMem, value: IntMem) -> Result<()> {
        self.0.insert(address, value);
        Ok(())
    }
}

impl Arguments for Program {
    fn argument(&self, address: IntMem) -> Result<IntMem> {
        self.0
            .get(&address)
            .ok_or(IntcodeError::InvalidAddress(address))
            .map(|arg| *arg)
    }

    fn tape(&self) -> Vec<IntMem> {
        let largest = *self.0.keys().max().unwrap_or(&0);

        (0..=largest)
            .map(|i| self.0.get(&i).map(|v| *v).unwrap_or(0))
            .collect()
    }

    fn len(&self) -> IntMem {
        self.0.len() as IntMem
    }
}

impl Into<Program> for Vec<IntMem> {
    fn into(self) -> Program {
        Program(
            self.iter()
                .enumerate()
                .map(|(i, v)| (i as IntMem, *v))
                .collect(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Memory {
    stack_pointer: IntMem,
    registers: Program,
}

impl Memory {
    pub fn new(program: Program) -> Self {
        Self {
            stack_pointer: 0,
            registers: program,
        }
    }

    pub fn program(self) -> Program {
        self.registers
    }

    pub fn argument(&self, position: IntMem) -> Result<IntMem> {
        self.registers.argument(position)
    }

    pub fn offset(&mut self, value: IntMem) -> Result<()> {
        self.stack_pointer += value;
        Ok(())
    }

    // Load a memory value from an address, with a parameter mode. Parameter modes
    // allow memory to be loaded immediately (as the literal value), by position
    // (where addresses must be positive integers), or relatively (relative to
    // the stack pointer).
    pub fn load(&mut self, address: IntMem, mode: ParameterMode) -> Result<IntMem> {
        let target = self.registers.argument(address)?;

        match (mode, target) {
            (ParameterMode::Immediate, t) => Ok(t),
            (ParameterMode::Position, a) if a < 0 => Err(IntcodeError::InvalidAddress(a)),
            (ParameterMode::Position, a) => Ok(self.registers.get(a).unwrap_or(0)),
            (ParameterMode::Relative, r) => {
                Ok(self.registers.get(r + self.stack_pointer).unwrap_or(0))
            }
        }
    }

    pub fn save(&mut self, address: IntMem, mode: ParameterMode, value: IntMem) -> Result<()> {
        let target = self.registers.argument(address)?;

        match (mode, target) {
            (ParameterMode::Immediate, _) => Err(IntcodeError::IllegalParameterMode(mode)),
            (ParameterMode::Position, a) if a < 0 => Err(IntcodeError::InvalidAddress(a)),
            (ParameterMode::Position, a) => {
                self.registers.insert(a, value)?;
                Ok(())
            }
            (ParameterMode::Relative, r) => {
                self.registers.insert(r + self.stack_pointer, value)?;
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    opcode: OpCode,
    arguments: Vec<IntMem>,
}

impl Instruction {
    pub fn n_arguments(&self) -> u32 {
        self.opcode.n_arguments()
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.opcode.op())?;

        // TODO: The unwrap is unsatisfying here -- opcode should always be valid once built.
        for (arg, mode) in self.arguments.iter().zip(self.opcode.modes().unwrap()) {
            write!(f, ",")?;
            match mode {
                ParameterMode::Immediate => write!(f, "{}", arg)?,
                ParameterMode::Position => write!(f, "&{}", arg)?,
                ParameterMode::Relative => write!(f, "${}", arg)?,
            }
        }

        Ok(())
    }
}

pub struct Assembly<'p, T>
where
    T: Arguments + Sized,
{
    program: &'p T,
}

impl<'p, T> fmt::Display for Assembly<'p, T>
where
    T: Arguments,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut pc = 0;
        let mut lines = 1;

        while pc < self.program.len() {
            if let Ok(instruction) = self.program.instruction(pc) {
                writeln!(f, "[{:04}|{:04}] {}", lines, pc, instruction)?;
                pc += instruction.n_arguments() as IntMem;
            } else {
                writeln!(
                    f,
                    "[{:04}|{:04}] {}",
                    lines,
                    pc,
                    self.program.argument(pc).expect("Opcode")
                )?;
                pc += 1;
            }
            lines += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_instruction() {
        let mem: Program = vec![3, 0, 4, 0, 99].into();

        assert_eq!(format!("{}", mem.instruction(0).unwrap()), "Inp,&0");
        assert_eq!(format!("{}", mem.instruction(2).unwrap()), "Out,&0");
        assert_eq!(format!("{}", mem.instruction(4).unwrap()), "Hlt");
    }

    #[test]
    fn display_prorgam() {
        let mem: Program = vec![3, 0, 4, 0, 99].into();
        assert_eq!(format!("{}", mem.assembly()), "Inp,&0\nOut,&0\nHlt\n");
    }
}
