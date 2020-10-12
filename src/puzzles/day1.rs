use std::io::{BufRead, BufReader, Read};

type Error = Box<dyn ::std::error::Error + Sync + Send + 'static>;

fn fuel(module: u32) -> u32 {
    (module / 3).saturating_sub(2)
}

fn recursive_fuel(module: u32) -> u32 {
    let mut total_fuel = 0;
    let mut f = fuel(module);
    total_fuel += f;
    while f > 0 {
        f = fuel(f);
        total_fuel += f;
    }
    total_fuel
}
pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let reader = BufReader::new(input);

    let modules = reader
        .lines()
        .map(|line| line.map(|element| element.parse::<u32>()))
        .collect::<Result<Result<Vec<u32>, std::num::ParseIntError>, std::io::Error>>()??;

    println!("Part 1: {}", modules.iter().map(|m| fuel(*m)).sum::<u32>());
    println!(
        "Part 2: {}",
        modules.iter().map(|m| recursive_fuel(*m)).sum::<u32>()
    );

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples_part1() {
        assert_eq!(fuel(12), 2);
        assert_eq!(fuel(14), 2);
        assert_eq!(fuel(1969), 654);
        assert_eq!(fuel(100756), 33583);
    }

    #[test]
    fn examples_part2() {
        assert_eq!(recursive_fuel(14), 2);
        assert_eq!(recursive_fuel(1969), 966);
        assert_eq!(recursive_fuel(100756), 50346);
    }
}
