use anyhow::{anyhow, Error};
use std::convert::TryInto;
use std::io::Read;
use std::str::FromStr;

pub(crate) fn main(mut input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let mut buf = String::new();
    input.read_to_string(&mut buf)?;
    let values = buf
        .split('-')
        .map(|part| part.parse::<u32>())
        .take(2)
        .collect::<Result<Vec<u32>, std::num::ParseIntError>>()?;
    let start = values[0];
    let end = values[1];

    let n = (start..=end)
        .filter(|&candidate| check(Password::from_number(candidate).unwrap()))
        .count();
    println!("Part 1: {} candidate passwords", n);

    let n2 = (start..=end)
        .filter(|&candidate| check_part_2(Password::from_number(candidate).unwrap()))
        .count();

    println!("Part 2: {} candidate passwords", n2);

    Ok(())
}

#[derive(Debug, Copy, Clone)]
struct Password([u32; 6]);

impl FromStr for Password {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s
            .chars()
            .map(|c| c.to_digit(10).ok_or(anyhow!("Invalid password")))
            .collect::<Result<Vec<u32>, Error>>()?;

        let array = {
            let boxed_slice = values.into_boxed_slice();
            let boxed_array: Box<[u32; 6]> = match boxed_slice.try_into() {
                Ok(ba) => ba,
                Err(o) => Err(anyhow!(
                    "Expected a password of length {} but it was {}",
                    6,
                    o.len()
                ))?,
            };
            *boxed_array
        };
        Ok(Password(array))
    }
}

impl Password {
    fn from_number(number: u32) -> Result<Password, Error> {
        let ns = format!("{}", number);
        ns.parse()
    }
}

fn check(password: Password) -> bool {
    let pairs = password.0.iter().zip(password.0.iter().skip(1));
    let mut doubles = false;
    for (a, b) in pairs {
        if a > b {
            return false;
        }
        if a == b {
            doubles = true;
        }
    }
    return doubles;
}

fn check_part_2(password: Password) -> bool {
    let pairs = password.0.iter().zip(password.0.iter().skip(1));
    let mut doubles = false;
    for (i, (a, b)) in pairs.enumerate() {
        if a > b {
            return false;
        }

        if a == b && (i == 0 || &password.0[i - 1] != a) && (i == 4 || &password.0[i + 2] != b) {
            doubles = true;
        }
    }

    doubles
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples_part1() {
        assert_eq!(check("111111".parse().unwrap()), true);
        assert_eq!(check("223450".parse().unwrap()), false);
        assert_eq!(check("123789".parse().unwrap()), false);
    }

    #[test]
    fn examples_part2() {
        assert_eq!(check_part_2("112233".parse().unwrap()), true);
        assert_eq!(check_part_2("123444".parse().unwrap()), false);
        assert_eq!(check_part_2("111122".parse().unwrap()), true);
    }
}
