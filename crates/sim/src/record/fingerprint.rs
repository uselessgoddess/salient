//! The state fingerprint.
//!
//! One mechanism serves four purposes, which is the difference between building these and
//! getting them: desync detection, replays, saves, and state diffing all read the same bytes.
//!
//! The bytes are `rkyv` with little-endian layout forced, so they are identical on every
//! platform. That is what makes a byte hash a cross-platform fingerprint with no extra work —
//! hashing the in-memory representation instead would depend on padding and field order.

use core::hash::Hasher;

use crate::state::State;

/// Hash of the serialized state.
///
/// Panics only if state fails to serialize, which cannot happen for a plain data structure.
/// This is outside the tick loop, so it is not covered by the tick loop's no-panic contract.
pub fn of(state: &State) -> u64 {
    let bytes = super::snapshot::to_bytes(state);
    let mut h = rustc_hash::FxHasher::default();
    h.write(&bytes);
    h.finish()
}
