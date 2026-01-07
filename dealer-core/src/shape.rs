//! Shape mask for O(1) shape matching
//!
//! Bridge hands have 560 possible ordered shapes (suit lengths S-H-D-C summing to 13).
//! This module provides a 560-bit mask that can be precomputed at parse time
//! and checked in O(1) at evaluation time.

/// Precomputed offsets for O(1) shape index calculation.
/// SPADE_OFFSETS[s] = number of shapes with fewer than s spades.
const SPADE_OFFSETS: [usize; 15] = [
    0, 105, 196, 274, 340, 395, 440, 476, 504, 525, 540, 550, 556, 559, 560,
];

/// Convert suit lengths to a shape index (0-559).
///
/// The index is computed using a closed-form formula based on
/// the lexicographic ordering of (spades, hearts, diamonds, clubs).
#[inline]
pub fn shape_to_index(s: usize, h: usize, d: usize, _c: usize) -> usize {
    debug_assert!(s + h + d <= 13, "Invalid shape: s={}, h={}, d={}", s, h, d);
    let remaining = 13 - s;
    // Note: h * (remaining + 1) - h * (h - 1) / 2 can overflow when h=0
    // Rewrite as: h * (remaining + 1 - (h - 1) / 2) but that's wrong too
    // Safe formula: h * (2 * remaining + 3 - h) / 2
    // Or just use saturating_sub for the h-1 term
    let hearts_offset = if h == 0 {
        0
    } else {
        h * (remaining + 1) - h * (h - 1) / 2
    };
    SPADE_OFFSETS[s] + hearts_offset + d
}

/// A 560-bit mask representing a set of hand shapes.
///
/// Each bit corresponds to one of the 560 possible ordered shapes.
/// This allows O(1) shape matching after a one-time precomputation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShapeMask {
    /// 9 x 64 = 576 bits (560 used)
    bits: [u64; 9],
}

impl ShapeMask {
    /// Create an empty mask (no shapes match).
    #[inline]
    pub const fn empty() -> Self {
        ShapeMask { bits: [0; 9] }
    }

    /// Create a full mask (all shapes match).
    pub fn all() -> Self {
        let mut mask = ShapeMask {
            bits: [u64::MAX; 9],
        };
        // Clear unused bits (560-576)
        mask.bits[8] &= (1u64 << (560 - 512)) - 1;
        mask
    }

    /// Set the bit for a specific shape index.
    #[inline]
    pub fn set(&mut self, index: usize) {
        debug_assert!(index < 560, "Shape index out of range: {}", index);
        let word = index / 64;
        let bit = index % 64;
        self.bits[word] |= 1u64 << bit;
    }

    /// Clear the bit for a specific shape index.
    #[inline]
    pub fn clear(&mut self, index: usize) {
        debug_assert!(index < 560, "Shape index out of range: {}", index);
        let word = index / 64;
        let bit = index % 64;
        self.bits[word] &= !(1u64 << bit);
    }

    /// Check if a specific shape index is set.
    #[inline]
    pub fn contains(&self, index: usize) -> bool {
        debug_assert!(index < 560, "Shape index out of range: {}", index);
        let word = index / 64;
        let bit = index % 64;
        (self.bits[word] & (1u64 << bit)) != 0
    }

    /// Check if a hand with given suit lengths matches this mask.
    #[inline]
    pub fn matches(&self, s: usize, h: usize, d: usize, c: usize) -> bool {
        self.contains(shape_to_index(s, h, d, c))
    }

    /// Union of two masks (OR).
    #[inline]
    pub fn union(&self, other: &ShapeMask) -> ShapeMask {
        let mut result = ShapeMask::empty();
        for i in 0..9 {
            result.bits[i] = self.bits[i] | other.bits[i];
        }
        result
    }

    /// Intersection of two masks (AND).
    #[inline]
    pub fn intersection(&self, other: &ShapeMask) -> ShapeMask {
        let mut result = ShapeMask::empty();
        for i in 0..9 {
            result.bits[i] = self.bits[i] & other.bits[i];
        }
        result
    }

    /// Difference of two masks (self AND NOT other).
    #[inline]
    pub fn difference(&self, other: &ShapeMask) -> ShapeMask {
        let mut result = ShapeMask::empty();
        for i in 0..9 {
            result.bits[i] = self.bits[i] & !other.bits[i];
        }
        result
    }

    /// Complement of this mask (NOT).
    pub fn complement(&self) -> ShapeMask {
        let mut result = ShapeMask::empty();
        for i in 0..9 {
            result.bits[i] = !self.bits[i];
        }
        // Clear unused bits
        result.bits[8] &= (1u64 << (560 - 512)) - 1;
        result
    }

