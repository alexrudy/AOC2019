#![feature(concat_idents)]

use clap::{value_t, App, Arg};

use anyhow;
use lazy_static::lazy_static;
use thiserror::Error;

use std::collections::HashMap;
use std::fs::File;
use std::io;

mod cartesian;
mod puzzles;

type Error = anyhow::Error;
type Actor = Box<dyn (Fn(Box<dyn std::io::Read>) -> Result<(), Error>) + Send + Sync + 'static>;

lazy_static! {
    static ref SOLVERS: HashMap<u32, Actor> = {
        let mut s: HashMap<u32, Actor> = HashMap::new();
        s.insert(1, Box::new(puzzles::day1::main));
        s.insert(2, Box::new(puzzles::day2::main));
        s.insert(3, Box::new(puzzles::day3::main));
        s
    };
}

fn main() -> () {
    match driver() {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
    }
}

fn driver() -> Result<(), Error> {
    let matches = App::new("Advent of Code 2019")
        .version("1.0")
        .author("Alex Rudy <opensource@alexrudy.net>")
        .about("Solve Advent of Code Puzzles")
        .arg(
            Arg::with_name("day")
                .value_name("DAY")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("input")
                .value_name("INPUT")
                .required(false)
                .takes_value(true),
        )
        .get_matches();

    let day = value_t!(matches, "day", u32).unwrap();
    println!("Day {}", day);

    let filename = matches.value_of("input");
    let reader: Box<dyn ::std::io::Read + 'static> = match filename {
        Some("-") => Box::new(::std::io::stdin()),
        Some(path) => {
            let f: File = File::open(path)?;
            Box::new(f)
        }
        None => get_default_input(day).map_err(|e| AoCError::DefaultInputNotFound(day, e))?,
    };

    match SOLVERS.get(&day) {
        None => panic!("No code found for day {}", day),
        Some(actor) => actor(reader),
    }
}

fn get_default_input(day: u32) -> std::io::Result<Box<dyn ::std::io::Read>> {
    let mut p = ::std::path::PathBuf::from("puzzles");
    p.push(format!("{}", day));
    p.push("input.txt");

    let f = File::open(p)?;

    Ok(Box::new(f))
}

#[derive(Debug, Error)]
pub enum AoCError {
    #[error("No code found for day {0}")]
    DayNotFound(u32),

    #[error("Input not found: puzzles/{0}/input.txt")]
    DefaultInputNotFound(u32, #[source] io::Error),

    #[error("Input not found")]
    InputNotFound(#[from] io::Error),
}
