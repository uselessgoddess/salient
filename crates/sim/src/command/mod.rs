//! The order boundary: validation on the way in, application at the start of a tick.

pub mod order;
mod validate;

pub use order::{Action, GroupId, Order, Reject, Target};
pub use validate::validate;

use crate::{Handle, State, Vec2};

/// Apply the orders due this tick.
///
/// Conflicting orders against the same unit are resolved by list position: the later one
/// wins. The list is what a recording stores, so replay resolves the conflict identically
/// without any tie-breaking rule that someone has to remember.
pub fn apply(state: &mut State, orders: &[Order]) {
    let State { units, goals, .. } = state;
    for o in orders {
        let targets: &[Handle] = match &o.target {
            Target::Units(hs) => hs,
            Target::Group(_) => &[],
        };
        for &h in targets {
            let Some(u) = units.get_mut(h) else { continue };
            if u.owner != o.issuer {
                continue;
            }
            match &o.action {
                Action::Move { to } => goals.insert(h, *to),
                Action::Stop => {
                    goals.remove(h);
                    u.vel = Vec2::ZERO;
                }
            }
        }
    }
}
