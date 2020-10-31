use anyhow::Error;
use breakout::arcade;
use clap::{App, Arg};
use intcode::Program;
use std::fs::File;

type Result<T> = std::result::Result<T, Error>;

fn load_program(filename: Option<&str>) -> Result<Program> {
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
    let matches = App::new("Breakout - Advent of Code 2019")
        .version("1.0")
        .author("Alex Rudy <opensource@alexrudy.net>")
        .about("Play Breakout")
        .arg(
            Arg::with_name("program")
                .value_name("PROGRAM")
                .required(false)
                .takes_value(true)
                .index(1),
        )
        .arg(Arg::with_name("ai").long("ai").help("Use AI?"))
        .get_matches();

    let filename = matches.value_of("program");

    let program = load_program(filename)?;
    arcade(program, matches.is_present("ai"))?;

    Ok(())
}
