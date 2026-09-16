//! Salient: the shell around the simulation.
//!
//! This binary renders and takes input. It holds no game logic and never writes to simulation
//! state except by issuing orders, which is the boundary the whole project rests on.

mod args;
mod draw;
mod driver;
mod view;

use bevy::prelude::*;

use crate::args::Args;
use crate::driver::Driver;

/// Pale ground, in the register of a printed staff map. The terrain that sits on it arrives
/// with the next increment.
const GROUND: Color = Color::srgb(0.937, 0.925, 0.894);

fn main() {
    let args = Args::parse();
    if let Some(path) = &args.replay {
        eprintln!(
            "salient: replay of {path} is not available until recordings are written to disk"
        );
        std::process::exit(2);
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "salient".into(), ..default() }),
            ..default()
        }))
        .insert_resource(ClearColor(GROUND))
        .insert_resource(Driver::new(&args))
        .insert_resource(args)
        .add_plugins((driver::plugin, view::plugin, draw::plugin))
        .add_systems(Startup, driver::announce)
        .run();
}
