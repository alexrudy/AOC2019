use anyhow::{anyhow, Error};
use geometry::coord3d::Point3D;
use lazy_static::lazy_static;
use num::integer::lcm;
use regex::Regex;

use std::collections::{HashMap, HashSet};
use std::convert::{From, TryInto};
use std::hash::Hash;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct Moon {
    position: Point3D,
    velocity: Point3D,
}

macro_rules! gravity_axis {
    ($this:ident, $axis:ident, $other:ident) => {
        if $this.position.$axis > $other.position.$axis {
            $this.velocity.$axis -= 1;
        } else if $this.position.$axis < $other.position.$axis {
            $this.velocity.$axis += 1;
        }
    };
}

macro_rules! velocity_axis {
    ($this:ident, $axis:ident) => {
        $this.position.$axis += $this.velocity.$axis
    };
}

macro_rules! coordinate {
    ($map:ident, $axis:expr, $source:expr) => {
        $map.get($axis)
            .ok_or(anyhow!("Missing axis {} from {}", $axis, $source))?
    };
}

impl Moon {
    #[allow(dead_code)]
    fn new(position: Point3D) -> Self {
        Moon {
            position: position,
            velocity: Point3D::origin(),
        }
    }

    fn gravity(&mut self, other: &Moon) {
        gravity_axis!(self, x, other);
        gravity_axis!(self, y, other);
        gravity_axis!(self, z, other);
    }

    fn movement(&mut self) {
        velocity_axis!(self, x);
        velocity_axis!(self, y);
        velocity_axis!(self, z);
    }

    fn potential(&self) -> i32 {
        self.position.x.abs() + self.position.y.abs() + self.position.z.abs()
    }

    fn kinetic(&self) -> i32 {
        self.velocity.x.abs() + self.velocity.y.abs() + self.velocity.z.abs()
    }
}

impl FromStr for Moon {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"<(x|y|z)=(-?[\d]+),?\s*(x|y|z)=(-?[\d]+),?\s*(x|y|z)=(-?[\d]+),?\s*>")
                    .unwrap();
        }

        let cap = match RE.captures(s) {
            None => {
                return Err(anyhow!("Can't parse moon {}", s));
            }
            Some(c) => c,
        };

        let mut coordinates = HashMap::new();
        for i in 0..=2 {
            let axis: &str = &cap[i * 2 + 1];
            let position: i32 = cap[i * 2 + 2]
                .parse()
                .map_err(|_| anyhow!("Can't parse digit {} for axis {}", &cap[i * 2 + 1], axis))?;
            coordinates.insert(axis.to_owned(), position);
        }

        let x = coordinate!(coordinates, "x", s);
        let y = coordinate!(coordinates, "y", s);
        let z = coordinate!(coordinates, "z", s);

        Ok(Moon {
            position: Point3D::new(*x, *y, *z),
            velocity: Point3D::origin(),
        })
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct Jupiter {
    moons: [Moon; 4],
}

impl From<Vec<Moon>> for Jupiter {
    fn from(v: Vec<Moon>) -> Self {
        let moons: [Moon; 4] = {
            let boxed_slice = v.into_boxed_slice();
            let boxed_array: Box<[Moon; 4]> = boxed_slice.try_into().unwrap();
            *boxed_array
        };

        Jupiter { moons }
    }
}

impl Jupiter {
    fn step(&mut self) {
        let n = self.moons.len();
        for _ in 0..n {
            if let Some((first, elements)) = self.moons.split_first_mut() {
                for other in elements.iter() {
                    first.gravity(other);
                }
            }
            self.moons.rotate_left(1);
        }
        for moon in self.moons.iter_mut() {
            moon.movement();
        }
    }

    fn total_energy(&self) -> i32 {
        self.moons.iter().map(|m| m.potential() * m.kinetic()).sum()
    }

    fn evolve(&mut self) -> Evolution {
        Evolution { system: self }
    }
}

struct Evolution<'a> {
    system: &'a mut Jupiter,
}

impl<'a> Iterator for Evolution<'a> {
    type Item = Jupiter;

    fn next(&mut self) -> Option<Self::Item> {
        let result = Some(self.system.clone());
        self.system.step();
        result
    }
}

