//! Order validation.
//!
//! Refusal belongs here, at the crate boundary, and nowhere deeper. The tick loop is total by
//! contract: anything that reaches it is already well formed, so it has no rejection paths to
//! get wrong and no way to panic on malformed input.

use super::order::{Action, Order, Reject, Target};
use crate::State;

pub fn validate(state: &State, order: &Order) -> Result<(), Reject> {
    match &order.target {
        Target::Units(hs) => {
            for &h in hs {
                let u = state.units.get(h).ok_or(Reject::StaleHandle)?;
                if u.owner != order.issuer {
                    return Err(Reject::NotOwned);
                }
            }
        }
        Target::Group(_) => {}
    }

    match &order.action {
        Action::Move { to } => {
            if !state.in_bounds(*to) {
                return Err(Reject::OffMap);
            }
        }
        Action::Stop => {}
    }

    Ok(())
}
