// Copyright (c) 2020 Helge Wrede, Alexander Schultheiß, Lukas Simon
// Copyright (c) 2022 Alexis Sellier
//
// Licensed under the MIT license.

//! Bit vector functionality.
use std::fmt::Debug;

/// A packed bit vector.
#[derive(Clone, PartialEq, Eq)]
pub struct BitVec {
    bytes: Vec<u8>,
    nbits: usize,
}

impl BitVec {
    /// Create a new bit vector of the given capacity, in bits.
    pub fn new(capacity: usize) -> Self {
        let byte_length = if capacity % 8 == 0 {
            capacity / 8
        } else {
            1 + capacity / 8
        };

        Self {
            nbits: capacity,
            bytes: vec![0; byte_length],
        }
    }

    /// Get the length in bits of the vector.
    pub fn len(&self) -> usize {
        self.nbits
    }

    /// Check whether this vector is empty, ie. has a length of zero.
    pub fn is_empty(&self) -> bool {
        self.nbits == 0
    }

    /// Set all bits to zero.
    pub fn clear(&mut self) {
        self.bytes.iter_mut().for_each(|b| *b = 0);
    }

    /// Set a single bit to `1`.
    pub fn set(&mut self, index: usize) {
        if index >= self.len() {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                self.len(),
                index,
            )
        }
        let byte_index = index / 8;
        let mask = 0x01 << (index % 8);

        self.bytes[byte_index] |= mask;
    }

    /// Check whether a bit is set.
    pub fn is_set(&self, index: usize) -> bool {
        if index >= self.len() {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                self.len(),
                index,
            )
        }
        let byte_index = index / 8;
        let mask = 0x01 << (index % 8);

        self.bytes[byte_index] & mask == mask
    }

    /// Count the number of `1` bits.
    pub fn count_ones(&self) -> usize {
        self.bytes.iter().map(|b| b.count_ones() as usize).sum()
    }

    /// Count the number of `0` bits.
    pub fn count_zeros(&self) -> usize {
        self.len() - self.count_ones()
    }

    /// Return the union of two bit vectors.
    /// This is a bitwise `OR` of two vectors.
    pub fn union(&self, other: &Self) -> Self {
        if self.nbits != other.nbits {
            panic!(
                "unable to union bitvecs with different lengths: {} and {}",
                self.nbits, other.nbits
            );
        }
        Self {
            bytes: self
                .bytes
                .iter()
                .zip(other.bytes.iter())
                .map(|(a, b)| a | b)
                .collect(),
            nbits: self.nbits,
        }
    }

    /// Return the intersection of two bit vectors.
    /// This is a bitwise `AND` of two vectors.
    pub fn intersection(&self, other: &Self) -> Self {
        if self.nbits != other.nbits {
            panic!(
                "unable to intersect bitvecs with different lengths: {} and {}",
                self.nbits, other.nbits
            );
        }
        Self {
            bytes: self
                .bytes
                .iter()
                .zip(other.bytes.iter())
                .map(|(a, b)| a & b)
                .collect(),
            nbits: self.nbits,
        }
    }

    /// Return the underlying bytes storage.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl From<Vec<u8>> for BitVec {
    fn from(bytes: Vec<u8>) -> Self {
        let nbits = bytes.len() * 8;

        Self { bytes, nbits }
    }
}

impl From<BitVec> for Vec<u8> {
    fn from(other: BitVec) -> Vec<u8> {
        other.bytes
    }
}

