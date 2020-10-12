use anyhow::{anyhow, Error};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
struct Node {
    name: String,
}

impl Node {
    fn new<S: Into<String>>(s: S) -> Node {
        Node { name: s.into() }
    }

    fn origin() -> Node {
        Node::new("COM")
    }
}

#[derive(Debug, Default)]
struct Tree {
    nodes: HashMap<Node, Node>,
}

fn parse_line(s: &str) -> Result<(Node, Node), Error> {
    let mut parts = s.trim().split(')');
    let parent = parts.next().ok_or(anyhow!("Missing parent node name!"))?;
    let child = parts.next().ok_or(anyhow!("Missing child node name!"))?;

    Ok((Node::new(parent), Node::new(child)))
}

impl Tree {
    fn insert(&mut self, parent: Node, child: Node) -> () {
        self.nodes.insert(child, parent);
    }

    fn walk(&self, start: Node) -> OrbitWalker {
        OrbitWalker {
            tree: self,
            location: Some(start),
        }
    }

    fn orbits(&self, target: Node) -> usize {
        self.walk(target).count()
    }

    fn total_orbits(&self) -> usize {
        self.nodes
            .keys()
            .map(|node| self.orbits(node.clone()))
            .sum()
    }

    fn common_ancestor(&self, left: Node, right: Node) -> Option<Node> {
        let mut seen = HashSet::new();
        let mut left_iter = self.walk(left);
        let mut right_iter = self.walk(right);

        loop {
            if let Some(loc) = left_iter.next() {
                if !seen.insert(loc.clone()) {
                    return Some(loc);
                }
            } else if let Some(loc) = right_iter.next() {
                if !seen.insert(loc.clone()) {
                    return Some(loc);
                }
            } else {
                return None;
            }
        }
    }

    fn orbit_route(&self, start: Node, finish: Node) -> Option<usize> {
        if let Some(common) = self.common_ancestor(start.clone(), finish.clone()) {
            Some(self.orbits(start) + self.orbits(finish) - 2 * self.orbits(common) - 2)
        } else {
            None
        }
    }
}

impl FromStr for Tree {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tree = Tree::default();
        for line in s.lines() {
            let (parent, child) = parse_line(line)?;
            tree.insert(parent, child);
        }
        Ok(tree)
    }
}

struct OrbitWalker<'t> {
    tree: &'t Tree,
    location: Option<Node>,
}

impl<'t> Iterator for OrbitWalker<'t> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        // Never travel from the origin
        if Some(Node::origin()) == self.location {
            return None;
        };

        // Walk around the orbit map
        self.location = match self.location {
            Some(ref loc) => self.tree.nodes.get(loc).map(|l| l.clone()),
            None => None,
        };
        self.location.clone()
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let reader = BufReader::new(input);

    let mut tree = Tree::default();
    for line in reader.lines() {
        let (parent, child) = parse_line(&line?)?;

        tree.insert(parent, child);
    }

    println!("Part 1: Total orbits {}", tree.total_orbits());

    println!(
        "Part 2: Transfers: {}",
        tree.orbit_route(Node::new("YOU"), Node::new("SAN"))
            .ok_or(anyhow!("No orbit route found!"))?
    );

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples_part1() {
        let tree: Tree = "COM)B
        B)C
        C)D
        D)E
        E)F
        B)G
        G)H
        D)I
        E)J
        J)K
        K)L"
        .parse()
        .unwrap();

        assert_eq!(tree.total_orbits(), 42);
    }

    #[test]
    fn examples_part2() {
        let tree: Tree = "COM)B
        B)C
        C)D
        D)E
        E)F
        B)G
        G)H
        D)I
        E)J
        J)K
        K)L
        K)YOU
        I)SAN"
            .parse()
            .unwrap();

        assert_eq!(
            tree.common_ancestor(Node::new("YOU"), Node::new("SAN")),
            Some(Node::new("D"))
        );
        assert_eq!(
            tree.orbit_route(Node::new("YOU"), Node::new("SAN")),
            Some(4)
        )
    }
}
