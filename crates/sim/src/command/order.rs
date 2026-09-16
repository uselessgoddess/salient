//! Orders: the only thing that enters the simulation.
//!
//! Everything a player, an opponent, a script, or a remote peer can ever do is expressible
//! here, which is what makes a recording a complete description of a match. An action that
//! cannot be written as `(tick, issuer, target, action)` would be a side channel around the
//! simulation boundary, and is a signal to reconsider the design rather than to widen this
//! type.

use core::fmt;

use crate::{Handle, Player, Vec2};

/// A rule-defined group. Membership is derived each tick, never stored as a list, so units
/// produced later join the group they belong to without anyone maintaining it.
pub type GroupId = u16;

#[derive(Clone, Debug, PartialEq)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum Target {
    Units(Vec<Handle>),
    Group(GroupId),
}

/// Actions grow over the increments. Adding one must not change the shape of [`Order`].
#[derive(Clone, Debug, PartialEq)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum Action {
    Move { to: Vec2 },
    Stop,
}

#[derive(Clone, Debug, PartialEq)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Order {
    /// The tick this takes effect on.
    pub tick: u32,
    pub issuer: Player,
    pub target: Target,
    pub action: Action,
}

/// Why an order was refused. Refusal happens at the boundary, before recording, so a rejected
/// order never reaches the tick loop and never appears in a replay.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reject {
    /// The handle refers to a slot that is empty or has been reused.
    StaleHandle,
    /// The issuer does not own the target.
    NotOwned,
    /// The destination is outside the world.
    OffMap,
    /// The target's class cannot perform this action.
    Unsupported,
}

impl fmt::Display for Reject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Reject::StaleHandle => "handle is stale",
            Reject::NotOwned => "target not owned by issuer",
            Reject::OffMap => "destination is off the map",
            Reject::Unsupported => "action unsupported by target",
        };
        f.write_str(s)
    }
}
