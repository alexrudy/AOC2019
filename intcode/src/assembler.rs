use anyhow::Error;
use clap::{App, Arg};
use intcode::{Arguments, Program};
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
        .about("Assemble Intcode Programs")
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

    print!("{}", prog.assembly());

    Ok(())
}
