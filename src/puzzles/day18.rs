use anyhow::Error;
use geometry::coord2d::pathfinder;
use geometry::coord2d::{Direction, Point};

use std::cell::Cell;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::io::Read;
use std::time;

use searcher::{self, SearchCandidate, SearchHeuristic};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Key {
    door: char,
    location: Point,
}

impl Key {
    pub(crate) fn new(door: char, location: Point) -> Self {
        Key { door, location }
    }
}

struct NoDoorMap<'m>(&'m map::Map);

impl<'m> pathfinder::Map for NoDoorMap<'m> {
    fn is_traversable(&self, location: Point) -> bool {
        self.0.get(location).is_some()
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct SpelunkState(String, Point);

#[derive(Debug, Clone)]
pub(crate) struct Spelunker<'m> {
    caves: &'m map::Map,
    keys: map::KeyRing,
    path: Vec<char>,
    location: Point,
    distance: usize,
    heuristic: Cell<Option<usize>>,
}

impl<'m> PartialEq for Spelunker<'m> {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path) && self.location.eq(&other.location)
    }
}

impl<'m> Eq for Spelunker<'m> {}

impl<'m> Ord for Spelunker<'m> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance().cmp(&other.distance()).reverse()
    }
}

impl<'m> PartialOrd for Spelunker<'m> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'m> SearchCandidate for Spelunker<'m> {
    type State = SpelunkState;

    fn is_complete(&self) -> bool {
        self.keys.len() == self.caves.keys().len()
    }

    fn state(&self) -> SpelunkState {
        SpelunkState(self.keys.state(), self.location().unwrap())
    }

    fn score(&self) -> usize {
        self.distance()
    }

    fn children(&self) -> Vec<Self> {
        self.candidates().unwrap()
    }
}

impl<'m> SearchHeuristic for Spelunker<'m> {
    fn heuristic(&self) -> usize {
        if let Some(h) = self.heuristic.get() {
            return h;
        }

        use pathfinder::Map;
        let mut here = self.location().unwrap();
        let mut h = 0;

        for key in self.caves.keys() {
            if self.keys.contains(&key.door) {
                continue;
            }

            let p = NoDoorMap(self.caves).path(here, key.location).unwrap();
            h += p.distance();
            here = *p.destination();
        }

        let total_heuristic = h + self.distance();

        self.heuristic.set(Some(total_heuristic));
        total_heuristic
    }
}

impl<'m> pathfinder::Map for Spelunker<'m> {
    fn is_traversable(&self, location: Point) -> bool {
        match self.caves.get(location) {
            Some(map::Tile::Door(c)) => self.keys.contains(&c),
            Some(_) => true,
            None => false,
        }
    }
}

impl<'m> Spelunker<'m> {
    fn new(map: &'m map::Map) -> Self {
        Self {
            caves: map,
            keys: map::KeyRing::default(),
            path: Vec::new(),
            location: map.entrance().unwrap(),
            distance: 0,
            heuristic: Cell::new(None),
        }
    }

    fn location(&self) -> Result<Point, Error> {
        Ok(self.location)
    }

    fn candidates(&self) -> Result<Vec<Spelunker<'m>>, Error> {
        let mut candidates = Vec::with_capacity(4);

        for direction in Direction::all() {
            let target = self.location.step(direction);
            if self.caves.is_deadend(&target, &self.keys) {
                continue;
            }

            match self.caves.get(target) {
                Some(map::Tile::Key(c)) => {
                    let mut newsp = self.clone();
                    newsp.found_key(c);
                    newsp.location = target;
                    newsp.distance += 1;
                    candidates.push(newsp);
                }
                Some(map::Tile::Door(c)) => {
                    if self.keys.contains(&c) {
                        let mut newsp = self.clone();
                        newsp.location = target;
                        newsp.distance += 1;
                        candidates.push(newsp);
                    }
                }
                Some(map::Tile::Entrance) => {
                    let mut newsp = self.clone();
                    newsp.location = target;
                    newsp.distance += 1;
                    candidates.push(newsp);
                }
                Some(map::Tile::Hall) => {
                    let mut newsp = self.clone();
                    newsp.location = target;
                    newsp.distance += 1;
                    candidates.push(newsp);
                }
                None => {}
            }
        }

        Ok(candidates)
    }

    fn found_key(&mut self, key: char) {
        if self.keys.insert(key) {
            self.path.push(key);
        }
    }

    fn distance(&self) -> usize {
        self.distance
    }

    fn keys(&self) -> KeyPath {
        KeyPath(self.path.clone())
    }
}

