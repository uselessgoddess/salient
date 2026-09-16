# Contract: Recordings and Snapshots

One mechanism serves four purposes — desync detection, replays, saves, and state diffing. They are
not separate features (spec, Assumptions).

## Recording

```text
Header { magic, format_version: u16, sim_version: u32, seed: u64, rules: MatchRules }
Orders [ Order ]                        // ordered by tick, then by issue order within a tick
Checkpoints [ (tick: u32, fingerprint: u64) ]
```

**Rules**

- `sim_version` is compared on load. A mismatch is **refused with an explanation**, never played
  back (FR-003). Silent playback against changed behaviour produces a plausible wrong result, which
  is worse than a failure.
- `rules` is restored, not reselected. A match that ended by commander loss replays to the same
  ending (FR-041).
- Checkpoints exist to bisect divergence. Replaying and comparing at each checkpoint locates the
  first differing tick without instrumenting the simulation (FR-004, SC-010).
- Orders within a tick keep their recorded order. This is what makes the same-tick conflict rule
  reproducible.

## Snapshot

`rkyv` serialization of `SimState` with the `little_endian` feature, so the bytes are identical on
every platform. The fingerprint is a hash over exactly these bytes, which is why the fingerprint is
cross-platform without any extra work.

A snapshot plus the remaining orders is a resumable save (FR-044). A recording alone is also a
resumable save, replayed forward. Which to use is a size-versus-time trade-off, not a design
decision.

## Versioning obligation

Any change to simulation behaviour MUST bump `SIM_VERSION`, regenerate the golden recording, and
record the reason in the commit (Constitution: determinism gate). `format_version` is separate and
changes only when the file layout changes, so an old recording can be rejected for the right reason.
