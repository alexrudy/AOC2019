use anyhow::{anyhow, Error};
use geometry::coord2d::{BoundingBox, Point};

use geometry::coord2d::graph;
use geometry::coord2d::pathfinder;

use lazy_static::lazy_static;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use std::string::ToString;

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

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Tile::Hall => '.',
                Tile::Entrance => '@',
                Tile::Door(c) => c.to_ascii_uppercase(),
                Tile::Key(c) => *c,
            }
        )
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

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub(crate) struct KeyRing(BTreeSet<char>);

impl KeyRing {
    pub(crate) fn insert(&mut self, key: char) -> bool {
        self.0.insert(key)
    }

    pub(crate) fn contains(&self, key: &char) -> bool {
        self.0.contains(key)
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl ToString for KeyRing {
    fn to_string(&self) -> String {
        self.0.iter().collect()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Map {
    tiles: BTreeMap<Point, Tile>,
    n_keys: usize,
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tiles = BTreeMap::new();

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

pub(crate) trait TileMap: Sized {
    fn get(&self, location: Point) -> Option<Tile>;

    fn bbox(&self) -> BoundingBox;

    fn printer(&self) -> Printer<Self> {
        Printer(&self)
    }
}

impl Map {
    fn new(tiles: BTreeMap<Point, Tile>) -> Self {
        let n_keys = tiles.values().filter(|t| t.is_key()).count();

        Map { tiles, n_keys }
    }

    pub(crate) fn n_keys(&self) -> usize {
        self.n_keys
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

    #[allow(dead_code)]
    pub(crate) fn key_lookup(&self) -> HashMap<char, Point> {
        self.keys().iter().map(|k| (k.door, k.location)).collect()
    }

    pub(crate) fn entrance(&self) -> Option<Point> {
        self.tiles.iter().find_map(|(p, t)| match t {
            Tile::Entrance => Some(*p),
            _ => None,
        })
    }
}

impl TileMap for Map {
    fn get(&self, location: Point) -> Option<Tile> {
        self.tiles.get(&location).copied()
    }

    fn bbox(&self) -> BoundingBox {
        BoundingBox::from_points(self.tiles.keys())
    }
}

pub(crate) struct Printer<'m, M>(&'m M)
where
    M: TileMap;

impl<'m, M> std::fmt::Display for Printer<'m, M>
where
    M: TileMap,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bbox = self.0.bbox();

        bbox.printer(f, |f, p| {
            write!(
                f,
                "{}",
                match self.0.get(*p) {
                    Some(Tile::Hall) => '.',
                    Some(Tile::Entrance) => '@',
                    Some(Tile::Door(c)) => c.to_ascii_uppercase(),
                    Some(Tile::Key(c)) => c,
                    None => '#',
                }
            )
        })
    }
}

impl graph::Graphable for Map {
    fn is_node(&self, point: &Point) -> bool {
        match self.get(*point) {
            Some(Tile::Door(_)) => true,
            Some(Tile::Key(_)) => true,
            Some(Tile::Entrance) => true,
            Some(Tile::Hall) => false,
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
                (-1, -1).into(),
                (-1, 1).into(),
                (1, -1).into(),
                (1, 1).into(),
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

    pub(crate) fn n_keys(&self) -> usize {
        self.0.n_keys
    }

    pub(crate) fn entrances(&self) -> &[Point; 4] {
        &self.2
    }

    #[allow(dead_code)]
    pub(crate) fn quadrant(&self, location: Point) -> usize {
        let Point { x, y } = self.0.entrance().unwrap().offset(location);

        match (x > 0, y > 0) {
            (true, true) => 1,
            (true, false) => 2,
            (false, true) => 3,
            (false, false) => 4,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn key_lookup(&self) -> HashMap<char, Point> {
        self.0.key_lookup()
    }

    #[allow(dead_code)]
    pub(crate) fn neighbors(&self, location: Point) -> usize {
        if self.get(location).is_some() {
            location
                .adjacent()
                .filter(|p| self.get(*p).is_some())
                .count()
        } else {
            0
        }
    }
}

impl TileMap for MultiMap {
    fn get(&self, location: Point) -> Option<Tile> {
        match self.1.get(&location) {
            Some(t) => t.clone(),
            None => self.0.get(location),
        }
    }
    fn bbox(&self) -> BoundingBox {
        BoundingBox::from_points(self.0.tiles.keys())
    }
}

impl graph::Graphable for MultiMap {
    fn is_node(&self, point: &Point) -> bool {
        match self.get(*point) {
            Some(Tile::Door(_)) => true,
            Some(Tile::Key(_)) => true,
            Some(Tile::Entrance) => true,
            Some(Tile::Hall) => false,
            // Some(Tile::Hall) => self.neighbors(*point) > 2,
            None => false,
        }
    }
}

impl pathfinder::Map for MultiMap {
    fn is_traversable(&self, location: Point) -> bool {
        self.get(location).is_some()
    }
}
