use anyhow::Error;
use std::io::Read;

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    println!("Hello!");

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples_part1() {}

    #[test]
    fn answer_part1() {}

    #[test]
    fn examples_part2() {}

    #[test]
    fn answer_part2() {}
}
