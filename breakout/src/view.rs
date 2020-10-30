use std::sync::{mpsc, Arc, Mutex};

use cursive::traits::*;
use cursive::view::ViewWrapper;
use cursive::views::{ResizedView, TextView};
use cursive::wrap_impl;
use cursive::Printer;
use cursive::Vec2;
use cursive::XY;

use geometry::coord2d::Edge;

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
}

impl ScoreView {
    pub(crate) fn new(score: Arc<Mutex<i32>>) -> Self {
        Self {
            score,
            text: TextView::new("0").center(),
        }
    }
}

pub(crate) struct MessageView {
    channel: mpsc::Receiver<String>,
    message: Option<String>,
}

pub(crate) struct MessageClient {
    channel: mpsc::Sender<String>,
}

impl MessageClient {
    pub(crate) fn set(&mut self, msg: String) -> Result<(), mpsc::SendError<String>> {
        self.channel.send(msg)
    }
}

impl MessageView {
    fn new(channel: mpsc::Receiver<String>) -> Self {
        Self {
            channel,
            message: None,
        }
    }

    pub(crate) fn pair() -> (MessageClient, Self) {
        let (tx, rx) = mpsc::channel();
        (MessageClient { channel: tx }, Self::new(rx))
    }

    fn recieve(&mut self) {
        while let Ok(msg) = self.channel.try_recv() {
            self.message = Some(msg);
        }
    }
}

impl View for MessageView {
    fn layout(&mut self, _: Vec2) {
        // Before drawing, we'll want to update the buffer
        self.recieve();
    }

    fn draw(&self, printer: &Printer) {
        // Print the end of the buffer
        if let Some(ref msg) = self.message {
            printer.print((0, 0), msg);
        }
    }
}
