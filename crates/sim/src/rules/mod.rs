//! Match rules.
//!
//! The defeat condition is a setting rather than a fixture, so that the commander can exist as
//! a unit in every match without its death necessarily ending one. That split is what keeps
//! every rule set opening the same way, which in turn means there is only one early game to
//! balance rather than one per rule.

use crate::fx::Fx;

/// A condition that ends a match. Each is a predicate over state, evaluated once per tick;
/// adding one must touch nothing but its own predicate and this enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
#[repr(u8)]
pub enum Defeat {
    /// The commander dies.
    Commander = 0,
    /// Every unit is lost.
    Annihilation = 1,
    /// Nothing that can build remains.
    BuildCapability = 2,
}

#[derive(Clone, Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Rules {
    pub defeat: Defeat,
    /// How fast a local stock drains relative to how fast the network refills it.
    ///
    /// The single dial the whole economy turns on. At its permissive extreme the economy is
    /// one shared pool with no positional effect; at its restrictive extreme a disconnected
    /// builder halts within seconds. Somewhere between is the game.
    pub supply_gradient: Fx,
    pub unit_cap: u32,
}

impl Default for Rules {
    fn default() -> Self {
        Rules { defeat: Defeat::BuildCapability, supply_gradient: Fx::HALF, unit_cap: 20_000 }
    }
}
