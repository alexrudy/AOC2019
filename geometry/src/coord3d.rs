use crate::Position;
use std::convert::From;

#[derive(Debug, Copy, Clone)]
pub struct Point3D {
    pub x: Position,
    pub y: Position,
    pub z: Position,
}

impl Point3D {
    fn new(x: Position, y: Position, z: Position) -> Self {
        Point3D { x, y, z }
    }

    pub fn origin() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

impl From<(Position, Position, Position)> for Point3D {
    fn from(coordinates: (Position, Position, Position)) -> Self {
        Self {
            x: coordinates.0,
            y: coordinates.1,
            z: coordinates.2,
        }
    }
}
