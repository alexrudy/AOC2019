use anyhow::anyhow;
use anyhow::Error;

use std::convert::TryInto;
use std::io::Read;
use std::iter;
use std::str::FromStr;

const PATTERN: [i32; 4] = [0, 1, 0, -1];

fn pattern_for_element(element: usize) -> impl Iterator<Item = i32> {
    PATTERN
        .iter()
        .flat_map(move |p| iter::repeat(*p).take(element))
        .cycle()
        .skip(1)
}

fn signal_for_element(signal: &[i32], element: usize) -> i32 {
    signal
        .iter()
        .zip(pattern_for_element(element))
        .skip(element - 1)
        .map(|(&s, v)| s * v)
        .sum::<i32>()
        .abs()
        % 10
}

fn phase(signal: &[i32]) -> Vec<i32> {
    (1..=signal.len())
        .map(|element| signal_for_element(signal, element))
        .collect()
}

#[derive(Debug, Clone)]
struct FlawedFrequencySignal {
    signal: Vec<i32>,
}

impl FlawedFrequencySignal {
    fn new(signal: &[i32]) -> FlawedFrequencySignal {
        FlawedFrequencySignal {
            signal: signal.into(),
        }
    }

    fn len(&self) -> usize {
        self.signal.len()
    }

    fn message_offset(&self) -> usize {
        self.signal
            .iter()
            .take(7)
            .enumerate()
            .map(|(oom, s)| 10i32.pow(6 - oom as u32) * s)
            .sum::<i32>()
            .try_into()
            .expect("Valid offset")
    }

    fn transform(&self) -> FlawedFrequencyTransform {
        FlawedFrequencyTransform {
            signal: self.signal.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct FlawedFrequencyTransform {
    signal: Vec<i32>,
}

impl Iterator for FlawedFrequencyTransform {
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.signal = phase(&self.signal);
        Some(self.signal.clone())
    }
}

impl FlawedFrequencyTransform {
    #[cfg(test)]
    fn new(signal: FlawedFrequencySignal) -> Self {
        FlawedFrequencyTransform {
            signal: signal.signal,
        }
    }
}

impl FromStr for FlawedFrequencySignal {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let digits = s
            .chars()
            .map(|c| {
                c.to_digit(10)
                    .ok_or(anyhow!("Invalid number: {}", s))
                    .map(|d| d as i32)
            })
            .collect::<Result<Vec<i32>, Error>>()?;
        Ok(FlawedFrequencySignal { signal: digits })
    }
}

#[derive(Debug)]
struct FlawedFrequencyMessage {
    signal: FlawedFrequencySignal,
    iterations: usize,
    repeats: usize,
    offset: usize,
    len: usize,
}

impl FlawedFrequencyMessage {
    fn process(self) -> Vec<i32> {
        let mut transform = FlawedFrequencySignal::new(&self.signal_materialized()).transform();
        transform
            .nth(self.iterations)
            .unwrap()
            .into_iter()
            .skip(self.offset)
            .take(self.len)
            .collect::<Vec<_>>()
    }

    fn signal_materialized(&self) -> Vec<i32> {
        self.signal
            .signal
            .iter()
            .cycle()
            .take(self.signal.len() * self.repeats)
            .copied()
            .collect()
    }

    fn triangle_process(self) -> Vec<i32> {
        let mut signal: Vec<i32> = self
            .signal_materialized()
            .into_iter()
            .skip(self.offset)
            .collect();

        for _ in 0..100 {
            let restsum: i32 = signal.iter().sum();

            signal = signal
                .iter()
                .scan(restsum, |rs, v| {
                    let output = rs.abs() % 10;
                    *rs -= v;
                    Some(output)
                })
                .collect()
        }

        signal.into_iter().take(8).collect()
    }

    fn part1(signal: FlawedFrequencySignal) -> Vec<i32> {
        let msg = FlawedFrequencyMessage {
            signal: signal,
            repeats: 1,
            iterations: 99,
            offset: 0,
            len: 8,
        };

        msg.process()
    }

    fn part2(signal: FlawedFrequencySignal) -> Vec<i32> {
        let offset = signal.message_offset();
        assert!(
            offset * 2 > (signal.len() * 10_000),
            "Solution only applies when offset covers at least half the input"
        );

        let msg = FlawedFrequencyMessage {
            signal: signal,
            repeats: 10_000,
            iterations: 99,
            offset: offset,
            len: 8,
        };

        msg.triangle_process()
    }
}

fn read(mut input: Box<dyn Read + 'static>) -> Result<FlawedFrequencySignal, Error> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    buffer.parse()
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let signal = read(input)?;
    {
        print!("Part 1: ");
        for e in FlawedFrequencyMessage::part1(signal.clone()) {
            print!("{}", e);
        }
        println!("");
    }

    {
        print!("Part 2: ");
        for e in FlawedFrequencyMessage::part2(signal.clone()) {
            print!("{}", e);
        }
        println!("");
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pattern() {
        assert_eq!(
            pattern_for_element(3).take(15).collect::<Vec<_>>(),
            vec![0, 0, 1, 1, 1, 0, 0, 0, -1, -1, -1, 0, 0, 0, 1]
        );
        assert_eq!(
            pattern_for_element(2).take(15).collect::<Vec<_>>(),
            vec![0, 1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -1]
        );
    }

    #[test]
    fn examples_part1() {
        let mut signal = FlawedFrequencySignal::new(&[1, 2, 3, 4, 5, 6, 7, 8]).transform();
        assert_eq!(signal.next().unwrap(), vec![4, 8, 2, 2, 6, 1, 5, 8]);
        assert_eq!(signal.next().unwrap(), vec![3, 4, 0, 4, 0, 4, 3, 8]);
        assert_eq!(signal.next().unwrap(), vec![0, 3, 4, 1, 5, 5, 1, 8]);
        assert_eq!(signal.next().unwrap(), vec![0, 1, 0, 2, 9, 4, 9, 8]);

        assert_eq!(
            big_example("80871224585914546619083218645595"),
            vec![2, 4, 1, 7, 6, 1, 7, 6]
        );
    }

    fn big_example(s: &str) -> Vec<i32> {
        let signal: FlawedFrequencySignal = s.parse().unwrap();
        FlawedFrequencyMessage::part1(signal)
    }

    fn big_example_part2(s: &str) -> Vec<i32> {
        let signal: FlawedFrequencySignal = s.parse().unwrap();

        FlawedFrequencyMessage::part2(signal)
    }

    #[test]
    fn examples_part2() {
        assert_eq!(
            big_example_part2("03036732577212944063491565474664"),
            vec![8, 4, 4, 6, 2, 0, 2, 6]
        );
    }
}
