//! Mobile entities.

use crate::{Fx, Handle, Vec2};

/// The medium a unit occupies. Exactly one, and it never changes.
///
/// The display separates these by draw order rather than by a mode switch: subsurface glyphs
/// are drawn beneath the water hatch so the hatch crosses them, air glyphs above everything
/// with an offset and a ground shadow. Shape encodes the same thing redundantly, so the
/// reading survives wherever layering is ambiguous.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
#[repr(u8)]
pub enum Domain {
    Land = 0,
    Surface = 1,
    Subsurface = 2,
    Air = 3,
}

impl Domain {
    pub const ALL: [Domain; 4] = [Domain::Land, Domain::Surface, Domain::Subsurface, Domain::Air];
}

/// Which side a thing belongs to. Index into a small fixed set; the hostile force holds its
/// own index and neutral ownership is a reserved value.
pub type Player = u8;

pub const NEUTRAL: Player = u8::MAX;

#[derive(Clone, Debug)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Unit {
    pub pos: Vec2,
    pub vel: Vec2,
    pub facing: Fx,
    pub domain: Domain,
    pub owner: Player,
    pub health: Fx,
}

impl Unit {
    pub fn new(pos: Vec2, domain: Domain, owner: Player) -> Unit {
        Unit { pos, vel: Vec2::ZERO, facing: Fx::ZERO, domain, owner, health: Fx::from_int(100) }
    }
}

/// A unit reference as it appears in orders.
pub type UnitId = Handle;
