use anyhow::{anyhow, Error};
use geometry::Point;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryInto;
use std::fmt;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;

#[derive(Debug, Default)]
struct AsteroidMap {
    asteroids: HashSet<Point>,
}

fn gcd(x: i32, y: i32) -> i32 {
    let mut a = x;
    let mut b = y;
    let mut t;

    while b != 0 {
        t = b;
        b = a % b;
        a = t;
    }
    a
}

fn parse_asteroid(x: i32, y: i32, space: char) -> Result<Option<Point>, Error> {
    match space {
        '#' => Ok(Some(Point::new(x, y))),
        'X' => Ok(Some(Point::new(x, y))),
        '.' => Ok(None),
        _ => Err(anyhow!("Invalid Point!")),
    }
}

impl AsteroidMap {
    fn read(input: Box<dyn Read + 'static>) -> Result<AsteroidMap, Error> {
        let mut asteroids = HashSet::new();
        let reader = BufReader::new(input);
        for (row, line) in reader.lines().enumerate() {
            for (col, space) in line?.trim().chars().enumerate() {
                match parse_asteroid(col.try_into()?, row.try_into()?, space)? {
                    Some(point) => {
                        asteroids.insert(point);
                    }
                    None => {}
                };
            }
        }
        Ok(AsteroidMap {
            asteroids: asteroids,
        })
    }

    fn bbox(&self) -> Option<(i32, i32, i32, i32)> {
        let lower = self.asteroids.iter().map(|a| a.y).min()?;
        let upper = self.asteroids.iter().map(|a| a.y).max()?;
        let left = self.asteroids.iter().map(|a| a.x).min()?;
        let right = self.asteroids.iter().map(|a| a.x).max()?;
        Some((left, right, lower, upper))
    }

    fn observatories(&self) -> Observatories {
        let mut locations = HashMap::new();
        for asteroid in self.asteroids.iter().copied() {
            for other in self.asteroids.iter() {
                if other != &asteroid {
                    locations
                        .entry(asteroid.clone())
                        .or_insert(Observatory::new(asteroid.clone()))
                        .add(*other);
                }
            }
        }
        Observatories {
            locations: locations,
        }
    }
}

impl FromStr for AsteroidMap {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut asteroids = HashSet::new();
        for (row, line) in s.lines().enumerate() {
            for (col, space) in line.trim().chars().enumerate() {
                match parse_asteroid(col.try_into()?, row.try_into()?, space)? {
                    Some(point) => {
                        asteroids.insert(point);
                    }
                    None => {}
                };
            }
        }
        Ok(AsteroidMap {
            asteroids: asteroids,
        })
    }
}

impl fmt::Display for AsteroidMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(bbox) = self.bbox() {
            for y in bbox.2..=bbox.3 {
                for x in bbox.0..=bbox.1 {
                    let point = Point::new(x, y);
                    if self.asteroids.contains(&point) {
                        write!(f, "#")?;
                    } else {
                        write!(f, ".")?;
                    }
                }
                writeln!(f, "")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
struct Angle {
    x: i32,
    y: i32,
}

impl Angle {
    fn new(x: i32, y: i32) -> Self {
        let divisor = gcd(x, y).abs();
        Angle {
            x: x / divisor,
            y: y / divisor,
        }
    }

    fn radians(&self) -> f32 {
        let mut r = (self.y as f32).atan2(self.x as f32) + std::f32::consts::FRAC_PI_2;
        if r < 0.0 {
            r += 2.0 * std::f32::consts::PI;
        }
        r
    }
}

impl From<(i32, i32)> for Angle {
    fn from(coordinates: (i32, i32)) -> Self {
        Self::new(coordinates.0, coordinates.1)
    }
}

impl From<Point> for Angle {
    fn from(point: Point) -> Self {
        Self::new(point.x, point.y)
    }
}

#[derive(Debug, Clone)]
struct Observatory {
    location: Point,
    sightlines: HashMap<Angle, Vec<Point>>,
}

impl Observatory {
    fn new(location: Point) -> Self {
        Self {
            location: location,
            sightlines: HashMap::new(),
        }
    }
    fn add(&mut self, target: Point) -> () {
        let angle: Angle = target.offset(self.location).into();
        self.sightlines
            .entry(angle)
            .or_insert(Vec::new())
            .push(target);
    }

    fn ntargets(&self) -> usize {
        self.sightlines.keys().count()
    }

    fn cannon<'o>(&'o self) -> LaserCannon<'o> {
        LaserCannon::new(self)
    }
}

