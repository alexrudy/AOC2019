use anyhow::{anyhow, Error};
use clap::{App, Arg};
use intcode::{CPUState, Computer, IntMem, Program};
use std::collections::VecDeque;
use std::fs::File;

type Result<T> = std::result::Result<T, Error>;

fn program(filename: Option<&str>) -> Result<Program> {
    let reader: Box<dyn ::std::io::Read + 'static> = match filename {
        Some("-") => Box::new(::std::io::stdin()),
        Some(path) => {
            let f: File = File::open(path)?;
            Box::new(f)
        }
        None => Box::new(::std::io::stdin()),
    };

    Ok(Program::read(reader)?)
}

fn main() -> Result<()> {
    let matches = App::new("Advent of Code 2019")
        .version("1.0")
        .author("Alex Rudy <opensource@alexrudy.net>")
        .about("Execute Intcode Programs")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .required(false)
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("program")
                .value_name("PROGRAM")
                .required(false)
                .takes_value(true)
                .index(1),
        )
        .get_matches();

    let filename = matches.value_of("program");
    let prog = program(filename)?;

    let mut cpu = Computer::new(prog);

    let mut inputs = matches
        .values_of("input")
        .map(|v| {
            v.map(|i| i.parse::<IntMem>())
                .collect::<std::result::Result<VecDeque<IntMem>, std::num::ParseIntError>>()
        })
        .transpose()?;

    loop {
        match cpu.op()? {
            CPUState::Continue => {}
            CPUState::Output(v) => println!("{}", v),
            CPUState::Halt => break,
            CPUState::Input => match inputs {
                Some(ref mut buffer) => {
                    cpu.feed(
                        buffer
                            .pop_front()
                            .ok_or(anyhow!("Another input value is required!"))?,
                    )?;
                }
                None => return Err(anyhow!("An input value is required!")),
            },
        }
    }

    Ok(())
}
