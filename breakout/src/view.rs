use cursive::traits::*;
use cursive::Printer;
use cursive::XY;

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
                    Tile::Block => "X",
                    Tile::Ball => "o",
                    Tile::Wall => "|",
                    Tile::Paddle => "_",
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
