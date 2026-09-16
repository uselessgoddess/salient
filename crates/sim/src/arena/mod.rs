//! Dense storage with generational handles.
//!
//! This exists instead of an entity-component system because iteration order has to be a
//! property of the data rather than of a library's internal layout. Query order in a general
//! ECS is explicitly unspecified and shifts as components are added and removed; here the
//! order is the slot index, and it changes only when the simulation changes it.
//!
//! Slot reuse is part of that: the free list is a stack, so an identical sequence of spawns
//! and despawns reuses identical slots. That makes handle values themselves reproducible,
//! which matters because they are part of the state fingerprint.

/// A stable reference into an [`Arena`]. The generation distinguishes a live entity from a
/// dead one that happened to occupy the same slot.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct Handle {
    slot: u32,
    epoch: u32,
}

#[derive(Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct Slot<T> {
    epoch: u32,
    value: Option<T>,
}

#[derive(Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    live: u32,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena { slots: Vec::new(), free: Vec::new(), live: 0 }
    }
}

impl<T> Arena<T> {
    pub fn new() -> Arena<T> {
        Arena::default()
    }

    pub fn len(&self) -> u32 {
        self.live
    }

    pub fn is_empty(&self) -> bool {
        self.live == 0
    }

    pub fn insert(&mut self, value: T) -> Handle {
        match self.free.pop() {
            Some(slot) => {
                let s = &mut self.slots[slot as usize];
                s.epoch += 1;
                s.value = Some(value);
                self.live += 1;
                Handle { slot, epoch: s.epoch }
            }
            None => {
                let slot = self.slots.len() as u32;
                self.slots.push(Slot { epoch: 0, value: Some(value) });
                self.live += 1;
                Handle { slot, epoch: 0 }
            }
        }
    }

    pub fn remove(&mut self, h: Handle) -> Option<T> {
        let s = self.slots.get_mut(h.slot as usize)?;
        if s.epoch != h.epoch {
            return None;
        }
        let taken = s.value.take();
        if taken.is_some() {
            self.live -= 1;
            self.free.push(h.slot);
        }
        taken
    }

    pub fn get(&self, h: Handle) -> Option<&T> {
        let s = self.slots.get(h.slot as usize)?;
        if s.epoch == h.epoch { s.value.as_ref() } else { None }
    }

    pub fn get_mut(&mut self, h: Handle) -> Option<&mut T> {
        let s = self.slots.get_mut(h.slot as usize)?;
        if s.epoch == h.epoch { s.value.as_mut() } else { None }
    }

    pub fn contains(&self, h: Handle) -> bool {
        self.get(h).is_some()
    }

    /// Live entries in slot order. This order is the simulation's order, not an accident.
    pub fn iter(&self) -> impl Iterator<Item = (Handle, &T)> {
        self.slots.iter().enumerate().filter_map(|(i, s)| {
            s.value.as_ref().map(|v| (Handle { slot: i as u32, epoch: s.epoch }, v))
        })
    }

    /// Live entries in slot order, mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Handle, &mut T)> {
        self.slots.iter_mut().enumerate().filter_map(|(i, s)| {
            let epoch = s.epoch;
            s.value.as_mut().map(move |v| (Handle { slot: i as u32, epoch }, v))
        })
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().filter_map(|s| s.value.as_ref())
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.iter_mut().filter_map(|s| s.value.as_mut())
    }
}