#[derive(Debug)]
struct LaserCannon<'o> {
    targets: Vec<Point>,
    index: usize,
    marker: std::marker::PhantomData<&'o Observatory>,
}

impl<'o> LaserCannon<'o> {
    fn new<'a>(obs: &'a Observatory) -> LaserCannon<'a> {
        let mut angles = obs.sightlines.keys().copied().collect::<Vec<Angle>>();
        angles.sort_by(|a, b| a.radians().partial_cmp(&b.radians()).unwrap());

        let mut targets_by_angle: Vec<VecDeque<Point>> = angles
            .iter()
            .map(|a| {
                let mut targets = obs.sightlines.get(a).unwrap().clone();
                targets.sort_by_key(|target| target.manhattan_distance(obs.location));
                targets.into()
            })
            .collect();

        let n = targets_by_angle.iter().map(|p| p.len()).sum();
        let mut targets = Vec::with_capacity(n);
        let mut seen = true;

        while seen {
            seen = false;
            for targets_at_angle in targets_by_angle.iter_mut() {
                match targets_at_angle.pop_front() {
                    None => {}
                    Some(t) => {
                        targets.push(t);
                        seen = true;
                    }
                }
            }
        }

        LaserCannon {
            targets: targets,
            index: 0,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'o> Iterator for LaserCannon<'o> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.targets.get(index).copied()
    }
}

#[derive(Debug, Default)]
struct Observatories {
    locations: HashMap<Point, Observatory>,
}

impl Observatories {
    fn bbox(&self) -> Option<(i32, i32, i32, i32)> {
        let lower = self.locations.keys().map(|a| a.y).min()?;
        let upper = self.locations.keys().map(|a| a.y).max()?;
        let left = self.locations.keys().map(|a| a.x).min()?;
        let right = self.locations.keys().map(|a| a.x).max()?;
        Some((left, right, lower, upper))
    }

    fn best(&self) -> Option<&Observatory> {
        self.locations.values().max_by_key(|&obs| obs.ntargets())
    }

    #[allow(dead_code)]
    fn get(&self, location: &Point) -> Option<&Observatory> {
        self.locations.get(location)
    }
}

impl fmt::Display for Observatories {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(bbox) = self.bbox() {
            for y in bbox.2..=bbox.3 {
                for x in bbox.0..=bbox.1 {
                    let point = Point::new(x, y);
                    match self.locations.get(&point) {
                        Some(obs) => {
                            write!(f, "{}", obs.ntargets())?;
                        }
                        None => {
                            write!(f, ".")?;
                        }
                    }
                }
                writeln!(f, "")?;
            }
        }
        Ok(())
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let map = AsteroidMap::read(input)?;
    let obs = map
        .observatories()
        .best()
        .ok_or(anyhow!("No observatory found!"))?
        .clone();
    let n = obs.ntargets();
    println!("Part 1: The best observatory spot can see {} asteroids", n);

    let winner = obs
        .cannon()
        .nth(199)
        .ok_or(anyhow!("Less than 200 asteroids were vaproized!"))?;
    let score = winner.x * 100 + winner.y;
    println!("Part 2: The bet winner is {}", score);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples_part1() {
        let input = ".#..#\n.....\n#####\n....#\n...##";
        let map: AsteroidMap = input.parse().unwrap();

        assert_eq!(input, format!("{}", map).trim());

        let output = ".7..7\n.....\n67775\n....7\n...87";

        assert_eq!(output, format!("{}", map.observatories()).trim());
        assert_eq!(
            map.observatories()
                .best()
                .map(|obs| (obs.location, obs.ntargets())),
            Some((Point::new(3, 4), 8))
        );

        assert_eq!(
            check_example(
                "......#.#.
        #..#.#....
        ..#######.
        .#.#.###..
        .#..#.....
        ..#....#.#
        #..#....#.
        .##.#..###
        ##...#..#.
        .#....####"
            ),
            ((5, 8).into(), 33)
        );

        assert_eq!(
            check_example(
                "#.#...#.#.
        .###....#.
        .#....#...
        ##.#.#.#.#
        ....#.#.#.
        .##..###.#
        ..#...##..
        ..##....##
        ......#...
        .####.###."
            ),
            ((1, 2).into(), 35)
        );

        assert_eq!(
            check_example(
                ".#..#..###
        ####.###.#
        ....###.#.
        ..###.##.#
        ##.##.#.#.
        ....###..#
        ..#.#..#.#
        #..#.#.###
        .##...##.#
        .....#.#.."
            ),
            ((6, 3).into(), 41)
        );

        assert_eq!(
            check_example(
                ".#..##.###...#######
        ##.############..##.
        .#.######.########.#
        .###.#######.####.#.
        #####.##.#.##.###.##
        ..#####..#.#########
        ####################
        #.####....###.#.#.##
        ##.#################
        #####.##.###..####..
        ..######..##.#######
        ####.##.####...##..#
        .#####..#.######.###
        ##...#.##########...
        #.##########.#######
        .####.#.###.###.#.##
        ....##.##.###..#####
        .#.#.###########.###
        #.#.#.#####.####.###
        ###.##.####.##.#..##"
            ),
            ((11, 13).into(), 210)
        );
    }

