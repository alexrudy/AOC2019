//! Coordinate work in two dimensions.

#![allow(dead_code)]

use std::cmp;
use std::default::Default;
use std::fmt;
use std::ops::{self, RangeInclusive};
use std::str::FromStr;

use itertools::iproduct;
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

use crate::Position;

pub mod graph;
pub mod map;
pub mod path;
pub mod pathfinder;

/// A movement direction in two dimensions.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Left,
    Direction::Right,
    Direction::Down,
];

impl Direction {
    /// Enumertates all directions of movement in "reading order",
    /// i.e. such that the resulting points are in reading order
    /// from the current position.
    pub fn all() -> impl Iterator<Item = Self> {
        DIRECTIONS.iter().cloned()
    }

    /// Rotates the direction as if it turned left
    pub fn turn_left(&self) -> Direction {
        match self {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }

    /// Rotates the direction as if it turned right
    pub fn turn_right(&self) -> Direction {
        match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }

    pub fn reverse(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

/// A location in 2D space.
///
/// Essentially a 2-tuple of x and y position,
/// but with a lot of provided methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: Position,
    pub y: Position,
}

impl Point {
    /// Build a new point from coordinates.
    pub fn new(x: Position, y: Position) -> Self {
        Self { x, y }
    }

    /// Returns a point at (0, 0)
    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }

    /// Compare this point to another in "reading order"
    /// which is y then x.
    pub fn reading_order(self, other: Point) -> cmp::Ordering {
        self.y.cmp(&other.y).then(self.x.cmp(&other.x)).reverse()
    }

    fn up(self) -> Self {
        Self {
            x: self.x,
            y: self.y - 1,
        }
    }

    fn down(self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
    }

    fn left(self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y,
        }
    }

    fn right(self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
        }
    }

    /// Step in a given direction.
    pub fn step(self, direction: Direction) -> Self {
        match direction {
            Direction::Left => self.left(),
            Direction::Right => self.right(),
            Direction::Up => self.up(),
            Direction::Down => self.down(),
        }
    }

    /// Iterate over all adjacent points.
    pub fn adjacent(self) -> impl Iterator<Item = Self> {
        Direction::all().map(move |d| self.step(d))
    }

    /// Check if a point is adjacent.
    pub fn is_adjacent(&self, point: &Point) -> bool {
        self.manhattan_distance(*point) == 1
    }

    /// Iterate over all diagonally adjacent points
    pub fn adjacent_diagonal(self) -> impl Iterator<Item = Self> {
        iproduct!(-1..2, -1..2)
            .filter(|(x, y)| !(*x == 0 && *y == 0))
            .map(move |(x, y)| Point::new(self.x + x, self.y + y))
    }

    /// Manhattan distance between two points is the distance along
    /// each coordinate
    pub fn manhattan_distance(self, other: Point) -> Position {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Compute offset for this point
    pub fn offset(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    /// What direction connects these two points?
    ///
    /// If they are not adjacent, return `None`.
    pub fn direction(self, other: Point) -> Option<Direction> {
        match (self.x, self.y) {
            (x, y) if x == other.x + 1 && y == other.y => Some(Direction::Left),
            (x, y) if x == other.x - 1 && y == other.y => Some(Direction::Right),
            (x, y) if x == other.x && y == other.y + 1 => Some(Direction::Up),
            (x, y) if x == other.x && y == other.y - 1 => Some(Direction::Down),
            _ => None,
        }
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::origin()
    }
}

impl ops::Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl cmp::Ord for Point {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.y.cmp(&other.y).then(self.x.cmp(&other.x))
    }
}

impl cmp::PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl From<(Position, Position)> for Point {
    fn from(coordinates: (Position, Position)) -> Self {
        Self {
            x: coordinates.0,
            y: coordinates.1,
        }
    }
}

impl From<(usize, usize)> for Point {
    fn from(coordinates: (usize, usize)) -> Self {
        Self {
            x: coordinates.0 as Position,
            y: coordinates.1 as Position,
        }
    }
}

/// Error when parsing a point from string.
#[derive(Debug, Error)]
pub enum ParsePointError {
    #[error("Invalid Point: {}", _0)]
    InvalidLiteral(String),

