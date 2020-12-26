#![feature(result_cloned)]
#![feature(concat_idents)]
#![feature(map_into_keys_values)]
#![deny(clippy::all)]

use clap::{value_t, App, Arg};

use anyhow;
use lazy_static::lazy_static;
use paste::paste;
use thiserror::Error;

use std::collections::HashMap;
use std::fs::File;
use std::io;

mod iterhelper;
pub mod puzzles;

type Error = anyhow::Error;
type Actor = Box<dyn (Fn(Box<dyn std::io::Read>) -> Result<(), Error>) + Send + Sync + 'static>;

macro_rules! day {
    ($day:tt) => {
        paste! {
            ($day, Box::new(puzzles::[<day $day>]::main))
        }
    };
}

lazy_static! {
    static ref SOLVERS: HashMap<u32, Actor> = {
        let mut s: HashMap<u32, Actor> = HashMap::new();
        let days: Vec<(u32, Actor)> = vec![
            day!(1),
            day!(2),
            day!(3),
            day!(4),
            day!(5),
            day!(6),
            day!(7),
            day!(8),
            day!(9),
            day!(10),
            day!(11),
            day!(12),
            day!(13),
            day!(14),
            day!(15),
            day!(16),
            day!(17),
            day!(18),
        ];

        for (d, func) in days.into_iter() {
            s.insert(d, func);
        }

        s
    };
}

pub fn solver() -> Result<(), Error> {
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

    let reader = get_input_reader(day, matches.value_of("input"))?;

    match SOLVERS.get(&day) {
        None => panic!("No code found for day {}", day),
        Some(actor) => actor(reader),
    }
}

type IOResult<T> = std::io::Result<T>;
type BoxedRead = Box<dyn ::std::io::Read + 'static>;

pub fn get_input_reader(day: u32, filename: Option<&str>) -> Result<BoxedRead, Error> {
    let reader: Box<dyn ::std::io::Read + 'static> = match filename {
        Some("-") => Box::new(::std::io::stdin()),
        Some(path) => {
            let f: File = File::open(path)?;
            Box::new(f)
        }
        None => get_default_input(day).map_err(|e| AoCError::DefaultInputNotFound(day, e))?,
    };
    Ok(reader)
}

pub fn get_default_input(day: u32) -> IOResult<BoxedRead> {
    let mut p = ::std::path::PathBuf::from("puzzles");
    p.push(format!("{}", day));
    p.push("input.txt");

    let f = File::open(p)?;

    Ok(Box::new(f))
}

#[derive(Debug, Error)]
pub enum AoCError {
    #[error("No module found for day {0}")]
    DayNotFound(u32),

    #[error("Input not found: puzzles/{0}/input.txt")]
    DefaultInputNotFound(u32, #[source] io::Error),

    #[error("Input not found")]
    InputNotFound(#[from] io::Error),
}
