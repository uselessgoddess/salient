//! A recording: everything needed to replay a match, and nothing else.
//!
//! Seed, rules, scenario and an ordered list of orders. That is the entire description of a
//! match, which is a consequence of orders being the simulation's only input rather than a
//! feature anybody had to build.

use rkyv::rancor;
use rkyv::util::AlignedVec;

use crate::command::Order;
use crate::rules::Rules;
use crate::scenario::Scenario;

pub const MAGIC: [u8; 4] = *b"SLNT";

/// Bumped when the file layout changes, which is separate from simulation behaviour changing.
/// Keeping them apart is what lets an old recording be refused for the right reason.
pub const FORMAT_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Recording {
    pub sim_version: u32,
    pub seed: u64,
    pub rules: Rules,
    pub scenario: Scenario,
    pub units: u32,
    /// Ordered by tick, and within a tick by issue order. That order is what makes same-tick
    /// conflicts resolve identically on replay.
    pub orders: Vec<Order>,
    /// `(tick, fingerprint)`. Replaying and comparing at each of these locates the first
    /// diverging tick without instrumenting the simulation.
    pub checkpoints: Vec<(u32, u64)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    NotARecording,
    /// The file layout changed.
    Format {
        found: u16,
        expected: u16,
    },
    /// Simulation behaviour changed. Playing this back would produce a plausible wrong
    /// result, which is worse than a refusal.
    SimVersion {
        found: u32,
        expected: u32,
    },
    Corrupt,
}

impl core::fmt::Display for LoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LoadError::NotARecording => f.write_str("not a Salient recording"),
            LoadError::Format { found, expected } => {
                write!(
                    f,
                    "recording format {found} cannot be read by this build (expects {expected})"
                )
            }
            LoadError::SimVersion { found, expected } => write!(
                f,
                "recording was made by simulation version {found}, this build is {expected}; \
                 replaying it would produce a plausible wrong result, so it is refused"
            ),
            LoadError::Corrupt => f.write_str("recording is corrupt"),
        }
    }
}

impl core::error::Error for LoadError {}

impl Recording {
    pub fn to_bytes(&self) -> Vec<u8> {
        let body = rkyv::to_bytes::<rancor::Error>(self).expect("recording is plain data");
        let mut out = Vec::with_capacity(body.len() + 6);
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&body);
        out
    }

    /// Refuses rather than guessing. A recording that cannot be replayed faithfully is not
    /// replayed at all.
    pub fn from_bytes(bytes: &[u8]) -> Result<Recording, LoadError> {
        if bytes.len() < 6 || bytes[..4] != MAGIC {
            return Err(LoadError::NotARecording);
        }
        let found = u16::from_le_bytes([bytes[4], bytes[5]]);
        if found != FORMAT_VERSION {
            return Err(LoadError::Format { found, expected: FORMAT_VERSION });
        }
        // Copy into an aligned buffer: the header pushes the body off any alignment the
        // allocation happened to have, and the archived layout needs more than byte alignment.
        let mut aligned = AlignedVec::<16>::with_capacity(bytes.len() - 6);
        aligned.extend_from_slice(&bytes[6..]);
        let rec: Recording = rkyv::from_bytes::<Recording, rancor::Error>(&aligned)
            .map_err(|_| LoadError::Corrupt)?;
        if rec.sim_version != crate::SIM_VERSION {
            return Err(LoadError::SimVersion {
                found: rec.sim_version,
                expected: crate::SIM_VERSION,
            });
        }
        Ok(rec)
    }

    /// Orders due on a given tick.
    pub fn due(&self, tick: u32) -> &[Order] {
        let start = self.orders.partition_point(|o| o.tick < tick);
        let end = self.orders.partition_point(|o| o.tick <= tick);
        &self.orders[start..end]
    }
}
