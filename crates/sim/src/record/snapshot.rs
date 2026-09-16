//! Serialized state.
//!
//! No file access here, or anywhere in this crate: the simulation performs no I/O. These
//! produce and consume bytes, and whoever wants them on disk owns that decision.

use rkyv::rancor;
use rkyv::util::AlignedVec;

use crate::state::State;

pub fn to_bytes(state: &State) -> Vec<u8> {
    rkyv::to_bytes::<rancor::Error>(state)
        .expect("state is plain data and always serializes")
        .to_vec()
}

/// Copy into an aligned buffer before reading.
///
/// A `Vec<u8>` is only byte-aligned, and the archived layout needs more than that. Whether a
/// given allocation happens to satisfy it is luck, and luck that holds in testing and fails in
/// the field is the worst kind.
pub fn from_bytes(bytes: &[u8]) -> Result<State, rancor::Error> {
    let mut aligned = AlignedVec::<16>::with_capacity(bytes.len());
    aligned.extend_from_slice(bytes);
    rkyv::from_bytes::<State, rancor::Error>(&aligned)
}
