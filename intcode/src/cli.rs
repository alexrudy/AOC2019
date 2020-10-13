use anyhow::Error;
use clap::{App, Arg};
use intcode::{Computer, Program, ProgramState};
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

    if let Some(values) = matches.values_of("input") {
        for ivalue in values {
            cpu.feed(ivalue.parse()?);
        }
    }

    while let ProgramState::Continue = cpu.op()? {
        if let Some(output) = cpu.read() {
            println!("{}", output);
        }
    }

    Ok(())
}
