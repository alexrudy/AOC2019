use std::sync::{Arc, Mutex};

use cursive::traits::*;
use cursive::view::ViewWrapper;
use cursive::views::TextView;
use cursive::wrap_impl;
use cursive::Printer;
use cursive::Vec2;
use cursive::XY;

use geometry::coord2d::{Corner, Edge, Side};

use crate::{Screen, Tile};

pub(crate) struct ScreenView {
    screen: Arc<Mutex<Screen>>,
}

impl ScreenView {
    pub(crate) fn new(screen: Arc<Mutex<Screen>>) -> Self {
        ScreenView { screen }
    }
}

impl View for ScreenView {
    fn draw(&self, printer: &Printer) {
        let screen = self.screen.lock().expect("Screen Mutex is poisoned!");

        let board_bbox = screen.bbox();
        let bbox = board_bbox.clone().margin(1);
        for (j, y) in bbox.vertical().enumerate() {
            for (i, x) in bbox.horizontal().enumerate() {
                let tile = match screen.get(&(x, y).into()).unwrap_or(&Tile::Empty) {
                    Tile::Empty => " ",
                    Tile::Block => "█",
                    Tile::Ball => "●",
                    Tile::Wall => match board_bbox.edge((x, y).into()) {
                        Some(Edge::Corner(Corner::TopLeft)) => "┏",
                        Some(Edge::Corner(Corner::TopRight)) => "┓",
                        Some(Edge::Corner(Corner::BottomLeft)) => "┗",
                        Some(Edge::Corner(Corner::BottomRight)) => "┛",
                        Some(Edge::Side(Side::Left)) => "┃",
                        Some(Edge::Side(Side::Right)) => "┃",
                        Some(Edge::Side(Side::Top)) => "━",
                        Some(Edge::Side(Side::Bottom)) => "━",
                        None => "+",
                    },
                    Tile::Paddle => "─",
                };
                printer.print((i, j), tile)
            }
        }
    }

    fn required_size(&mut self, _constraint: XY<usize>) -> XY<usize> {
        let bbox = self
            .screen
            .lock()
            .expect("Screen Mutex is poisoned!")
            .bbox()
            .margin(1);
        (bbox.width(), bbox.height()).into()
    }
}

pub(crate) struct ScoreView {
    score: Arc<Mutex<i32>>,
    text: TextView,
}

impl ViewWrapper for ScoreView {
    wrap_impl!(self.text: TextView);

    fn wrap_layout(&mut self, size: Vec2) {
        self.text
            .set_content(format!("{}", self.score.lock().unwrap()));
        self.text.layout(size)
    }
}

impl ScoreView {
    pub(crate) fn new(score: Arc<Mutex<i32>>) -> Self {
        Self {
            score,
            text: TextView::new("0").center(),
        }
    }
}
