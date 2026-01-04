//! Transposition table and caching
//!
//! TODO: Port from C++ solver

use super::cards::Cards;
use super::hands::Hands;
use super::types::*;

/// Bounds for cached positions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Bounds {
    pub lower: i8,
    pub upper: i8,
}

impl Bounds {
    pub fn new(lower: i8, upper: i8) -> Self {
        Bounds { lower, upper }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.upper < self.lower
    }

    #[inline]
    pub fn intersect(&self, other: Bounds) -> Bounds {
        Bounds {
            lower: self.lower.max(other.lower),
            upper: self.upper.min(other.upper),
        }
    }

    #[inline]
    pub fn cutoff(&self, beta: i8) -> bool {
        self.lower >= beta || self.upper < beta
    }
}

/// Shape encoding - 4 bits per suit per seat = 64 bits total
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Shape {
    value: u64,
}

impl Shape {
    pub fn new() -> Self {
        Shape { value: 0 }
    }

    pub fn from_hands(hands: &Hands) -> Self {
        let mut value = 0u64;
        for seat in 0..NUM_SEATS {
            for suit in 0..NUM_SUITS {
                let len = hands[seat].suit(suit).size() as u64;
                value += len << Self::offset(seat, suit);
            }
        }
        Shape { value }
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    #[inline]
    fn offset(seat: Seat, suit: Suit) -> u32 {
        (60 - (seat * NUM_SUITS + suit) * 4) as u32
    }

    pub fn suit_length(&self, seat: Seat, suit: Suit) -> usize {
        ((self.value >> Self::offset(seat, suit)) & 0xF) as usize
    }
}

/// Simple transposition table entry
#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub hash: u64,
    pub bounds: Bounds,
}

impl TTEntry {
    pub fn new() -> Self {
        TTEntry {
            hash: 0,
            bounds: Bounds::new(0, TOTAL_TRICKS as i8),
        }
    }
}

impl Default for TTEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Cut-off cache entry
#[derive(Clone, Copy, Debug, Default)]
pub struct CutoffEntry {
    pub hash: u64,
    pub cards: [u8; NUM_SEATS], // Best cutoff card per seat
}

impl CutoffEntry {
    pub fn new() -> Self {
        CutoffEntry {
            hash: 0,
            cards: [TOTAL_CARDS as u8; NUM_SEATS],
        }
    }
}

/// Simple hash-based cache - preallocated, no dynamic allocation
pub struct Cache<T: Default + Copy> {
    entries: Vec<T>,  // TODO: Use Box<[T]> for fixed allocation
    mask: usize,
}

impl<T: Default + Copy> Cache<T> {
    pub fn new(bits: usize) -> Self {
        let size = 1 << bits;
        let mut entries = Vec::with_capacity(size);
        entries.resize_with(size, T::default);
        Cache {
            entries,
            mask: size - 1,
        }
    }

    pub fn reset(&mut self) {
        for entry in &mut self.entries {
            *entry = T::default();
        }
    }

    #[inline]
    pub fn get(&self, hash: u64) -> &T {
        &self.entries[(hash as usize) & self.mask]
    }

    #[inline]
    pub fn get_mut(&mut self, hash: u64) -> &mut T {
        &mut self.entries[(hash as usize) & self.mask]
    }
}

/// Hash function for position lookup
#[inline]
pub fn hash_position(cards: &[Cards; 2]) -> u64 {
    const HASH_RAND: [u64; 2] = [0x9b8b4567327b23c7, 0x643c986966334873];
    (cards[0].value().wrapping_add(HASH_RAND[0]))
        .wrapping_mul(cards[1].value().wrapping_add(HASH_RAND[1]))
}