impl Debug for BitVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bits: String = (0..self.nbits)
            .map(|i| if self.is_set(i) { '1' } else { '0' })
            .collect();
        write!(f, "BitVec({})", bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitvec_with_length() {
        let bitvec = BitVec::new(1);
        assert_eq!(1, bitvec.nbits);
        assert_eq!(1, bitvec.len());
        assert_eq!(1, bitvec.bytes.len());

        let bitvec = BitVec::new(8);
        assert_eq!(8, bitvec.nbits);
        assert_eq!(8, bitvec.len());
        assert_eq!(1, bitvec.bytes.len());

        let bitvec = BitVec::new(9);
        assert_eq!(9, bitvec.nbits);
        assert_eq!(9, bitvec.len());
        assert_eq!(2, bitvec.bytes.len());
    }

    #[test]
    fn set_first_bit_only() {
        let mut bitvec = BitVec::new(3);
        bitvec.set(0);
        assert_eq!(true, bitvec.is_set(0));
        assert_eq!(false, bitvec.is_set(1));
        assert_eq!(false, bitvec.is_set(2));
    }

    #[test]
    fn set_last_bit_only() {
        let mut bitvec = BitVec::new(9);
        bitvec.set(8);
        for i in 0..8 {
            assert_eq!(false, bitvec.is_set(i));
        }
        assert_eq!(true, bitvec.is_set(8));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn must_set_with_correct_index() {
        BitVec::new(5).set(5);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn must_get_with_correct_index() {
        BitVec::new(12).is_set(12);
    }

    #[test]
    fn set() {
        let mut bitvec = BitVec::new(24);
        for i in 0..24 {
            assert_eq!(false, bitvec.is_set(i));
        }

        bitvec.set(0);
        bitvec.set(7);
        bitvec.set(8);
        bitvec.set(23);

        assert_eq!(true, bitvec.is_set(0));
        assert_eq!(true, bitvec.is_set(7));
        assert_eq!(true, bitvec.is_set(8));
        assert_eq!(true, bitvec.is_set(23));
    }

    #[test]
    fn set_each_bit_one_by_one() {
        let mut bitvec = BitVec::new(9);
        assert_eq!(0, bitvec.count_ones());
        assert_eq!(9, bitvec.count_zeros());

        bitvec.set(0);
        assert_eq!(true, bitvec.is_set(0));
        assert_eq!(1, bitvec.count_ones());
        assert_eq!(8, bitvec.count_zeros());

        bitvec.set(1);
        assert_eq!(true, bitvec.is_set(1));
        assert_eq!(2, bitvec.count_ones());
        assert_eq!(7, bitvec.count_zeros());

        bitvec.set(2);
        assert_eq!(true, bitvec.is_set(2));
        assert_eq!(3, bitvec.count_ones());
        assert_eq!(6, bitvec.count_zeros());

        bitvec.set(3);
        assert_eq!(true, bitvec.is_set(3));
        assert_eq!(4, bitvec.count_ones());
        assert_eq!(5, bitvec.count_zeros());

        bitvec.set(4);
        assert_eq!(true, bitvec.is_set(4));
        assert_eq!(5, bitvec.count_ones());
        assert_eq!(4, bitvec.count_zeros());

        bitvec.set(5);
        assert_eq!(true, bitvec.is_set(5));
        assert_eq!(6, bitvec.count_ones());
        assert_eq!(3, bitvec.count_zeros());

        bitvec.set(6);
        assert_eq!(true, bitvec.is_set(6));
        assert_eq!(7, bitvec.count_ones());
        assert_eq!(2, bitvec.count_zeros());

        bitvec.set(7);
        assert_eq!(true, bitvec.is_set(7));
        assert_eq!(8, bitvec.count_ones());
        assert_eq!(1, bitvec.count_zeros());

        bitvec.set(8);
        assert_eq!(true, bitvec.is_set(8));
        assert_eq!(9, bitvec.count_ones());
        assert_eq!(0, bitvec.count_zeros());
    }

    #[test]
    fn bitvec_union_test() {
        let mut bitvec_a = BitVec::new(6);
        assert_eq!(0, bitvec_a.count_ones());
        assert_eq!(6, bitvec_a.count_zeros());

        bitvec_a.set(0);
        assert_eq!(true, bitvec_a.is_set(0));
        assert_eq!(1, bitvec_a.count_ones());
        assert_eq!(5, bitvec_a.count_zeros());

        bitvec_a.set(3);
        assert_eq!(true, bitvec_a.is_set(3));
        assert_eq!(2, bitvec_a.count_ones());
        assert_eq!(4, bitvec_a.count_zeros());

        let mut bitvec_b = BitVec::new(6);
        assert_eq!(0, bitvec_b.count_ones());
        assert_eq!(6, bitvec_b.count_zeros());

        bitvec_b.set(2);
        assert_eq!(true, bitvec_b.is_set(2));
        assert_eq!(1, bitvec_b.count_ones());
        assert_eq!(5, bitvec_b.count_zeros());

        bitvec_b.set(3);
        assert_eq!(true, bitvec_b.is_set(3));
        assert_eq!(2, bitvec_b.count_ones());
        assert_eq!(4, bitvec_b.count_zeros());

        bitvec_b.set(5);
        assert_eq!(true, bitvec_b.is_set(5));
        assert_eq!(3, bitvec_b.count_ones());
        assert_eq!(3, bitvec_b.count_zeros());

        let bitvec = bitvec_a.union(&bitvec_b);
        assert_eq!(4, bitvec.count_ones());
        assert_eq!(2, bitvec.count_zeros());
        assert_eq!(true, bitvec.is_set(0));
        assert_eq!(true, bitvec.is_set(2));
        assert_eq!(true, bitvec.is_set(3));
        assert_eq!(true, bitvec.is_set(5));
    }

    #[test]
    fn bitvec_intersect_test() {
        let mut bitvec_a = BitVec::new(6);
        assert_eq!(0, bitvec_a.count_ones());
        assert_eq!(6, bitvec_a.count_zeros());

        bitvec_a.set(0);
        assert_eq!(true, bitvec_a.is_set(0));
        assert_eq!(1, bitvec_a.count_ones());
        assert_eq!(5, bitvec_a.count_zeros());

        bitvec_a.set(3);
        assert_eq!(true, bitvec_a.is_set(3));
        assert_eq!(2, bitvec_a.count_ones());
        assert_eq!(4, bitvec_a.count_zeros());

        let mut bitvec_b = BitVec::new(6);
        assert_eq!(0, bitvec_b.count_ones());
        assert_eq!(6, bitvec_b.count_zeros());

        bitvec_b.set(2);
        assert_eq!(true, bitvec_b.is_set(2));
        assert_eq!(1, bitvec_b.count_ones());
        assert_eq!(5, bitvec_b.count_zeros());

        bitvec_b.set(3);
        assert_eq!(true, bitvec_b.is_set(3));
        assert_eq!(2, bitvec_b.count_ones());
        assert_eq!(4, bitvec_b.count_zeros());

        bitvec_b.set(5);
        assert_eq!(true, bitvec_b.is_set(5));
        assert_eq!(3, bitvec_b.count_ones());
        assert_eq!(3, bitvec_b.count_zeros());

        let bitvec = bitvec_a.intersection(&bitvec_b);
        assert_eq!(1, bitvec.count_ones());
        assert_eq!(5, bitvec.count_zeros());
        assert_eq!(false, bitvec.is_set(0));
        assert_eq!(false, bitvec.is_set(2));
        assert_eq!(true, bitvec.is_set(3));
        assert_eq!(false, bitvec.is_set(5));
    }
}
