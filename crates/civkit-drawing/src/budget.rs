// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 20:24 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The heap budget (ADR 0015, Q-D): every heap allocation the reader makes reserves its bytes here first, and
//! releases them when freed, so the heap the reader has requested never passes the budget.
//!
//! The budget bounds **reader-requested heap**, not process memory: allocator overhead, the caller's data, stacks and
//! the rest of the program are outside it. Parsing and expansion (Phase 4 task 3) share one budget.

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ReadError;

/// A heap budget in bytes, shared by everything read or derived from one drawing. Cheap to clone (a shared handle).
#[derive(Clone)]
pub struct Budget(Arc<Inner>);

struct Inner {
    limit: usize,
    used: AtomicUsize,
    peak: AtomicUsize,
}

impl Budget {
    /// A budget of `limit` bytes.
    pub fn new(limit: usize) -> Self {
        Self(Arc::new(Inner {
            limit,
            used: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        }))
    }

    /// The limit, in bytes.
    pub fn limit(&self) -> usize {
        self.0.limit
    }

    /// Bytes reserved now.
    pub fn used(&self) -> usize {
        self.0.used.load(Ordering::Relaxed)
    }

    /// The most bytes ever reserved at once.
    pub fn peak(&self) -> usize {
        self.0.peak.load(Ordering::Relaxed)
    }

    fn reserve(&self, bytes: usize) -> Result<(), ReadError> {
        let exhausted = || ReadError::BudgetExhausted {
            requested: bytes,
            limit: self.0.limit,
        };
        let mut current = self.0.used.load(Ordering::Relaxed);
        loop {
            let next = current
                .checked_add(bytes)
                .filter(|n| *n <= self.0.limit)
                .ok_or_else(exhausted)?;
            match self.0.used.compare_exchange_weak(
                current,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.0.peak.fetch_max(next, Ordering::Relaxed);
                    return Ok(());
                }
                Err(seen) => current = seen,
            }
        }
    }

    fn release(&self, bytes: usize) {
        self.0.used.fetch_sub(bytes, Ordering::Relaxed);
    }
}

impl fmt::Debug for Budget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Budget")
            .field("limit", &self.limit())
            .field("used", &self.used())
            .field("peak", &self.peak())
            .finish()
    }
}

/// Bytes reserved from a budget, released when dropped.
pub(crate) struct Charge {
    budget: Budget,
    bytes: usize,
}

impl Charge {
    /// A charge of nothing yet.
    pub(crate) fn empty(budget: &Budget) -> Self {
        Self {
            budget: budget.clone(),
            bytes: 0,
        }
    }

    /// Reserves `bytes` more, or fails without changing anything.
    pub(crate) fn grow(&mut self, bytes: usize) -> Result<(), ReadError> {
        self.budget.reserve(bytes)?;
        self.bytes += bytes;
        Ok(())
    }

    /// Releases `bytes` of this charge (an old buffer that has been freed).
    fn shrink(&mut self, bytes: usize) {
        let bytes = bytes.min(self.bytes);
        self.budget.release(bytes);
        self.bytes -= bytes;
    }
}

impl Drop for Charge {
    fn drop(&mut self) {
        self.budget.release(self.bytes);
    }
}

fn allocation_failed(bytes: usize) -> ReadError {
    ReadError::AllocationFailed { bytes }
}

/// A vector whose capacity is reserved from a budget before it grows. Read it as a slice.
pub struct BVec<T> {
    items: Vec<T>,
    charge: Charge,
}

impl<T> BVec<T> {
    pub(crate) fn new(budget: &Budget) -> Self {
        Self {
            items: Vec::new(),
            charge: Charge::empty(budget),
        }
    }

    /// A vector of `len` copies of `value`, reserved in one step.
    pub(crate) fn filled(budget: &Budget, len: usize, value: T) -> Result<Self, ReadError>
    where
        T: Clone,
    {
        let size = std::mem::size_of::<T>();
        let bytes = len
            .checked_mul(size)
            .ok_or_else(|| allocation_failed(usize::MAX))?;
        let mut charge = Charge::empty(budget);
        charge.grow(bytes)?;
        let mut items = Vec::new();
        items
            .try_reserve_exact(len)
            .map_err(|_| allocation_failed(bytes))?;
        if items.capacity() > len {
            charge.grow((items.capacity() - len) * size)?;
        }
        items.resize(len, value);
        Ok(Self { items, charge })
    }

    /// Appends, growing the capacity explicitly (never by an unreserved implicit doubling).
    pub(crate) fn push(&mut self, value: T) -> Result<(), ReadError> {
        if self.items.len() == self.items.capacity() {
            self.grow()?;
        }
        self.items.push(value);
        Ok(())
    }

    /// Removes and returns the last element; the capacity (and its charge) is kept.
    pub(crate) fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Grows the capacity. While the allocator moves the elements, the old and the new buffer can both exist, so the
    /// new buffer is reserved in full while the old one is still charged, and the old one is released only after the
    /// move (gate X3 measured this transient before it was reserved).
    fn grow(&mut self) -> Result<(), ReadError> {
        let size = std::mem::size_of::<T>();
        let cap = self.items.capacity();
        let want = cap.saturating_mul(2).max(4);
        let new_bytes = want
            .checked_mul(size)
            .ok_or_else(|| allocation_failed(usize::MAX))?;
        self.charge.grow(new_bytes)?;
        self.items
            .try_reserve_exact(want - self.items.len())
            .map_err(|_| allocation_failed(new_bytes))?;
        // The allocator may give more than asked: reserve that too, so the charge covers the whole capacity.
        let got = self.items.capacity();
        if got > want {
            self.charge.grow((got - want) * size)?;
        }
        self.charge.shrink(cap * size);
        Ok(())
    }
}