    #[error("Invalid Number Literal")]
    InvalidNumber,
}

impl From<::std::num::ParseIntError> for ParsePointError {
    fn from(_: ::std::num::ParseIntError) -> Self {
        ParsePointError::InvalidNumber
    }
}

impl FromStr for Point {
    type Err = ParsePointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?P<x>[\d]+),\s*(?P<y>[\d]+)").unwrap();
        };

        let cap = match RE.captures(s) {
            None => return Err(ParsePointError::InvalidLiteral(s.to_string())),
            Some(c) => c,
        };

        Ok(Self::new(cap["x"].parse()?, cap["y"].parse()?))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Side {
    Left,
    Top,
    Right,
    Bottom,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Describes the side of a bounding box
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Edge {
    Side(Side),
    Corner(Corner),
}

impl Edge {
    pub fn is_corner(&self) -> bool {
        matches!(self, Edge::Corner(_))
    }
}

/// A rectangle which encloses points and is aligned
/// with the coordinate axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    left: Position,
    right: Position,
    top: Position,
    bottom: Position,
}

impl BoundingBox {
    /// Create a bounding box which covers no points.
    pub fn empty() -> Self {
        Self {
            left: Position::max_value(),
            right: Position::min_value(),
            top: Position::max_value(),
            bottom: Position::min_value(),
        }
    }

    /// Create a bounding box at zero, covering only the zero point.
    pub fn zero() -> Self {
        Self {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        }
    }

    /// Constructor for a boudning box from the extent coordinates.
    pub fn new(left: Position, right: Position, top: Position, bottom: Position) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    /// Create a bounding box from the top left and bottom right corners.
    pub fn from_corners(topleft: Point, bottomright: Point) -> Self {
        Self {
            left: cmp::min(topleft.x, bottomright.x),
            right: cmp::max(topleft.x, bottomright.x),
            top: cmp::min(topleft.y, bottomright.y),
            bottom: cmp::max(topleft.y, bottomright.y),
        }
    }

    /// Modify this boundign box to include a given point.
    pub fn include(&mut self, point: Point) -> bool {
        let mut updated = false;
        if point.x < self.left {
            self.left = point.x;
            updated = true;
        }
        if point.x > self.right {
            self.right = point.x;
            updated = true;
        }
        if point.y < self.top {
            self.top = point.y;
            updated = true;
        }
        if point.y > self.bottom {
            self.bottom = point.y;
            updated = true;
        }
        updated
    }