macro_rules! axis_state {
    ($element:ident, $axis:ident) => {
        AxisState {
            positions: [
                $element.moons[0].position.$axis,
                $element.moons[1].position.$axis,
                $element.moons[2].position.$axis,
                $element.moons[3].position.$axis,
            ],
            velocities: [
                $element.moons[0].velocity.$axis,
                $element.moons[1].velocity.$axis,
                $element.moons[2].velocity.$axis,
                $element.moons[3].velocity.$axis,
            ],
        }
    };
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct AxisState {
    positions: [i32; 4],
    velocities: [i32; 4],
}

impl AxisState {
    fn x(system: &Jupiter) -> Self {
        axis_state!(system, x)
    }
    fn y(system: &Jupiter) -> Self {
        axis_state!(system, y)
    }
    fn z(system: &Jupiter) -> Self {
        axis_state!(system, z)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct AxisPeriod {
    first: AxisState,
    start: usize,
    length: usize,
}

#[derive(Debug, Default)]
struct AxisPeriods {
    states: HashMap<AxisState, usize>,
    periods: HashSet<AxisPeriod>,
}

impl AxisPeriods {
    fn add(&mut self, state: AxisState, iteration: usize) -> Option<AxisPeriod> {
        if let Some(previous) = self.states.insert(state, iteration) {
            let period = AxisPeriod {
                first: state,
                start: previous,
                length: iteration - previous,
            };
            self.periods.insert(period);
            Some(period)
        } else {
            None
        }
    }

    fn is_empty(&self) -> bool {
        self.periods.is_empty()
    }

    fn period(&self) -> Option<usize> {
        self.periods.iter().map(|p| p.length).min()
    }
}

fn moon_periods<I>(iterator: I) -> Option<usize>
where
    I: Iterator<Item = Jupiter>,
{
    let mut x = AxisPeriods::default();
    let mut y = AxisPeriods::default();
    let mut z = AxisPeriods::default();

    for (i, e) in iterator.enumerate() {
        x.add(AxisState::x(&e), i);
        y.add(AxisState::y(&e), i);
        z.add(AxisState::z(&e), i);

        if !x.is_empty() && !y.is_empty() && !z.is_empty() {
            break;
        }
    }

    if x.is_empty() || y.is_empty() || z.is_empty() {
        return None;
    }

    let xp = x.period().unwrap();
    let yp = y.period().unwrap();
    let zp = z.period().unwrap();

    Some(lcm(lcm(xp, yp), zp))
}

fn read_moons(reader: Box<dyn Read + 'static>) -> Result<Vec<Moon>, Error> {
    let mut moons = Vec::with_capacity(4);
    let bufread = BufReader::new(reader);
    for line in bufread.lines() {
        let moon: Moon = line?.trim().parse()?;
        moons.push(moon);
    }

    Ok(moons)
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let jupiter: Jupiter = read_moons(input)?.into();

    let final_state = jupiter
        .clone()
        .evolve()
        .nth(1000)
        .ok_or(anyhow!("System stopped evolving!"))?;

    println!("Part 1: Total energy = {}", final_state.total_energy());

    let cycle = moon_periods(jupiter.clone().evolve()).unwrap();
    println!("Part 2: cycles afert {} steps", cycle);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::get_default_input;
    use crate::iterhelper::repeated_element;

    lazy_static! {
        static ref PARTA: Jupiter = read_moons(Box::new(
            "<x=-1, y=0, z=2>
        <x=2, y=-10, z=-7>
        <x=4, y=-8, z=8>
        <x=3, y=5, z=-1>"
                .as_bytes(),
        ))
        .unwrap()
        .into();
    }

    lazy_static! {
        static ref PARTB: Jupiter = read_moons(Box::new(
            "<x=-8, y=-10, z=0>
            <x=5, y=5, z=10>
            <x=2, y=-7, z=3>
            <x=9, y=-8, z=-3>"
                .as_bytes(),
        ))
        .unwrap()
        .into();
    }

    #[test]
    fn parse_moon() {
        assert_eq!(
            Moon::new((-1, 0, 2).into()),
            "<x=-1, y=0, z=2>".parse().unwrap(),
        );

        let moons = read_moons(Box::new(
            "<x=-1, y=0, z=2>
        <x=2, y=-10, z=-7>
        <x=4, y=-8, z=8>
        <x=3, y=5, z=-1>"
                .as_bytes(),
        ))
        .unwrap();
        assert_eq!(moons.len(), 4);
        assert_eq!(moons[2], Moon::new((4, -8, 8).into()));
    }

    #[test]
    fn example_day12_part1a() {
        let mut jupiter: Jupiter = PARTA.clone();

        let endstate = jupiter.evolve().nth(10).unwrap();

        let expected = vec![(6, 6), (9, 5), (10, 8), (6, 3)];

        for (moon, (p, k)) in endstate.moons.iter().zip(expected.iter()) {
            assert_eq!(moon.potential(), *p);
            assert_eq!(moon.kinetic(), *k);
        }
    }

    #[test]
    fn example_day12_part1b() {
        let mut jupiter: Jupiter = PARTB.clone();
        assert_eq!(jupiter.evolve().nth(100).unwrap().total_energy(), 1940);
    }

    #[test]
    fn problem_day12_part1() {
        let mut system: Jupiter = read_moons(get_default_input(12).unwrap()).unwrap().into();

        let part1_energy = system.evolve().nth(1000).unwrap().total_energy();
        assert_eq!(part1_energy, 7758);
    }

    #[test]
    fn example_day12_part2a() {
        let mut jupiter: Jupiter = PARTA.clone();

        let repeats = repeated_element(jupiter.evolve()).unwrap();
        assert_eq!(repeats.end(), 2772);
    }

    #[test]
    fn example_day12_part2b() {
        let mut jupiter: Jupiter = PARTB.clone();
        let period = moon_periods(jupiter.evolve()).unwrap();
        assert_eq!(period, 4686774924);
    }

    #[test]
    fn problem_day12_part2() {
        let mut system: Jupiter = read_moons(get_default_input(12).unwrap()).unwrap().into();
        let period = moon_periods(system.evolve()).unwrap();

        assert_eq!(period, 354540398381256);
    }
}