    /// Check if the mask is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|&b| b == 0)
    }

    /// Count the number of shapes in this mask.
    pub fn count(&self) -> usize {
        self.bits.iter().map(|b| b.count_ones() as usize).sum()
    }

    /// Create a mask for a single exact shape (e.g., 5-4-3-1 in S-H-D-C).
    pub fn exact(s: u8, h: u8, d: u8, c: u8) -> Self {
        let mut mask = ShapeMask::empty();
        if s + h + d + c == 13 {
            mask.set(shape_to_index(
                s as usize, h as usize, d as usize, c as usize,
            ));
        }
        mask
    }

    /// Create a mask for a wildcard shape (None = any length for that suit).
    /// E.g., (Some(5), Some(4), None, None) matches 5-4-x-x.
    pub fn wildcard(pattern: [Option<u8>; 4]) -> Self {
        let mut mask = ShapeMask::empty();

        // Iterate through all 560 shapes and check which match
        for s in 0..14usize {
            if let Some(req) = pattern[0] {
                if s != req as usize {
                    continue;
                }
            }
            for h in 0..(14 - s) {
                if let Some(req) = pattern[1] {
                    if h != req as usize {
                        continue;
                    }
                }
                for d in 0..(14 - s - h) {
                    if let Some(req) = pattern[2] {
                        if d != req as usize {
                            continue;
                        }
                    }
                    let c = 13 - s - h - d;
                    if let Some(req) = pattern[3] {
                        if c != req as usize {
                            continue;
                        }
                    }
                    mask.set(shape_to_index(s, h, d, c));
                }
            }
        }
        mask
    }

    /// Create a mask for an "any" distribution (order doesn't matter).
    /// E.g., any_distribution([4,3,3,3]) matches 4-3-3-3 in any suit order.
    pub fn any_distribution(pattern: [u8; 4]) -> Self {
        use std::collections::HashSet;

        let mut mask = ShapeMask::empty();
        let mut seen = HashSet::new();

        // Generate all permutations of the pattern
        let perms = permutations_of_4(pattern);
        for perm in perms {
            if seen.insert(perm) {
                mask.set(shape_to_index(
                    perm[0] as usize,
                    perm[1] as usize,
                    perm[2] as usize,
                    perm[3] as usize,
                ));
            }
        }
        mask
    }

    /// Create a mask for an "any wildcard" pattern (permutations of wildcard pattern).
    /// E.g., any_wildcard([Some(6), None, None, None]) matches any distribution
    /// where one suit has 6 cards (6-x-x-x in any suit order).
    pub fn any_wildcard(pattern: [Option<u8>; 4]) -> Self {
        use std::collections::HashSet;

        let mut mask = ShapeMask::empty();
        let mut seen_patterns = HashSet::new();

        // Generate all permutations of the wildcard pattern
        let perms = permute_option_pattern(pattern);
        for perm in perms {
            if seen_patterns.insert(perm) {
                // For each permutation, create a wildcard mask and union it
                let wildcard_mask = Self::wildcard(perm);
                mask = mask.union(&wildcard_mask);
            }
        }
        mask
    }
}

/// Generate all 24 permutations of a 4-element array.
fn permutations_of_4(arr: [u8; 4]) -> Vec<[u8; 4]> {
    let indices = [0, 1, 2, 3];
    let mut result = Vec::with_capacity(24);

    for perm in permute_indices(&indices) {
        result.push([arr[perm[0]], arr[perm[1]], arr[perm[2]], arr[perm[3]]]);
    }
    result
}

/// Generate all 24 permutations of an Option<u8> pattern.
fn permute_option_pattern(arr: [Option<u8>; 4]) -> Vec<[Option<u8>; 4]> {
    let indices = [0, 1, 2, 3];
    let mut result = Vec::with_capacity(24);

    for perm in permute_indices(&indices) {
        result.push([arr[perm[0]], arr[perm[1]], arr[perm[2]], arr[perm[3]]]);
    }
    result
}

