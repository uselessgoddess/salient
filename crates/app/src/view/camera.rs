//! The camera.
//!
//! Zoom is the thing that makes a battle at this scale legible, and it is nearly free in two
//! dimensions. The level-of-detail thresholds it drives — body, glyph, aggregate — arrive with
//! the display grammar in the next increment.

use bevy::prelude::*;

/// World units across the map. Matches the simulation's extent.
const WORLD: f32 = 40_000.0;

const MIN_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 60.0;
const PAN_SPEED: f32 = 900.0;

#[derive(Component)]
pub struct Eye;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn).add_systems(Update, (zoom, pan));
}

fn spawn(mut commands: Commands) {
    // Start with the whole world in view: this scenario is about seeing the shape of a force,
    // not about reading one unit.
    let fit = WORLD / 1_280.0;
    commands.spawn((
        Eye,
        Camera2d,
        Transform::from_xyz(WORLD / 2.0, WORLD / 2.0, 1_000.0).with_scale(Vec3::splat(fit)),
    ));
}

fn zoom(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut eye: Query<&mut Transform, With<Eye>>,
) {
    let scroll: f32 = wheel.read().map(|w| w.y).sum();
    if scroll == 0.0 {
        return;
    }
    for mut t in &mut eye {
        let next = (t.scale.x * (1.0 - scroll * 0.1)).clamp(MIN_ZOOM, MAX_ZOOM);
        t.scale = Vec3::splat(next);
    }
}

fn pan(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut eye: Query<&mut Transform, With<Eye>>,
) {
    let mut d = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        d.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        d.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        d.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        d.x += 1.0;
    }
    if d == Vec2::ZERO {
        return;
    }
    for mut t in &mut eye {
        let step = d.normalize() * PAN_SPEED * t.scale.x * time.delta_secs();
        t.translation.x += step.x;
        t.translation.y += step.y;
    }
}
