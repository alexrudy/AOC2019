use cursive::traits::*;
use cursive::Printer;
use cursive::XY;

use geometry::coord2d::Edge;

use crate::{Screen, Tile};

pub(crate) struct ScreenView {
    screen: Screen,
}

impl ScreenView {
    pub(crate) fn new(screen: Screen) -> Self {
        ScreenView { screen }
    }
}

impl View for ScreenView {
    fn draw(&self, printer: &Printer) {
        let board_bbox = self.screen.bbox();
        let bbox = board_bbox.clone().margin(1);
        for (j, y) in bbox.vertical().enumerate() {
            for (i, x) in bbox.horizontal().enumerate() {
                let tile = match self.screen.get(&(x, y).into()).unwrap_or(&Tile::Empty) {
                    Tile::Empty => " ",
                    Tile::Block => "█",
                    Tile::Ball => "●",
                    Tile::Wall => match board_bbox.edge((x, y).into()) {
                        Some(Edge::TopLeft) => "┏",
                        Some(Edge::TopRight) => "┓",
                        Some(Edge::BottomLeft) => "┗",
                        Some(Edge::BottomRight) => "┛",
                        Some(Edge::Left) => "┃",
                        Some(Edge::Right) => "┃",
                        Some(Edge::Top) => "━",
                        Some(Edge::Bottom) => "━",
                        None => "+",
                    },
                    Tile::Paddle => "─",
                };
                eprintln!("({},{}) = {}", x, y, tile);
                printer.print((i, j), tile)
            }
        }
    }

    fn required_size(&mut self, _constraint: XY<usize>) -> XY<usize> {
        let bbox = self.screen.bbox().margin(1);
        (bbox.width(), bbox.height()).into()
    }
}