    /// Construct a bounding box from an iterator of points.
    pub fn from_points<'a>(points: impl Iterator<Item = &'a Point>) -> Self {
        let mut bbox = Self::empty();
        for point in points {
            bbox.include(*point);
        }
        bbox
    }

    /// Combine this bounding box with another
    /// bounding box, resulting in a bounding
    /// box which contains both input boxes.
    pub fn union(&self, other: &Self) -> Self {
        Self {
            left: if self.left > other.left {
                other.left
            } else {
                self.left
            },
            right: if self.right > other.right {
                self.right
            } else {
                other.right
            },
            top: if self.top > other.top {
                other.top
            } else {
                self.top
            },
            bottom: if self.bottom > other.bottom {
                self.bottom
            } else {
                other.bottom
            },
        }
    }

    /// Return a new bounding box with a margin
    /// added to all sides. `size` is the margin
    /// on each side, i.e. adding a margin of 1 makes
    /// the bounding box bigger by 2, one on the left and one
    /// on the right.
    pub fn margin(&self, size: Position) -> Self {
        Self {
            left: self.left - size,
            right: self.right + size,
            top: self.top - size,
            bottom: self.bottom + size,
        }
    }

    /// Return a new bounding box with an added horizontal
    /// margin on both sides (e.g. adding a margin of 1 makes
    /// the bounding box bigger by 2, one on the left and one
    /// on the right.)
    pub fn horizontal_margin(&self, size: Position) -> Self {
        Self {
            left: self.left - size,
            right: self.right + size,
            top: self.top,
            bottom: self.bottom,
        }
    }

    /// Return a new bounding box with an added vertical
    /// margin on both sides (e.g. adding a margin of 1 makes
    /// the bounding box bigger by 2, one on the top and one
    /// on the bottom.)
    pub fn vertical_margin(&self, size: Position) -> Self {
        Self {
            left: self.left,
            right: self.right,
            top: self.top - size,
            bottom: self.bottom + size,
        }
    }

    /// Range of vertical positions
    pub fn vertical(&self) -> RangeInclusive<Position> {
        self.top..=self.bottom
    }

    /// Range of horizontal positions
    pub fn horizontal(&self) -> RangeInclusive<Position> {
        self.left..=self.right
    }

    /// Check if a point is contained within this bounding
    /// box, including the edges.
    pub fn contains(&self, point: Point) -> bool {
        (point.x >= self.left)
            && (point.x <= self.right)
            && (point.y >= self.top)
            && (point.y <= self.bottom)
    }

    /// Width for this box.
    pub fn width(&self) -> Position {
        self.right.saturating_sub(self.left) + 1
    }

    /// Height for this box.
    pub fn height(&self) -> Position {
        self.bottom.saturating_sub(self.top) + 1
    }

    /// Left coordinate for this box.
    pub fn left(&self) -> Position {
        self.left
    }

    /// Right coordinate for this box.
    pub fn right(&self) -> Position {
        self.right
    }

    /// Top coordinate for this box.
    pub fn top(&self) -> Position {
        self.top
    }

    /// Bottom coordinate for this box.
    pub fn bottom(&self) -> Position {
        self.bottom
    }

    /// Check if a point falls on the edge of this
    /// bounding box.
    pub fn is_edge(&self, point: Point) -> bool {
        point.x == self.left
            || point.x == self.right
            || point.y == self.top
            || point.y == self.bottom
    }

    /// Return the direction for this edge.
    ///
    /// This method prioritizes left then top directions when a point
    /// satisfies multiple edges. (left - top - right - bottom)
    pub fn edge(&self, point: Point) -> Option<Edge> {
        match (point.x, point.y) {
            (x, y) if self.left == x && self.top == y => Some(Edge::Corner(Corner::TopLeft)),
            (x, y) if self.left == x && self.bottom == y => Some(Edge::Corner(Corner::BottomLeft)),
            (x, y) if self.right == x && self.top == y => Some(Edge::Corner(Corner::TopRight)),
            (x, y) if self.right == x && self.bottom == y => {
                Some(Edge::Corner(Corner::BottomRight))
            }
            (x, _) if self.left == x => Some(Edge::Side(Side::Left)),
            (_, y) if self.top == y => Some(Edge::Side(Side::Top)),
            (x, _) if self.right == x => Some(Edge::Side(Side::Right)),
            (_, y) if self.bottom == y => Some(Edge::Side(Side::Bottom)),
            _ => None,
        }
    }

    pub fn corners(&self) -> [Point; 4] {
        [
            (self.left, self.top).into(),
            (self.right, self.top).into(),
            (self.right, self.bottom).into(),
            (self.left, self.bottom).into(),
        ]
    }

    pub fn corner(&self, corner: Corner) -> Point {
        match corner {
            Corner::TopLeft => (self.left, self.top).into(),
            Corner::TopRight => (self.right, self.top).into(),
            Corner::BottomLeft => (self.left, self.bottom).into(),
            Corner::BottomRight => (self.bottom, self.right).into(),
        }
    }

    /// Iterate through all the points contained in this
    /// bounding box
    pub fn points(&self) -> BoundingBoxIterator {
        BoundingBoxIterator {
            bbox: self,
            px: 0,
            py: 0,
        }
    }

    /// Call a function which should write a single character at every position
    /// in this bounding box.
    ///
    /// This function will handle newlines. The callback should print
    /// a single character for each point.
    pub fn printer<F>(&self, f: &mut fmt::Formatter, cb: F) -> fmt::Result
    where
        F: Fn(&mut fmt::Formatter, &Point) -> fmt::Result,
    {
        for y in self.vertical() {
            for x in self.horizontal() {
                let point = (x, y).into();
                cb(f, &point)?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<BoundingBox> for (Position, Position, Position, Position) {
    fn from(bbox: BoundingBox) -> Self {
        (bbox.left, bbox.right, bbox.top, bbox.bottom)
    }
}

impl From<(Position, Position, Position, Position)> for BoundingBox {
    fn from(bbox: (Position, Position, Position, Position)) -> Self {
        BoundingBox::new(bbox.0, bbox.1, bbox.2, bbox.3)
    }
}

/// Implements iteration over the points in a bounding
/// box.
pub struct BoundingBoxIterator<'b> {
    bbox: &'b BoundingBox,
    px: i32,
    py: i32,
}

impl<'b> Iterator for BoundingBoxIterator<'b> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.px == self.bbox.width() {
            self.px = 0;
            self.py += 1;
        }

        if self.py >= self.bbox.height() {
            return None;
        }

        let result = Some(Point {
            x: self.px + self.bbox.left(),
            y: self.py + self.bbox.top(),
        });
        self.px += 1;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point() {
        let point = Point::new(1, 1);

        assert_eq!(point.step(Direction::Up), Point::new(1, 0));
        assert_eq!(point.step(Direction::Down), Point::new(1, 2));
        assert_eq!(point.step(Direction::Left), Point::new(0, 1));
        assert_eq!(point.step(Direction::Right), Point::new(2, 1));

        assert_eq!(&point.to_string(), "1,1");

        assert_eq!(
            point.adjacent().collect::<Vec<_>>(),
            vec![
                Point::new(1, 0),
                Point::new(0, 1),
                Point::new(2, 1),
                Point::new(1, 2)
            ]
        );

        assert_eq!(
            point.adjacent_diagonal().collect::<Vec<_>>(),
            vec![
                Point::new(0, 0),
                Point::new(0, 1),
                Point::new(0, 2),
                Point::new(1, 0),
                Point::new(1, 2),
                Point::new(2, 0),
                Point::new(2, 1),
                Point::new(2, 2)
            ]
        );
    }

    #[test]
    fn direction() {
        let origin = Point::new(0, 0);

        let mut steps = Vec::new();
        for direction in Direction::all() {
            steps.push(origin.step(direction));
        }

        assert_eq!(
            steps,
            vec![
                Point::new(0, -1),
                Point::new(-1, 0),
                Point::new(1, 0),
                Point::new(0, 1)
            ]
        );

        let mut others = steps.clone();
        others.reverse();
        steps.sort_by(|s, o| s.reading_order(*o));
        assert_eq!(steps, others);
    }

    #[test]
    fn bbox() {
        let mut bbox = BoundingBox::empty();

        let point = Point::new(1, 2);

        bbox.include(point);
        assert_eq!(bbox.left(), 1);
        assert_eq!(bbox.right(), 1);
        assert_eq!(bbox.width(), 1);
        assert_eq!(bbox.top(), 2);
        assert_eq!(bbox.bottom(), 2);
        assert_eq!(bbox.height(), 1);

        assert_eq!(bbox.horizontal(), 1..=1);
        assert_eq!(bbox.vertical(), 2..=2);

        bbox.include(Point::new(2, 2));

        assert_eq!(bbox.left(), 1);
        assert_eq!(bbox.right(), 2);
        assert_eq!(bbox.width(), 2);
        assert_eq!(bbox.top(), 2);
        assert_eq!(bbox.bottom(), 2);
        assert_eq!(bbox.height(), 1);

        assert_eq!(bbox.horizontal(), 1..=2);
        assert_eq!(bbox.vertical(), 2..=2);

        let other_bbox = BoundingBox {
            left: 3,
            right: 5,
            top: 0,
            bottom: 2,
        };

        assert!(!other_bbox.contains(point));

        let combined = bbox.union(&other_bbox);
        assert!(combined.contains(point));
        assert_eq!(combined.left(), 1);
        assert_eq!(combined.right(), 5);
        assert_eq!(combined.width(), 5);
        assert_eq!(combined.top(), 0);
        assert_eq!(combined.bottom(), 2);
        assert_eq!(combined.height(), 3);
    }
}
