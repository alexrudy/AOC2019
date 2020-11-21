use anyhow::anyhow;
use anyhow::Error;
use geometry::coord2d::pathfinder;
use geometry::coord2d::Point;

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::{HashMap, HashSet};
use std::io::Read;

use crate::searcher::{bfs, djirkstra, SearchCandidate};

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

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct SpelunkState(String, Point);

#[derive(Debug, Clone, Default)]
struct SpelunkGraph {
    graph: HashMap<Option<char>, HashMap<char, (pathfinder::Path, HashSet<char>)>>,
}

struct NoDoorMap<'m>(&'m map::Map);

impl<'m> pathfinder::Map for NoDoorMap<'m> {
    fn is_traversable(&self, location: Point) -> bool {
        self.0.get(location).is_some()
    }
}

impl SpelunkGraph {
    fn build(map: &map::Map) -> Self {
        use geometry::coord2d::pathfinder::{Map, Pathfinder};

        let mut graph = Self::default();
        let pfm = NoDoorMap(map);

        let keys = map.keys();

        let entrance = map.entrance().unwrap();
        for first in &keys {
            if let Some(path) = pfm.path(entrance, first.location) {
                let mut req = HashSet::new();
                for step in path.iter().skip(1) {
                    match map.get(*step) {
                        Some(map::Tile::Door(c)) => {
                            req.insert(c);
                        }
                        _ => {}
                    }
                }

                graph
                    .graph
                    .entry(None)
                    .or_insert(HashMap::new())
                    .insert(first.door, (path, req));
            }
        }

        for start in &keys {
            for end in &keys {
                if start == end {
                    continue;
                }
                if let Some(path) = pfm.path(start.location, end.location) {
                    let mut req = HashSet::new();
                    for step in path.iter().skip(1) {
                        match map.get(*step) {
                            Some(map::Tile::Door(c)) => {
                                req.insert(c);
                            }
                            _ => {}
                        }
                    }

                    graph
                        .graph
                        .entry(Some(start.door))
                        .or_insert(HashMap::new())
                        .insert(end.door, (path, req));
                }
            }
        }

        let n: usize = graph.graph.values().map(|v| v.len()).sum();
        eprintln!("Built graph with {} elements", n);
        graph
    }

    fn edges(&self, start: Option<Key>, keys: &HashSet<char>) -> HashMap<char, pathfinder::Path> {
        let mut results = HashMap::new();
        for (end, (path, req)) in self.graph.get(&start.map(|k| k.door)).unwrap().iter() {
            if req.iter().all(|k| keys.contains(k)) && !keys.contains(end) {
                results.insert(*end, path.clone());
            }
        }
        results
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Spelunker<'m> {
    caves: &'m map::Map,
    keys: HashSet<char>,
    path: Vec<char>,
    location: Point,
    distance: usize,
    cache: SpelunkGraph,
}

impl<'m> PartialEq for Spelunker<'m> {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
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

    fn heuristic(&self) -> usize {
        let entrance = self.caves.entrance().unwrap();
        let here = self.location().unwrap();

        self.caves
            .keys()
            .iter()
            .filter_map(|k| {
                if self.keys.contains(&k.door) {
                    None
                } else {
                    let h = k.location.manhattan_distance(here) * 2;
                    // let e = k.location.manhattan_distance(entrance) * 2;
                    // if h > e {
                    // Some(e)
                    // } else {
                    // Some(h)
                    // }
                    Some(h)
                }
            })
            .sum::<i32>() as usize
            + self.distance()
    }

    fn state(&self) -> SpelunkState {
        let mut keys: Vec<char> = self.keys.iter().copied().collect();
        keys.sort();
        let ks: String = keys.iter().collect();

        SpelunkState(ks, self.location().unwrap())
    }

    fn score(&self) -> usize {
        self.distance()
    }

    fn children(&self) -> Vec<Self> {
        self.candidates().unwrap()
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
            keys: HashSet::new(),
            path: Vec::new(),
            location: map.entrance().unwrap(),
            distance: 0,
            cache: SpelunkGraph::build(map),
        }
    }

    fn location(&self) -> Result<Point, Error> {
        Ok(self.location)
    }

    fn candidate_paths(&self) -> Result<Vec<pathfinder::Path>, Error> {
        use geometry::coord2d::pathfinder::Map;

        let location = self.location()?;
        Ok(self
            .caves
            .keys()
            .iter()
            .filter(|&k| !self.keys.contains(&k.door))
            .filter_map(|k| self.path(location, k.location))
            .collect())
    }

    fn candidates(&self) -> Result<Vec<Spelunker<'m>>, Error> {
        let location = self.location()?;
        let door = match self.caves.get(location) {
            Some(map::Tile::Key(c)) => Some(c),
            Some(map::Tile::Entrance) => None,
            _ => Err(anyhow!("Didn't start on a key!"))?,
        };

        let edges = self
            .cache
            .edges(door.map(|c| Key::new(c, location)), &self.keys);

        Ok(edges
            .iter()
            .map(|(c, p)| {
                let mut newsp = self.clone();
                newsp.keys.insert(*c);
                newsp.path.push(*c);
                newsp.distance += p.distance();
                newsp
            })
            .collect())
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

    djirkstra(origin, Some(10_000))?.ok_or(anyhow!("No search result found!"))
}

mod map {
    use anyhow::{anyhow, Error};
    use geometry::coord2d::Point;

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

    impl Tile {
        fn is_key(&self) -> bool {
            match self {
                Tile::Key(_) => true,
                _ => false,
            }
        }
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

    #[derive(Debug, Clone)]
    pub(crate) struct Map {
        tiles: HashMap<Point, Tile>,
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

            Ok(Map { tiles })
        }
    }

    impl Map {
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

        pub(crate) fn key(&self, door: char) -> Option<Point> {
            self.tiles.iter().find_map(|(p, t)| match t {
                Tile::Key(c) if *c == door => Some(*p),
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

    let sp = search(&map)?;
    println!("Part 1: {}", sp.distance());
    println!("  Keys: {}", sp.keys().to_string());

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
    fn graph() {
        let map: map::Map = "
        ########################
        #f.D.E.e.C.b.A.@.a.B.c.#
        ######################.#
        #d.....................#
        ########################
        "
        .parse()
        .unwrap();

        let sg = SpelunkGraph::build(&map);

        let mut keys = sg.graph.keys().filter_map(|&c| c).collect::<Vec<char>>();
        keys.sort();

        assert_eq!(keys, vec!['a', 'b', 'c', 'd', 'e', 'f']);

        assert_eq!(sg.graph.get(&Some('a')).unwrap().len(), 5);

        for (d, (_, r)) in sg.graph.get(&Some('a')).unwrap().iter() {
            eprintln!("{} -> {} {:?}", 'a', d, r);
        }

        let mut keys = HashSet::new();
        keys.insert('a');
        let edges = sg.edges(Some(Key::new('a', (17, 1).into())), &keys);

        eprintln!("{:?}", edges.keys().collect::<Vec<_>>());

        assert_eq!(edges.len(), 1);
        assert_eq!(edges.keys().next(), Some(&'b'));
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

        assert!(search(&map).is_err());

        // let sp = search(&map).unwrap();
        // assert_eq!(sp.distance(), 136);
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
