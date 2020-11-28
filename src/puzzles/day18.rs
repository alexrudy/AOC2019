use anyhow::Error;

use geometry::coord2d::Point;

use std::cmp::{Eq, PartialEq};
use std::io::Read;
use std::time;

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
pub(crate) struct KeyPath(Vec<char>);

impl ToString for KeyPath {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

mod map;
mod multi;
mod multigraph;
mod single;

pub(crate) fn main(mut input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let map: map::Map = {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf.parse()?
    };

    {
        let start = time::Instant::now();
        let mm = map::MultiMap::new(map.clone());

        let sp = multigraph::search(&mm)?;
        println!("Part 2: {}", sp.distance());
        println!("  Keys: {}", sp.keys().to_string());
        println!("  Time: {}s", start.elapsed().as_secs());
    }

    {
        let start = time::Instant::now();

        let sp = single::search(&map)?;
        println!("Part 1: {}", sp.distance());
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

        let sp = single::search(&map).unwrap();
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

        let sp = single::search(&map).unwrap();
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

        let sp = single::search(&map).unwrap();
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

        let sp = single::search(&map).unwrap();
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

        let sp = single::search(&map).unwrap();
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

        let sp = single::search(&map).unwrap();
        eprintln!("{}", sp.keys().to_string());
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

        {
            let mp = multi::search(&mmap).unwrap();
            assert_eq!(mp.distance(), 8);
        }

        {
            let mp = multigraph::search(&mmap).unwrap();
            assert_eq!(mp.distance(), 8);
        }
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

        {
            let mp = multi::search(&mmap).unwrap();
            assert_eq!(mp.distance(), 24);
        }

        {
            let mp = multigraph::search(&mmap).unwrap();
            assert_eq!(mp.distance(), 24);
        }
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

        {
            let mp = multi::search(&mmap).unwrap();
            assert_eq!(mp.distance(), 32);
        }

        {
            let mp = multigraph::search(&mmap).unwrap();
            assert_eq!(mp.distance(), 32);
        }
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

        {
            let mp = multi::search(&mmap).unwrap();
            assert_eq!(mp.distance(), 72);
        }

        {
            let mp = multigraph::search(&mmap).unwrap();
            assert_eq!(mp.distance(), 72);
        }
    }
}