struct KeyPath(Vec<char>);

impl ToString for KeyPath {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn search<'m>(map: &'m map::Map) -> Result<Spelunker<'m>, Error> {
    let origin = Spelunker::new(map);

    Ok(searcher::djirkstra(origin).run()?)
}

mod map {
    use anyhow::{anyhow, Error};
    use geometry::coord2d::pathfinder;
    use geometry::coord2d::{Direction, Point};

    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};
    use std::convert::{TryFrom, TryInto};
    use std::str::FromStr;

    use super::Key;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub(crate) enum Tile {
        Hall,
        Entrance,
        Door(char),
        Key(char),
    }

    impl TryFrom<char> for Tile {
        type Error = Error;

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '.' => Ok(Tile::Hall),
                '@' => Ok(Tile::Entrance),
                '#' => Err(anyhow!("Unexpected wall!")),
                c if c.is_ascii_lowercase() && c.is_ascii_alphabetic() => Ok(Tile::Key(c)),
                c if c.is_ascii_uppercase() && c.is_ascii_alphabetic() => {
                    Ok(Tile::Door(c.to_ascii_lowercase()))
                }
                c => Err(anyhow!("Unexpected character: {}", c)),
            }
        }
    }

    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    pub(crate) struct KeyRing(HashSet<char>);

    impl KeyRing {
        pub(crate) fn insert(&mut self, key: char) -> bool {
            self.0.insert(key)
        }

        pub(crate) fn contains(&self, key: &char) -> bool {
            self.0.contains(key)
        }

        pub(crate) fn state(&self) -> String {
            let mut keys: Vec<&char> = self.0.iter().collect();
            keys.sort();
            keys.into_iter().collect()
        }

        pub(crate) fn len(&self) -> usize {
            self.0.len()
        }
    }

    #[derive(Debug, Clone, Default)]
    struct DeadendCache(HashMap<String, HashSet<Point>>);

    impl DeadendCache {
        fn inherit(&mut self, keys: &KeyRing) {
            let mut initial = HashSet::new();
            for (state, deadends) in self.0.iter() {
                let keystate: HashSet<char> = state.chars().collect();
                if keys.0.is_superset(&keystate) {
                    for loc in deadends {
                        initial.insert(*loc);
                    }
                }
            }
            self.0.insert(keys.state(), initial);
        }

        fn warmed(&self, keys: &KeyRing) -> bool {
            self.0.contains_key(&keys.state())
        }

        fn contains(&self, location: &Point, keys: &KeyRing) -> bool {
            self.0
                .get(&keys.state())
                .map(|d| d.contains(location))
                .unwrap_or(false)
        }

        fn insert(&mut self, location: &Point, keys: &KeyRing) -> bool {
            self.0
                .entry(keys.state())
                .or_insert(HashSet::new())
                .insert(*location)
        }
    }

    #[derive(Debug, Clone, Default)]
    pub(crate) struct Map {
        tiles: HashMap<Point, Tile>,
        deadends: RefCell<DeadendCache>,
    }

    impl FromStr for Map {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut tiles = HashMap::new();

            for (y, line) in s.lines().enumerate() {
                for (x, c) in line.trim().chars().enumerate() {
                    if c != '#' {
                        let point: Point = (x, y).into();
                        let tile: Tile = c.try_into()?;
                        tiles.insert(point, tile);
                    }
                }
            }

            Ok(Map::new(tiles))
        }
    }

    struct Scout<'m, 'k> {
        origin: Point,
        map: &'m Map,
        keys: &'k KeyRing,
    }

    impl<'m, 'k> pathfinder::Map for Scout<'m, 'k> {
        fn is_traversable(&self, location: Point) -> bool {
            location != self.origin
                && match self.map.get(location) {
                    Some(Tile::Door(ref c)) => self.keys.contains(c),
                    Some(_) => true,
                    None => false,
                }
        }
    }

    impl Map {
        fn new(tiles: HashMap<Point, Tile>) -> Self {
            Map {
                tiles,
                ..Map::default()
            }
        }

        fn precompute_deadend(&self, location: &Point, keys: &KeyRing) {
            for direction in Direction::all() {
                if !self.precompute_deadend_direction(location, direction, keys) {
                    self.deadends
                        .borrow_mut()
                        .insert(&location.step(direction), keys);
                }
            }
        }

        fn precompute_deadend_direction(
            &self,
            location: &Point,
            direction: Direction,
            keys: &KeyRing,
        ) -> bool {
            use pathfinder::Map;
            let start = location.step(direction);
            let scout = Scout {
                origin: *location,
                map: self,
                keys: keys,
            };

            for (point, tile) in self.tiles.iter() {
                let target = match tile {
                    Tile::Key(c) if !keys.contains(c) => Some(point),
                    Tile::Door(c) if !keys.contains(c) => Some(point),
                    _ => None,
                };
                if let Some(end) = target {
                    if scout.path(start, *end).is_some() {
                        return true;
                    }
                }
            }
            false
        }

        fn precompute_deadends(&self, keys: &KeyRing) {
            for (point, tile) in self.tiles.iter() {
                match tile {
                    Tile::Key(_) => self.precompute_deadend(point, keys),
                    // Tile::Door(_) => self.precompute_deadend(point, keys),
                    _ => {}
                }
            }
        }

        pub(crate) fn is_deadend(&self, location: &Point, keys: &KeyRing) -> bool {
            if !self.deadends.borrow().warmed(keys) {
                // self.deadends.borrow_mut().inherit(keys);
                self.precompute_deadends(keys);
                // eprintln!("{:?}", self.deadends.borrow());
            }
            self.deadends.borrow().contains(location, keys)
        }

        pub(crate) fn keys(&self) -> HashSet<Key> {
            self.tiles
                .iter()
                .filter_map(|(p, t)| match t {
                    Tile::Key(k) => Some(Key::new(*k, *p)),
                    _ => None,
                })
                .collect()
        }

        pub(crate) fn get(&self, location: Point) -> Option<Tile> {
            self.tiles.get(&location).copied()
        }

        pub(crate) fn entrance(&self) -> Option<Point> {
            self.tiles.iter().find_map(|(p, t)| match t {
                Tile::Entrance => Some(*p),
                _ => None,
            })
        }
    }
}

