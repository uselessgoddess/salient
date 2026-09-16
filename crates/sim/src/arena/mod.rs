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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct Handle {
    slot: u32,
    epoch: u32,
}

#[derive(Clone, Debug)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct Slot<T> {
    epoch: u32,
    value: Option<T>,
}

#[derive(Clone, Debug)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
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

/// A value attached to a live [`Arena`] entry and stored beside the arena rather than inside
/// it.
///
/// This is where state belonging to one removable subsystem goes, so that it does not have to
/// sit in the record every other subsystem reads. Entries are epoch-checked exactly as the
/// arena is, which is the whole reason this type exists rather than a bare parallel vector: a
/// slot reused by a new entry never inherits the previous occupant's value.
#[derive(Clone, Debug)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Table<T> {
    slots: Vec<Option<Entry<T>>>,
}

#[derive(Clone, Debug)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct Entry<T> {
    epoch: u32,
    value: T,
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Table { slots: Vec::new() }
    }
}

impl<T> Table<T> {
    pub fn new() -> Table<T> {
        Table::default()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|s| s.is_none())
    }

    pub fn insert(&mut self, h: Handle, value: T) {
        let slot = h.slot as usize;
        if slot >= self.slots.len() {
            self.slots.resize_with(slot + 1, || None);
        }
        self.slots[slot] = Some(Entry { epoch: h.epoch, value });
    }

    pub fn remove(&mut self, h: Handle) -> Option<T> {
        let s = self.slots.get_mut(h.slot as usize)?;
        match s {
            Some(e) if e.epoch == h.epoch => s.take().map(|e| e.value),
            _ => None,
        }
    }

    pub fn get(&self, h: Handle) -> Option<&T> {
        match self.slots.get(h.slot as usize)? {
            Some(e) if e.epoch == h.epoch => Some(&e.value),
            _ => None,
        }
    }

    /// Visit every entry in slot order, dropping those the predicate rejects.
    ///
    /// Slot order rather than insertion order, so the sequence is a property of the data and
    /// the same on every machine. Dropping is how an entry outliving its unit is reclaimed:
    /// the caller refuses a handle the arena no longer knows, and the entry goes with it.
    pub fn retain(&mut self, mut keep: impl FnMut(Handle, &mut T) -> bool) {
        for (i, s) in self.slots.iter_mut().enumerate() {
            let Some(e) = s else { continue };
            let h = Handle { slot: i as u32, epoch: e.epoch };
            if !keep(h, &mut e.value) {
                *s = None;
            }
        }
    }
}
