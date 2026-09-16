//! Glyph geometry.
//!
//! Shapes are the first half of the visual grammar: domain by outline, so a unit reads
//! correctly even where draw order is ambiguous. The full grammar — tier markings, roles,
//! and the layering that puts subsurface glyphs beneath the water hatch — arrives with the
//! map in the next increment. These are the primitives it is built from.
//!
//! Every shape is a triangle list in glyph-local space, in units where 1.0 is the glyph's
//! nominal radius, pointing along `+x`.

/// Nominal glyph radius in world units. Chosen so a glyph is roughly ten pixels across when
/// the whole forty-kilometre map is on screen.
pub const RADIUS: f32 = 130.0;

/// Land: a blunt rectangle. Ground formations are drawn as boxes on every staff map there is.
const LAND: &[[f32; 2]] =
    &[[-0.9, -0.6], [0.9, -0.6], [0.9, 0.6], [-0.9, -0.6], [0.9, 0.6], [-0.9, 0.6]];

/// Surface: a wedge, pointed along the heading.
const SURFACE: &[[f32; 2]] = &[[1.2, 0.0], [-0.8, 0.7], [-0.8, -0.7]];

/// Subsurface: the same wedge with a bar beneath it, so the two read apart even when one is
/// directly under the other.
const SUBSURFACE: &[[f32; 2]] = &[
    [1.0, 0.15],
    [-0.7, 0.75],
    [-0.7, -0.15],
    [-0.9, -0.55],
    [1.0, -0.55],
    [1.0, -0.85],
    [-0.9, -0.55],
    [1.0, -0.85],
    [-0.9, -0.85],
];

/// Air: a chevron. The offset and ground shadow that carry altitude belong to the layering
/// pass, not to the shape.
const AIR: &[[f32; 2]] =
    &[[1.1, 0.0], [-0.5, 0.8], [-0.1, 0.0], [1.1, 0.0], [-0.1, 0.0], [-0.5, -0.8]];

pub fn of(domain: u8) -> &'static [[f32; 2]] {
    match domain {
        0 => LAND,
        1 => SURFACE,
        2 => SUBSURFACE,
        _ => AIR,
    }
}

/// Pale, high-key colours in the register of a printed staff map rather than of a game.
/// The palette proper arrives with the terrain it has to sit against.
pub fn tint(domain: u8) -> [f32; 4] {
    match domain {
        0 => [0.25, 0.28, 0.22, 1.0],
        1 => [0.13, 0.35, 0.52, 1.0],
        2 => [0.30, 0.46, 0.55, 0.85],
        _ => [0.45, 0.22, 0.20, 1.0],
    }
}
