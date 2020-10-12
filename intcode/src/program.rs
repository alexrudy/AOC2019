use anyhow::Error;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Program(pub(crate) Vec<i32>);

#[derive(Debug, Error)]
pub enum ParseProgramError {
    #[error("Failed to parse integer {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl FromStr for Program {
    type Err = ParseProgramError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut v = Vec::new();
        for line in s.lines() {
            let elements = line
                .trim()
                .trim_end_matches(',')
                .split(",")
                .map(|element| element.trim())
                .map(|element| element.parse::<i32>())
                .collect::<Result<Vec<i32>, std::num::ParseIntError>>()?;
            v.extend(elements)
        }
        Ok(Program(v))
    }
}

impl Program {
    pub fn read(reader: Box<dyn Read + 'static>) -> Result<Self, Error> {
        let bufread = BufReader::new(reader);
        let mut v = Vec::new();
        for line in bufread.lines() {
            let elements = line?
                .trim()
                .split(",")
                .map(|element| element.trim())
                .map(|element| element.parse::<i32>())
                .collect::<Result<Vec<i32>, std::num::ParseIntError>>()?;
            v.extend(elements)
        }
        Ok(v.into())
    }
}

impl Into<Program> for Vec<i32> {
    fn into(self) -> Program {
        Program(self)
    }
}