    fn check_example(example: &str) -> (Point, usize) {
        let map: AsteroidMap = example.parse().unwrap();
        return map
            .observatories()
            .best()
            .map(|obs| (obs.location, obs.ntargets()))
            .unwrap();
    }

    fn observatory(input: &str, location: Point) -> Observatory {
        let map: AsteroidMap = input.parse().unwrap();
        let observatories = map.observatories();
        observatories.get(&location).unwrap().clone()
    }

    #[test]
    fn example_part2_small() {
        let obs = observatory(
            ".#....#####...#..
        ##...##.#####..##
        ##...#...#.#####.
        ..#.....X...###..
        ..#.#.....#....##",
            (8, 3).into(),
        );

        assert_eq!(obs.cannon().nth(0).unwrap(), (8, 1).into());

        let hits: Vec<Point> = obs.cannon().take(6).collect();

        assert_eq!(hits[0], (8, 1).into());
        assert_eq!(hits[1], (9, 0).into());
        assert_eq!(hits[2], (9, 1).into());
        assert_eq!(hits[3], (10, 0).into());
        assert_eq!(hits[4], (9, 2).into());
        assert_eq!(hits[5], (11, 1).into());
    }

    #[test]
    fn example_part2_large() {
        let obs = observatory(
            ".#..##.###...#######
        ##.############..##.
        .#.######.########.#
        .###.#######.####.#.
        #####.##.#.##.###.##
        ..#####..#.#########
        ####################
        #.####....###.#.#.##
        ##.#################
        #####.##.###..####..
        ..######..##.#######
        ####.##.####...##..#
        .#####..#.######.###
        ##...#.##########...
        #.##########.#######
        .####.#.###.###.#.##
        ....##.##.###..#####
        .#.#.###########.###
        #.#.#.#####.####.###
        ###.##.####.##.#..##",
            (11, 13).into(),
        );

        assert_eq!(obs.cannon().nth(0).unwrap(), (11, 12).into());

        let hits: Vec<Point> = obs.cannon().collect();
        assert_eq!(hits.len(), 299);
        assert_eq!(hits[0], (11, 12).into());
        assert_eq!(hits[1], (12, 1).into());
        assert_eq!(hits[2], (12, 2).into());
        assert_eq!(hits[9], (12, 8).into());
        assert_eq!(hits[19], (16, 0).into());
        assert_eq!(hits[49], (16, 9).into());
        assert_eq!(hits[99], (10, 16).into());
        assert_eq!(hits[198], (9, 6).into());
        assert_eq!(hits[199], (8, 2).into());
        assert_eq!(hits[200], (10, 9).into());
        assert_eq!(hits[298], (11, 1).into());
    }

    #[test]
    fn angles_to_radians() {
        // Up
        check_angle((0, -1).into(), 0.0);
        // Right
        check_angle((5, 0).into(), std::f32::consts::FRAC_PI_2);
        // Down
        check_angle((0, 4).into(), std::f32::consts::PI);
        // Left
        check_angle(
            (-3, 0).into(),
            std::f32::consts::FRAC_PI_2 + std::f32::consts::PI,
        );

        // Up Right
        check_angle((1, -1).into(), std::f32::consts::FRAC_PI_4);
        // Right Down
        check_angle(
            (5, 5).into(),
            std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4,
        );
        // Down
        check_angle(
            (-4, 4).into(),
            std::f32::consts::PI + std::f32::consts::FRAC_PI_4,
        );
        // Left
        check_angle(
            (-3, -3).into(),
            std::f32::consts::FRAC_PI_2 + std::f32::consts::PI + std::f32::consts::FRAC_PI_4,
        );
    }

    fn check_angle(angle: Angle, radians: f32) -> () {
        let error = (angle.radians() - radians).abs();
        dbg!(angle.radians(), radians);
        assert!(error.abs() < (10.0 * std::f32::EPSILON));
    }
}
