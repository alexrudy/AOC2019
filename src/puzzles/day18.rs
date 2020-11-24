use anyhow::{anyhow, Error};
use geometry::coord2d::graph;
use geometry::coord2d::pathfinder;
use geometry::coord2d::Point;

use std::cell::Cell;
use std::cmp::{Eq, PartialEq};
use std::io::Read;
use std::time;

use searcher::{self, SearchCacher, SearchCandidate, SearchHeuristic};

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

#[derive(Debug)]
struct NoDoorMap<'m>(&'m map::Map);

impl<'m> pathfinder::Map for NoDoorMap<'m> {
    fn is_traversable(&self, location: Point) -> bool {
        self.0.get(location).is_some()
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct SpelunkState(String, Point);

#[derive(Debug, Clone)]
pub(crate) struct SpelunkPath {
    keys: map::KeyRing,
    path: Vec<char>,
    location: Point,
    distance: usize,
}

impl SpelunkPath {
    fn start(origin: Point) -> Self {
        Self {
            keys: map::KeyRing::default(),
            path: Vec::new(),
            location: origin,
            distance: 0,
        }
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

#[derive(Debug, Clone)]
pub(crate) struct Spelunker<'m> {
    caves: &'m map::Map,
    graph: &'m graph::Graph<'m, map::Map>,
    path: SpelunkPath,
    heuristic: Cell<Option<usize>>,
}

impl<'m> SearchCandidate for Spelunker<'m> {
    fn is_complete(&self) -> bool {
        self.path.keys.len() == self.caves.keys().len()
    }

    fn score(&self) -> usize {
        self.distance()
    }

    fn children(&self) -> Vec<Self> {
        self.candidates().unwrap()
    }
}

impl<'m> SearchCacher for Spelunker<'m> {
    type State = SpelunkState;

    fn state(&self) -> SpelunkState {
        SpelunkState(self.path.keys.state(), self.location().unwrap())
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
            if self.path.keys.contains(&key.door) {
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

impl<'m> Spelunker<'m> {
    fn new(map: &'m map::Map, graph: &'m graph::Graph<'m, map::Map>) -> Self {
        Self {
            caves: map,
            graph: graph,
            path: SpelunkPath::start(map.entrance().unwrap()),
            heuristic: Cell::new(None),
        }
    }

    fn location(&self) -> Result<Point, Error> {
        Ok(self.path.location)
    }

    fn candidates(&self) -> Result<Vec<Spelunker<'m>>, Error> {
        let mut candidates = Vec::with_capacity(4);

        for (point, path) in self.graph.edges(self.location()?) {
            match self.caves.get(*point) {
                Some(map::Tile::Key(c)) => {
                    let mut newsp = self.clone();
                    newsp.path.found_key(c);
                    newsp.path.location = *point;
                    newsp.path.distance += path.distance();
                    candidates.push(newsp);
                }
                Some(map::Tile::Door(c)) if self.path.keys.contains(&c) => {
                    let mut newsp = self.clone();
                    newsp.path.location = *point;
                    newsp.path.distance += path.distance();
                    candidates.push(newsp);
                }
                Some(map::Tile::Entrance) => {
                    let mut newsp = self.clone();
                    newsp.path.location = *point;
                    newsp.path.distance += path.distance();
                    candidates.push(newsp);
                }
                Some(map::Tile::Door(_)) => {}
                Some(map::Tile::Hall) => {
                    let mut newsp = self.clone();
                    newsp.path.location = *point;
                    newsp.path.distance += path.distance();
                    candidates.push(newsp);
                }
                None => {}
            }
        }

        Ok(candidates)
    }

    fn distance(&self) -> usize {
        self.path.distance
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

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct MultiSpelunkState(String, [Point; 4]);

#[derive(Debug, Clone)]
pub(crate) struct MultiSpelunkPath {
    keys: map::KeyRing,
    path: Vec<char>,
    locations: [Point; 4],
    distance: usize,
}

impl MultiSpelunkPath {
    fn start(origins: [Point; 4]) -> Self {
        Self {
            keys: map::KeyRing::default(),
            path: Vec::new(),
            locations: origins,
            distance: 0,
        }
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

#[derive(Debug, Clone)]
struct MultiSpelunker<'m> {
    caves: &'m map::MultiMap,
    graphs: [&'m graph::Graph<'m, map::MultiMap>; 4],
    path: MultiSpelunkPath,
}

impl<'m> SearchCandidate for MultiSpelunker<'m> {
    fn is_complete(&self) -> bool {
        self.path.keys.len() == self.caves.keys().len()
    }

    fn score(&self) -> usize {
        self.distance()
    }

    fn children(&self) -> Vec<Self> {
        self.candidates()
    }
}

impl<'m> SearchCacher for MultiSpelunker<'m> {
    type State = MultiSpelunkState;

    fn state(&self) -> MultiSpelunkState {
        MultiSpelunkState(self.path.keys.state(), self.path.locations)
    }
}

impl<'m> MultiSpelunker<'m> {
    fn new(
        map: &'m map::MultiMap,
        graphs: [&'m graph::Graph<'m, map::MultiMap>; 4],
        origins: [Point; 4],
    ) -> Self {
        Self {
            caves: map,
            graphs: graphs,
            path: MultiSpelunkPath::start(origins),
        }
    }

    fn candidates(&self) -> Vec<MultiSpelunker<'m>> {
        let mut candidates = Vec::new();

        for (i, (location, graph)) in self
            .path
            .locations
            .iter()
            .zip(self.graphs.iter())
            .enumerate()
        {
            for (point, path) in graph.edges(*location) {
                match self.caves.get(*point) {
                    Some(map::Tile::Key(c)) => {
                        let mut newsp = self.clone();
                        newsp.path.found_key(c);
                        newsp.path.locations[i] = *point;
                        newsp.path.distance += path.distance();
                        candidates.push(newsp);
                    }
                    Some(map::Tile::Door(c)) if self.path.keys.contains(&c) => {
                        let mut newsp = self.clone();
                        newsp.path.locations[i] = *point;
                        newsp.path.distance += path.distance();
                        candidates.push(newsp);
                    }
                    Some(map::Tile::Entrance) => {
                        let mut newsp = self.clone();
                        newsp.path.locations[i] = *point;
                        newsp.path.distance += path.distance();
                        candidates.push(newsp);
                    }
                    Some(map::Tile::Door(_)) => {}
                    Some(map::Tile::Hall) => {
                        let mut newsp = self.clone();
                        newsp.path.locations[i] = *point;
                        newsp.path.distance += path.distance();
                        candidates.push(newsp);
                    }
                    None => {}
                }
            }
        }

        candidates
    }

    fn distance(&self) -> usize {
        self.path.distance
    }
}

fn multisearch<'m>(map: &'m map::MultiMap) -> Result<MultiSpelunkPath, Error> {
    use geometry::coord2d::graph::Graphable;
    use searcher::SearchOptions;

    use std::convert::TryInto;

    let entrances = map.entrances();

    let graphs: Vec<_> = entrances.iter().map(|e| map.graph(*e)).collect();

    {
        let grefs: [&graph::Graph<map::MultiMap>; 4] = graphs
            .iter()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| anyhow!("Can't form graph ref"))?;

        let origin = MultiSpelunker::new(map, grefs, entrances.clone());

        let options = {
            let mut o = SearchOptions::default();
            o.verbose = Some(10_000);
            o
        };

        Ok(searcher::dijkstra::build(origin)
            .with_options(options)
            .run()
            .map(|c| c.path)?)
    }
}

fn search<'m>(map: &'m map::Map) -> Result<SpelunkPath, Error> {
    use geometry::coord2d::graph::Graphable;

    let graph = map.graph(map.entrance().ok_or(anyhow!("No entrance?"))?);
    let origin = Spelunker::new(map, &graph);

    Ok(searcher::dijkstra::run(origin).map(|c| c.path)?)
}

mod map {
    use anyhow::{anyhow, Error};
    use geometry::coord2d::{BoundingBox, Point};

    use geometry::coord2d::graph;
    use geometry::coord2d::pathfinder;

    use lazy_static::lazy_static;

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

            Ok(Map::new(tiles))
        }
    }

    impl Map {
        fn new(tiles: HashMap<Point, Tile>) -> Self {
            Map {
                tiles,
                ..Map::default()
            }
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

    impl graph::Graphable for Map {
        fn is_node(&self, point: &Point) -> bool {
            let options = self.movement_options(point);
            match self.get(*point) {
                Some(Tile::Door(_)) => true,
                Some(Tile::Key(_)) => true,
                Some(Tile::Entrance) => true,
                Some(Tile::Hall) => options == 1 || options > 2,
                None => false,
            }
        }
    }

    impl pathfinder::Map for Map {
        fn is_traversable(&self, location: Point) -> bool {
            self.get(location).is_some()
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct MultiMap(Map, HashMap<Point, Option<Tile>>, [Point; 4]);

    impl FromStr for MultiMap {
        type Err = Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let map: Map = s.parse()?;
            Ok(MultiMap::new(map))
        }
    }

    impl MultiMap {
        pub(crate) fn new(map: Map) -> Self {
            lazy_static! {
                static ref OFFSETS: [Point; 4] = [
                    (1, 1).into(),
                    (1, -1).into(),
                    (-1, 1).into(),
                    (-1, -1).into()
                ];
            }

            let entrance = map.entrance().unwrap();

            let entrances: [Point; 4] = {
                let v: Vec<Point> = OFFSETS
                    .iter()
                    .map(|p| Point::new(entrance.x + p.x, entrance.y + p.y))
                    .collect();
                v.try_into().unwrap_or_else(|v: Vec<Point>| {
                    panic!("Expected a Vec of length {} but it was {}", 4, v.len())
                })
            };

            let mut overrides = HashMap::new();
            {
                let mut bbox = BoundingBox::empty();
                bbox.include(entrance);
                bbox = bbox.margin(1);

                for p in bbox.points() {
                    if bbox.edge(p).map(|e| e.is_corner()).unwrap_or(false) {
                        overrides.insert(p, Some(Tile::Entrance));
                    } else {
                        overrides.insert(p, None);
                    }
                }
            }

            MultiMap(map, overrides, entrances)
        }

        pub(crate) fn entrances(&self) -> &[Point; 4] {
            &self.2
        }

        pub(crate) fn keys(&self) -> HashSet<Key> {
            self.0.keys()
        }

        pub(crate) fn get(&self, location: Point) -> Option<Tile> {
            match self.1.get(&location) {
                Some(t) => t.clone(),
                None => self.0.get(location),
            }
        }
    }

    impl graph::Graphable for MultiMap {
        fn is_node(&self, point: &Point) -> bool {
            let options = self.movement_options(point);
            match self.get(*point) {
                Some(Tile::Door(_)) => true,
                Some(Tile::Key(_)) => true,
                Some(Tile::Entrance) => true,
                Some(Tile::Hall) => options == 1 || options > 2,
                None => false,
            }
        }
    }

    impl pathfinder::Map for MultiMap {
        fn is_traversable(&self, location: Point) -> bool {
            self.get(location).is_some()
        }
    }
}

pub(crate) fn main(mut input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let map: map::Map = {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf.parse()?
    };

    {
        let start = time::Instant::now();

        let sp = search(&map)?;
        println!("Part 1: {}", sp.distance());
        println!("  Keys: {}", sp.keys().to_string());
        println!("  Time: {}s", start.elapsed().as_secs());
    }

    {
        let start = time::Instant::now();
        let mm = map::MultiMap::new(map);

        let sp = multisearch(&mm)?;
        println!("Part 2: {}", sp.distance());
        println!("  Keys: {}", sp.keys().to_string());
        println!("  Time: {}s", start.elapsed().as_secs());
    }

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

    #[test]
    fn examples_part2_a() {
        let mmap: map::MultiMap = "
        #######
        #a.#Cd#
        ##...##
        ##.@.##
        ##...##
        #cB#Ab#
        #######
        "
        .parse()
        .unwrap();

        for entrance in mmap.entrances().iter() {
            assert_eq!(mmap.get(*entrance), Some(map::Tile::Entrance));
        }

        let mp = multisearch(&mmap).unwrap();
        assert_eq!(mp.distance(), 8);
    }

    #[test]
    fn examples_part2_b() {
        let mmap: map::MultiMap = "
        ###############
        #d.ABC.#.....a#
        ######...######
        ######.@.######
        ######...######
        #b.....#.....c#
        ###############
        "
        .parse()
        .unwrap();

        let mp = multisearch(&mmap).unwrap();
        assert_eq!(mp.distance(), 24);
    }

    #[test]
    fn examples_part2_c() {
        let mmap: map::MultiMap = "
        #############
        #DcBa.#.GhKl#
        #.###...#I###
        #e#d#.@.#j#k#
        ###C#...###J#
        #fEbA.#.FgHi#
        #############
        "
        .parse()
        .unwrap();

        let mp = multisearch(&mmap).unwrap();
        assert_eq!(mp.distance(), 32);
    }

    #[test]
    fn examples_part2_d() {
        let mmap: map::MultiMap = "
        #############
        #g#f.D#..h#l#
        #F###e#E###.#
        #dCba...BcIJ#
        #####.@.#####
        #nK.L...G...#
        #M###N#H###.#
        #o#m..#i#jk.#
        #############
        "
        .parse()
        .unwrap();

        let mp = multisearch(&mmap).unwrap();
        assert_eq!(mp.distance(), 72);
    }
}