pub(crate) fn main(mut input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let map: map::Map = {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf.parse()?
    };

    let start = time::Instant::now();

    let sp = search(&map)?;
    println!("Part 1: {}", sp.distance());
    println!("  Keys: {}", sp.keys().to_string());
    println!("  Time: {}s", start.elapsed().as_secs());

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples_part1_a() {
        let map: map::Map = "
        #########
        #b.A.@.a#
        #########"
            .parse()
            .unwrap();

        let sp = search(&map).unwrap();
        assert_eq!(sp.distance(), 8);
        assert_eq!(sp.keys().to_string(), "a,b");
    }

    #[test]
    fn examples_part1_b() {
        let map: map::Map = "
        ########################
        #f.D.E.e.C.b.A.@.a.B.c.#
        ######################.#
        #d.....................#
        ########################
        "
        .parse()
        .unwrap();

        let sp = search(&map).unwrap();
        assert_eq!(sp.distance(), 86);
    }

    #[test]
    fn examples_part1_c() {
        let map: map::Map = "
        ########################
        #...............b.C.D.f#
        #.######################
        #.....@.a.B.c.d.A.e.F.g#
        ########################
        "
        .parse()
        .unwrap();

        let sp = search(&map).unwrap();
        assert_eq!(sp.distance(), 132);
        assert_eq!(sp.keys().to_string(), "b,a,c,d,f,e,g")
    }

    #[test]
    fn examples_part1_d() {
        let map: map::Map = "
        #################
        #i.G..c...e..H.p#
        ########.########
        #j.A..b...f..D.o#
        ########@########
        #k.E..a...g..B.n#
        ########.########
        #l.F..d...h..C.m#
        #################
        "
        .parse()
        .unwrap();

        // assert!(search(&map).is_err());

        let sp = search(&map).unwrap();
        assert_eq!(sp.distance(), 136);
    }

    #[test]
    fn examples_part1_e() {
        let map: map::Map = "
        #################
        #j.A.b......fG.o#
        ########@########
        #k.F..a.....gB.n#
        #################
        "
        .parse()
        .unwrap();

        let sp = search(&map).unwrap();
        assert_eq!(sp.distance(), 62);
        assert_eq!(sp.keys().to_string(), "a,b,j,g,n,f,o,k")
    }

    #[test]
    fn examples_part1_f() {
        let map: map::Map = "
        ########################
        #@..............ac.GI.b#
        ###d#e#f################
        ###A#B#C################
        ###g#h#i################
        ########################
        "
        .parse()
        .unwrap();

        let sp = search(&map).unwrap();
        assert_eq!(sp.distance(), 81);
    }
}
