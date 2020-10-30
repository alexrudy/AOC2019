use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::Error;
use cursive::event::Key;
use cursive::view::Boxable;
use cursive::views::{BoxedView, LinearLayout, OnEventView, Panel, TextView};
use cursive::{ncurses, Cursive};

use intcode::Program;

mod game;
mod view;

use crate::game::State;
use crate::view::{ScoreView, ScreenView};
pub use game::{Breakout, Controller, Joystick, Screen, SimpleController, Tile};

pub fn arcade(program: Program, ai: bool) -> Result<(), Error> {
    let mut app = ncurses()?;
    app.set_fps(30);
    app.add_global_callback('q', Cursive::quit);

    let engine = if ai {
        Engine::build_ai(program, &mut app)
    } else {
        Engine::build(program, &mut app)
    };

    thread::spawn(move || worker(engine));

    app.run();
    Ok(())
}

#[derive(Debug)]
struct CursiveController {
    channel: mpsc::Receiver<Key>,
}

impl CursiveController {
    fn new(channel: mpsc::Receiver<Key>) -> Self {
        Self { channel }
    }
}

impl Controller for CursiveController {
    fn control(&self, _screen: &Screen) -> Joystick {
        let mut key = Joystick::Neutral;
        loop {
            match self.channel.try_recv() {
                Ok(Key::Left) => {
                    key = Joystick::Left;
                }
                Ok(Key::Right) => {
                    key = Joystick::Right;
                }
                Ok(_) => {}
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => panic!("Disconnected"),
            }
        }
        key
    }
}

#[derive(Debug)]
struct Engine {
    breakout: Breakout,
    screen: Arc<Mutex<Screen>>,
    score: Arc<Mutex<i32>>,
    delay: Duration,
}

impl Engine {
    fn new(breakout: Breakout, delay: Duration) -> Self {
        let screen = breakout.screen().clone();
        Engine {
            breakout: breakout,
            screen: Arc::new(Mutex::new(screen)),
            score: Arc::new(Mutex::new(0)),
            delay: delay,
        }
    }

    fn layout(&self) -> BoxedView {
        let title = TextView::new("Breakout!").center().fixed_height(1);
        let screen = Panel::new(ScreenView::new(self.screen.clone()));
        let score = ScoreView::new(self.score.clone()).fixed_height(1);

        let layout = LinearLayout::vertical()
            .child(title)
            .child(screen)
            .child(score);

        BoxedView::boxed(layout)
    }

    fn build(program: Program, app: &mut Cursive) -> Self {
        let (etx, erx) = mpsc::channel();
        let engine = Engine::new(
            Breakout::new_with_coins(program, Box::new(CursiveController::new(erx))),
            Duration::from_millis(500),
        );

        let layout = engine.layout();
        let etx_left = etx.clone();
        let etx_right = etx.clone();

        app.add_layer(
            OnEventView::new(layout)
                .on_event(Key::Left, move |_| {
                    let _ = etx_left.send(Key::Left);
                })
                .on_event(Key::Right, move |_| {
                    let _ = etx_right.send(Key::Right);
                }),
        );
        engine
    }

    fn build_ai(program: Program, app: &mut Cursive) -> Self {
        let engine = Engine::new(
            Breakout::new_with_coins(program, Box::new(SimpleController::new())),
            Duration::from_millis(10),
        );
        let layout = engine.layout();
        app.add_layer(layout);
        engine
    }
}

fn worker(mut engine: Engine) -> () {
    loop {
        *engine.screen.lock().unwrap() = engine.breakout.screen().clone();
        *engine.score.lock().unwrap() = engine.breakout.screen().score() as i32;
        match engine.breakout.step().unwrap() {
            State::Step => {
                thread::sleep(engine.delay);
            }
            State::Halt => break,
        }
    }
    *engine.screen.lock().unwrap() = engine.breakout.screen().clone();
    *engine.score.lock().unwrap() = engine.breakout.screen().score() as i32;
}
