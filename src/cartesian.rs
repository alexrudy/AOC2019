use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }

    pub fn manhattan(&self, other: &Point) -> i32 {
        i32::abs(self.x - other.x) + i32::abs(self.y - other.y)
    }

    pub fn offset(&self, other: &Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

}

impl From<(usize, usize)> for Point {
    fn from(coordinates: (usize, usize)) -> Self {
        Self { x: coordinates.0 as i32, y: coordinates.1 as i32}
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Point {
    pub fn step(&self, direction: &Direction) -> Point {
        match direction {
            Direction::Up => Point {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Down => Point {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Left => Point {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Point {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
    pub fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
}
