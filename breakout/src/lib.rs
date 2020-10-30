use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::Error;
use cursive::event::Key;
use cursive::view::Boxable;
use cursive::view::SizeConstraint;
use cursive::views::{LinearLayout, OnEventView, Panel, ResizedView, TextView};
use cursive::{ncurses, Cursive};

mod game;
mod view;

use crate::game::State;
use crate::view::{MessageClient, MessageView, ScoreView, ScreenView};
pub use game::{Breakout, Screen, Tile};

pub fn start(game: Breakout) -> Result<(), Error> {
    let mut app = ncurses()?;
    app.set_fps(30);
    app.add_global_callback('q', Cursive::quit);

    let engine = Engine::build(game, &mut app);

    eprintln!("Starting worker...");
    thread::spawn(move || worker(engine));

    app.run();
    Ok(())
}

struct Engine {
    breakout: Breakout,
    screen: Arc<Mutex<Screen>>,
    score: Arc<Mutex<i32>>,
    message: MessageClient,
    controller: mpsc::Receiver<Key>,
}

impl Engine {
    fn new(breakout: Breakout, message: MessageClient, controller: mpsc::Receiver<Key>) -> Self {
        let screen = breakout.screen().clone();
        Engine {
            breakout: breakout,
            screen: Arc::new(Mutex::new(screen)),
            score: Arc::new(Mutex::new(0)),
            message: message,
            controller: controller,
        }
    }

    fn build(breakout: Breakout, app: &mut Cursive) -> Self {
        let (etx, erx) = mpsc::channel();
        let (tx, message) = MessageView::pair();
        let engine = Engine::new(breakout, tx, erx);

        let title = TextView::new("Breakout!").center().fixed_height(1);
        let screen = Panel::new(ScreenView::new(engine.screen.clone()));
        let score = ScoreView::new(engine.score.clone()).fixed_height(1);
        let message = ResizedView::new(SizeConstraint::Free, SizeConstraint::Fixed(1), message);

        let layout = LinearLayout::vertical()
            .child(title)
            .child(screen)
            .child(score)
            .child(message);

        let etx_left = etx.clone();
        let etx_right = etx.clone();

        app.add_layer(
            OnEventView::new(layout)
                .on_event(Key::Left, move |_| etx_left.send(Key::Left).unwrap())
                .on_event(Key::Right, move |_| etx_right.send(Key::Right).unwrap()),
        );
        engine
    }
}

fn worker(mut engine: Engine) -> () {
    loop {
        eprintln!("Running arcade");
        match engine.breakout.step().unwrap() {
            State::Input => match engine.controller.try_recv() {
                Ok(Key::Left) => engine.breakout.feed(-1),
                Ok(Key::Right) => engine.breakout.feed(1),
                Ok(_) => engine.breakout.feed(0),
                Err(mpsc::TryRecvError::Empty) => engine.breakout.feed(0),
                Err(_) => break,
            },
            State::Halt => break,
        }
        thread::sleep(Duration::from_millis(300));
        eprintln!("Updating screen");
        *engine.screen.lock().unwrap() = engine.breakout.screen().clone();
        *engine.score.lock().unwrap() = engine.breakout.screen().score() as i32;
    }
    eprintln!("Finished worker");
}
