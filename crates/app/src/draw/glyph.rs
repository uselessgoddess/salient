//! Builds the glyph mesh.
//!
//! One mesh per domain rather than one entity per unit. Twenty thousand entities would cost
//! more in change detection and transform propagation than the drawing itself, and it would
//! need textures, which there are none of and never will be.
//!
//! Interpolation between ticks happens here, on the processor. Moving it into a vertex shader
//! would let the mesh be rebuilt at ten hertz instead of at frame rate, and that is the plan
//! for when unit counts make it matter. It does not matter yet, and building the optimisation
//! before the measurement asks for it is how projects acquire machinery they cannot debug.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;

use super::shape;
use crate::driver::Driver;

/// Marks the mesh entity for one domain.
#[derive(Component)]
pub struct Layer(pub u8);

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Draw order carries meaning, so the layers are spawned in it: subsurface lowest, air
    // highest. The water hatch will slot between subsurface and the rest.
    for (order, domain) in [2u8, 0, 1, 3].into_iter().enumerate() {
        let c = shape::tint(domain);
        commands.spawn((
            Layer(domain),
            Mesh2d(meshes.add(empty())),
            MeshMaterial2d(materials.add(Color::srgba(c[0], c[1], c[2], c[3]))),
            Transform::from_xyz(0.0, 0.0, order as f32),
        ));
    }
}

fn empty() -> Mesh {
    // Both worlds: the mesh is rewritten every frame, so the processor-side copy has to
    // survive extraction. RENDER_WORLD alone discards it and the next rebuild panics.
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new())
}

pub fn rebuild(
    driver: Res<Driver>,
    mut meshes: ResMut<Assets<Mesh>>,
    layers: Query<(&Layer, &Mesh2d)>,
) {
    let alpha = driver.alpha;
    let pairs = driver.prev.iter().zip(driver.next.iter());

    for (layer, handle) in &layers {
        let Some(mut mesh) = meshes.get_mut(&handle.0) else { continue };
        let outline = shape::of(layer.0);

        let mut pos: Vec<[f32; 3]> = Vec::new();
        for (a, b) in pairs.clone() {
            if b.domain != layer.0 {
                continue;
            }
            let centre = a.pos.lerp(b.pos, alpha);
            // Heading is in turns and wraps, so interpolate along the shorter arc rather than
            // sweeping the long way round whenever a unit crosses zero.
            let turn = b.facing - a.facing;
            let turn = turn - (turn + 0.5).floor();
            let angle = (a.facing + turn * alpha) * std::f32::consts::TAU;
            let (s, c) = angle.sin_cos();

            for v in outline {
                let (x, y) = (v[0] * shape::RADIUS, v[1] * shape::RADIUS);
                pos.push([centre.x + x * c - y * s, centre.y + x * s + y * c, 0.0]);
            }
        }

        let n = pos.len();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; n]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; n]);
    }
}
