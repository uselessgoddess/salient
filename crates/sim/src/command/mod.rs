//! The order boundary: validation on the way in, application at the start of a tick.

pub mod order;
mod validate;

pub use order::{Action, GroupId, Order, Reject, Target};
pub use validate::validate;

use crate::state::State;

/// Apply the orders due this tick.
///
/// Conflicting orders against the same unit are resolved by list position: the later one
/// wins. The list is what a recording stores, so replay resolves the conflict identically
/// without any tie-breaking rule that someone has to remember.
pub fn apply(state: &mut State, orders: &[Order]) {
    for o in orders {
        let targets: Vec<_> = match &o.target {
            Target::Units(hs) => hs.clone(),
            // Group membership is derived, and no group rules exist yet. Later increments
            // resolve this against the rule rather than against a stored list.
            Target::Group(_) => Vec::new(),
        };
        for h in targets {
            let Some(u) = state.units.get_mut(h) else { continue };
            if u.owner != o.issuer {
                continue;
            }
            match &o.action {
                Action::Move { to } => u.goal = Some(*to),
                Action::Stop => {
                    u.goal = None;
                    u.vel = crate::fx::Vec2::ZERO;
                }
            }
        }
    }
}
