//! Everything the player sees, generated rather than loaded.

pub mod glyph;
pub mod shape;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, glyph::spawn).add_systems(Update, glyph::rebuild);
}