/// Generate all permutations of 4 indices.
fn permute_indices(indices: &[usize; 4]) -> Vec<[usize; 4]> {
    let mut result = Vec::with_capacity(24);
    let mut arr = *indices;

    // Heap's algorithm for generating permutations
    fn heap_permute(n: usize, arr: &mut [usize; 4], result: &mut Vec<[usize; 4]>) {
        if n == 1 {
            result.push(*arr);
            return;
        }
        for i in 0..n {
            heap_permute(n - 1, arr, result);
            if n.is_multiple_of(2) {
                arr.swap(i, n - 1);
            } else {
                arr.swap(0, n - 1);
            }
        }
    }

    heap_permute(4, &mut arr, &mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_to_index_bounds() {
        // First shape: 0-0-0-13
        assert_eq!(shape_to_index(0, 0, 0, 13), 0);
        // Last shape: 13-0-0-0
        assert_eq!(shape_to_index(13, 0, 0, 0), 559);
    }

    #[test]
    fn test_shape_to_index_unique() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();

        for s in 0..14 {
            for h in 0..(14 - s) {
                for d in 0..(14 - s - h) {
                    let c = 13 - s - h - d;
                    let idx = shape_to_index(s, h, d, c);
                    assert!(
                        idx < 560,
                        "Index {} out of range for shape {}-{}-{}-{}",
                        idx,
                        s,
                        h,
                        d,
                        c
                    );
                    assert!(
                        seen.insert(idx),
                        "Duplicate index {} for shape {}-{}-{}-{}",
                        idx,
                        s,
                        h,
                        d,
                        c
                    );
                }
            }
        }
        assert_eq!(seen.len(), 560);
    }

    #[test]
    fn test_exact_shape() {
        let mask = ShapeMask::exact(5, 4, 3, 1);
        assert!(mask.matches(5, 4, 3, 1));
        assert!(!mask.matches(5, 4, 2, 2));
        assert!(!mask.matches(4, 4, 3, 2));
        assert_eq!(mask.count(), 1);
    }

    #[test]
    fn test_wildcard_shape() {
        // 5-4-x-x
        let mask = ShapeMask::wildcard([Some(5), Some(4), None, None]);
        assert!(mask.matches(5, 4, 3, 1));
        assert!(mask.matches(5, 4, 2, 2));
        assert!(mask.matches(5, 4, 4, 0));
        assert!(!mask.matches(5, 3, 3, 2));
        assert!(!mask.matches(4, 4, 3, 2));
        // 5 spades, 4 hearts, remaining 4 cards can be 4-0, 3-1, 2-2, 1-3, 0-4 = 5 shapes
        assert_eq!(mask.count(), 5);
    }

    #[test]
    fn test_any_distribution() {
        // any 4333
        let mask = ShapeMask::any_distribution([4, 3, 3, 3]);
        assert!(mask.matches(4, 3, 3, 3));
        assert!(mask.matches(3, 4, 3, 3));
        assert!(mask.matches(3, 3, 4, 3));
        assert!(mask.matches(3, 3, 3, 4));
        assert!(!mask.matches(4, 4, 3, 2));
        // 4 permutations (which suit has 4)
        assert_eq!(mask.count(), 4);
    }

    #[test]
    fn test_any_distribution_5431() {
        // any 5431 - all different, so 24 permutations
        let mask = ShapeMask::any_distribution([5, 4, 3, 1]);
        assert!(mask.matches(5, 4, 3, 1));
        assert!(mask.matches(1, 3, 4, 5));
        assert!(mask.matches(4, 5, 1, 3));
        assert!(!mask.matches(5, 4, 2, 2));
        assert_eq!(mask.count(), 24);
    }

    #[test]
    fn test_union() {
        let mask1 = ShapeMask::exact(5, 4, 3, 1);
        let mask2 = ShapeMask::exact(4, 4, 3, 2);
        let combined = mask1.union(&mask2);
        assert!(combined.matches(5, 4, 3, 1));
        assert!(combined.matches(4, 4, 3, 2));
        assert!(!combined.matches(5, 5, 2, 1));
        assert_eq!(combined.count(), 2);
    }

    #[test]
    fn test_difference() {
        // any 4333 minus exact 4-3-3-3 (spades = 4)
        let any = ShapeMask::any_distribution([4, 3, 3, 3]);
        let exact = ShapeMask::exact(4, 3, 3, 3);
        let result = any.difference(&exact);
        assert!(!result.matches(4, 3, 3, 3));
        assert!(result.matches(3, 4, 3, 3));
        assert!(result.matches(3, 3, 4, 3));
        assert!(result.matches(3, 3, 3, 4));
        assert_eq!(result.count(), 3);
    }

    #[test]
    fn test_all_mask() {
        let mask = ShapeMask::all();
        assert_eq!(mask.count(), 560);
        assert!(mask.matches(0, 0, 0, 13));
        assert!(mask.matches(13, 0, 0, 0));
        assert!(mask.matches(4, 3, 3, 3));
    }

    #[test]
    fn test_complement() {
        let exact = ShapeMask::exact(5, 4, 3, 1);
        let complement = exact.complement();
        assert_eq!(complement.count(), 559);
        assert!(!complement.matches(5, 4, 3, 1));
        assert!(complement.matches(4, 4, 3, 2));
    }
}