impl<T> Deref for BVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.items
    }
}

impl<T> DerefMut for BVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.items
    }
}

impl<T: fmt::Debug> fmt::Debug for BVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.items.iter()).finish()
    }
}

/// A string whose bytes are reserved from a budget. Read it as a `str`.
pub struct BStr {
    text: String,
    _charge: Charge,
}

impl BStr {
    /// A string of exactly `len` bytes, written by `fill`, which must push exactly that many.
    pub(crate) fn build(
        budget: &Budget,
        len: usize,
        fill: impl FnOnce(&mut String),
    ) -> Result<Self, ReadError> {
        let mut charge = Charge::empty(budget);
        charge.grow(len)?;
        let mut text = String::new();
        text.try_reserve_exact(len)
            .map_err(|_| allocation_failed(len))?;
        if text.capacity() > len {
            charge.grow(text.capacity() - len)?;
        }
        fill(&mut text);
        debug_assert_eq!(
            text.len(),
            len,
            "BStr::build: fill wrote a different length"
        );
        Ok(Self {
            text,
            _charge: charge,
        })
    }
}

impl Deref for BStr {
    type Target = str;
    fn deref(&self) -> &str {
        &self.text
    }
}

impl fmt::Debug for BStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.text, f)
    }
}

/// A reusable text buffer for decoding a value before it is looked up; its capacity is reserved.
pub(crate) struct Scratch {
    text: String,
    charge: Charge,
}

impl Scratch {
    pub(crate) fn new(budget: &Budget) -> Self {
        Self {
            text: String::new(),
            charge: Charge::empty(budget),
        }
    }

    /// Clears the buffer and makes room for `len` bytes, reserving any growth first.
    pub(crate) fn prepare(&mut self, len: usize) -> Result<&mut String, ReadError> {
        self.text.clear();
        let cap = self.text.capacity();
        if len > cap {
            // As for BVec: the new buffer is reserved in full before the old one is released.
            let want = len.max(cap.saturating_mul(2)).max(64);
            self.charge.grow(want)?;
            self.text
                .try_reserve_exact(want)
                .map_err(|_| allocation_failed(want))?;
            let got = self.text.capacity();
            if got > want {
                self.charge.grow(got - want)?;
            }
            self.charge.shrink(cap);
        }
        Ok(&mut self.text)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }
}

/// An open-addressing hash index of entry numbers (into a list kept elsewhere). Its slots are a [`BVec`], so every
/// growth is reserved first and a refused allocation is a clean error (ADR 0015, Q-D). The hasher is std's randomly
/// keyed one, so a hostile file cannot force collisions; nothing read depends on the hash, so results stay
/// deterministic. Kept at most half full.
pub(crate) struct BIndex {
    slots: BVec<u32>,
    used: usize,
    hasher: std::hash::RandomState,
    budget: Budget,
}

const EMPTY: u32 = u32::MAX;

impl BIndex {
    pub(crate) fn new(budget: &Budget) -> Self {
        Self {
            slots: BVec::new(budget),
            used: 0,
            hasher: std::hash::RandomState::new(),
            budget: budget.clone(),
        }
    }

    /// The hash of a key.
    pub(crate) fn hash<T: std::hash::Hash + ?Sized>(&self, key: &T) -> u64 {
        use std::hash::BuildHasher;
        self.hasher.hash_one(key)
    }

    /// The entry with this hash for which `is` holds, if any.
    pub(crate) fn find(&self, hash: u64, mut is: impl FnMut(u32) -> bool) -> Option<u32> {
        if self.slots.is_empty() {
            return None;
        }
        let mask = self.slots.len() - 1;
        let mut i = (hash as usize) & mask;
        loop {
            let slot = self.slots[i];
            if slot == EMPTY {
                return None;
            }
            if is(slot) {
                return Some(slot);
            }
            i = (i + 1) & mask;
        }
    }

    /// Adds entry `id` (not yet present) with `hash`. When the table grows, `rehash(hasher, id)` gives each existing
    /// entry's hash.
    pub(crate) fn insert(
        &mut self,
        hash: u64,
        id: u32,
        rehash: impl Fn(&std::hash::RandomState, u32) -> u64,
    ) -> Result<(), ReadError> {
        if (self.used + 1) * 2 > self.slots.len() {
            let len = self.slots.len().saturating_mul(2).max(16);
            // The new table is reserved in full while the old one is still charged; the old is released when dropped.
            let mut fresh = BVec::filled(&self.budget, len, EMPTY)?;
            for &old in self.slots.iter() {
                if old != EMPTY {
                    place(&mut fresh, rehash(&self.hasher, old), old);
                }
            }
            self.slots = fresh;
        }
        place(&mut self.slots, hash, id);
        self.used += 1;
        Ok(())
    }
}

fn place(slots: &mut [u32], hash: u64, id: u32) {
    let mask = slots.len() - 1;
    let mut i = (hash as usize) & mask;
    while slots[i] != EMPTY {
        i = (i + 1) & mask;
    }
    slots[i] = id;
}
