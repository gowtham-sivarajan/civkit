// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

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

    /// Appends, growing the capacity explicitly (never by an unreserved implicit doubling).
    pub(crate) fn push(&mut self, value: T) -> Result<(), ReadError> {
        if self.items.len() == self.items.capacity() {
            self.grow()?;
        }
        self.items.push(value);
        Ok(())
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

// Ordered and looked up as its text, so a map keyed by `BStr` can be searched with a `&str`.
impl PartialEq for BStr {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for BStr {}

impl PartialOrd for BStr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BStr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.text.cmp(&other.text)
    }
}

impl std::borrow::Borrow<str> for BStr {
    fn borrow(&self) -> &str {
        &self.text
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

/// A sorted map whose entries are reserved from a budget at a stated cost each (key heap bytes are reserved by the
/// key's own storage). The cost covers the standard B-tree's nodes at their minimum fill; the first node is reserved
/// when the map gets its first entry. Gate X3 measures that this covers what is actually allocated.
pub(crate) struct BMap<K, V> {
    map: std::collections::BTreeMap<K, V>,
    charge: Charge,
}

/// Reserved per entry: a leaf holds at least 5 of its 11 slots after a split, plus a share of internal nodes.
const MAP_ENTRY_COST: usize = 96;
/// Reserved with the first entry: the first leaf is allocated whole.
const MAP_FIRST_NODE: usize = 512;

impl<K: Ord, V> BMap<K, V> {
    pub(crate) fn new(budget: &Budget) -> Self {
        Self {
            map: std::collections::BTreeMap::new(),
            charge: Charge::empty(budget),
        }
    }

    pub(crate) fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.get(key)
    }

    pub(crate) fn get_mut<Q: Ord + ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.get_mut(key)
    }

    /// Inserts a key that is not yet present, reserving its entry first.
    pub(crate) fn insert_new(&mut self, key: K, value: V) -> Result<(), ReadError> {
        let cost = MAP_ENTRY_COST
            + if self.map.is_empty() {
                MAP_FIRST_NODE
            } else {
                0
            };
        self.charge.grow(cost)?;
        self.map.insert(key, value);
        Ok(())
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for BMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.map.iter()).finish()
    }
}
