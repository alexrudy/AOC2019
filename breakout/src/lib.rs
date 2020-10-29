use anyhow::Error;
use cursive::view::Boxable;
use cursive::view::SizeConstraint;
use cursive::views::{LinearLayout, Panel, ResizedView, TextView};
use cursive::{ncurses, Cursive};

mod game;
mod view;

pub use game::{Breakout, Screen, Tile};
use view::ScreenView;

pub fn start(mut game: Breakout) -> Result<(), Error> {
    let screen = game.next()?.clone();


    let mut app = ncurses()?;
    app.set_fps(30);
    app.add_global_callback('q', Cursive::quit);

    let game_layout = ResizedView::new(
        SizeConstraint::Free,
        SizeConstraint::Free,
        LinearLayout::vertical()
            .child(TextView::new("Breakout!").center().fixed_height(1))
            .child(Panel::new(ScreenView::new(screen)))
            .child(TextView::new("Score!").center().fixed_height(1)),
    );

    app.add_layer(game_layout);
    app.run();
    Ok(())
}
