// Copyright (c) 2018 Aleksandr Bezobchuk
// Copyright (c) 2022 Alexis Sellier
//
// Licensed under the MIT license.

//! A simple implementation of a Bloom filter using enhanced double hashing.

use std::f64;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use siphasher::sip::SipHasher13;

use crate::bitvec::BitVec;

/// The default false positive probability value, 1%.
pub const DEFAULT_FALSE_POSITIVE_RATE: f64 = 0.01;

/// `ln` squared.
const LN_SQR: f64 = f64::consts::LN_2 * f64::consts::LN_2;

/// Seeds used for SipHash.
const HASHER_SEEDS: [[u8; 16]; 2] = [
    [
        136, 168, 28, 251, 141, 239, 69, 38, 166, 209, 98, 201, 2, 169, 146, 170,
    ],
    [
        103, 236, 177, 212, 54, 11, 66, 5, 194, 86, 6, 254, 82, 93, 203, 37,
    ],
];

/// A Bloom filter that keeps track of items of type `K`.
#[derive(Clone, Debug)]
pub struct BloomFilter<K> {
    bits: BitVec,
    nhashes: usize,
    hashers: [SipHasher13; 2],
    key: PhantomData<K>,
}

impl<K: Hash> BloomFilter<K> {
    /// Return a new Bloom filter with a given approximate item capacity.
    /// The default false positive probability is set and defined by [`DEFAULT_FALSE_POS`].
    pub fn new(capacity: usize) -> BloomFilter<K> {
        BloomFilter::with_rate(capacity, DEFAULT_FALSE_POSITIVE_RATE)
    }

    /// Return a new Bloom filter given a size in bytes for the filter.
    pub fn with_size(nbytes: usize) -> BloomFilter<K> {
        let nbits = nbytes * 8;
        let capacity = optimal_capacity(nbits, DEFAULT_FALSE_POSITIVE_RATE);
        let nhashes = optimal_hashes(nbits, capacity);
        let hashers = [
            SipHasher13::new_with_key(&HASHER_SEEDS[0]),
            SipHasher13::new_with_key(&HASHER_SEEDS[1]),
        ];

        BloomFilter {
            bits: BitVec::new(nbits as usize),
            nhashes,
            hashers,
            key: PhantomData,
        }
    }

    /// Return a new Bloom filter with a given approximate item capacity
    /// and a desired false positive rate.
    pub fn with_rate(capacity: usize, fp_rate: f64) -> BloomFilter<K> {
        let nbits = optimal_bits(capacity, fp_rate);
        let nhashes = optimal_hashes(nbits, capacity);
        let hashers = [
            SipHasher13::new_with_key(&HASHER_SEEDS[0]),
            SipHasher13::new_with_key(&HASHER_SEEDS[1]),
        ];

        BloomFilter {
            bits: BitVec::new(nbits as usize),
            nhashes,
            hashers,
            key: PhantomData,
        }
    }

    /// Set an item in the Bloom filter. This operation is idempotent with regards
    /// to each unique item. Each item must implement the Hash trait.
    pub fn insert(&mut self, item: &K) {
        let (h1, h2) = self.sip_hashes(item);

        for i in 0..self.nhashes {
            let index = self.bloom_hash(h1, h2, i as u64) as usize;
            self.bits.set(index);
        }
    }

    /// Return whether or not a given item is likely in the Bloom filter or not. There is a
    /// possibility for a false positive with the probability being under the Bloom filter's `p`
    /// value, but a false negative will never occur.
    pub fn contains(&self, item: &K) -> bool {
        let (h1, h2) = self.sip_hashes(item);

        for i in 0..self.nhashes {
            let index = self.bloom_hash(h1, h2, i as u64) as usize;
            if !self.bits.is_set(index) {
                return false;
            }
        }
        true
    }

    /// Set all bits to zero.
    pub fn clear(&mut self) {
        self.bits.clear();
    }

    /// Return the number of bits in this filter.
    pub fn bits(&self) -> usize {
        self.bits.len()
    }

    /// Number of hashes used (`k` parameter).
    pub fn hashes(&self) -> usize {
        self.nhashes
    }

    /// Count the approximate number of items in the filter.
    pub fn count(&self) -> usize {
        let nbits = self.bits.len() as f64;
        let nbits_set = self.bits.count_ones() as f64;
        let nhashes = self.nhashes as f64;
        let count = -(nbits / nhashes) * (1. - (nbits_set / nbits)).ln();

        count.round() as usize
    }

    /// Compute the approximate similarity between two filters using the Jaccard Index.
    pub fn similarity(&self, other: &Self) -> f64 {
        assert!(
            self.is_comparable(other),
            "unable to compare filters with different configurations"
        );
        let intersection = self.intersection(other).count() as f64;
        let union = self.union(other).count() as f64;

        intersection / union
    }

    /// Compute the approximate overlap between two filters using the overlap coefficient.
    pub fn overlap(&self, other: &Self) -> f64 {
        assert!(
            self.is_comparable(other),
            "unable to compare filters with different configurations"
        );
        let intersection = self.intersection(other).count() as f64;
        let smallest = usize::min(self.count(), other.count()) as f64;

        intersection / smallest
    }

    /// Compute the union of two Bloom filters.
    pub fn union(&self, other: &Self) -> Self {
        assert!(
            self.is_comparable(other),
            "unable to union filters with different configurations"
        );
        let bits = self.bits.union(&other.bits);

        Self {
            bits,
            nhashes: self.nhashes,
            hashers: self.hashers,
            key: self.key,
        }
    }

    /// Compute the intersection of two Bloom filters.
    pub fn intersection(&self, other: &Self) -> Self {
        assert!(
            self.is_comparable(other),
            "unable to intersect filters with different configurations"
        );
        let bits = self.bits.intersection(&other.bits);

        Self {
            bits,
            nhashes: self.nhashes,
            hashers: self.hashers,
            key: self.key,
        }
    }

    /// Check whether two filters can be compared, intersected and unioned.
    pub fn is_comparable(&self, other: &Self) -> bool {
        self.nhashes == other.nhashes
            && self.bits.len() == other.bits.len()
            && self.hashers[0].keys() == other.hashers[0].keys()
            && self.hashers[1].keys() == other.hashers[1].keys()
    }

    /// Return the underlying bytes storage.
    pub fn as_bytes(&self) -> &[u8] {
        self.bits.as_bytes()
    }

    fn sip_hashes(&self, item: &K) -> (u64, u64) {
        let mut sip1 = self.hashers[0];
        let mut sip2 = self.hashers[1];

        item.hash(&mut sip1);
        item.hash(&mut sip2);

        let h1 = sip1.finish();
        let h2 = sip2.finish();

        (h1, h2)
    }

    fn bloom_hash(&self, h1: u64, h2: u64, i: u64) -> u64 {
        let r = h1.wrapping_add(i.wrapping_mul(h2)).wrapping_add(i.pow(3));
        r % self.bits() as u64
    }
}

/// Return the optimal bit vector size for a Bloom filter given an approximate
/// size and a desired false positive rate.
pub fn optimal_bits(capacity: usize, fp_rate: f64) -> usize {
    (-((fp_rate.ln() * (capacity as f64)) / LN_SQR)).ceil() as usize
}

/// Return the optimal item capacity of a filter given a bit vector size and false positive rate.
pub fn optimal_capacity(nbits: usize, fp_rate: f64) -> usize {
    ((-(nbits as f64) * LN_SQR) / fp_rate.ln()).round() as usize
}

/// Return the optimal number of hash functions for a Bloom filter given a
/// bit vector size and an approximate set size.
///
/// Also called `k`.
pub fn optimal_hashes(nbits: usize, capacity: usize) -> usize {
    (((nbits / capacity) as f64) * f64::consts::LN_2).ceil() as usize
}

impl<K> AsRef<[u8]> for BloomFilter<K> {
    fn as_ref(&self) -> &[u8] {
        self.bits.as_bytes()
    }
}

impl<K> PartialEq for BloomFilter<K> {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits && self.nhashes == other.nhashes
    }
}

impl<K> Eq for BloomFilter<K> {}

impl<K> From<Vec<u8>> for BloomFilter<K> {
    fn from(other: Vec<u8>) -> BloomFilter<K> {
        let bits = BitVec::from(other);
        let capacity = optimal_capacity(bits.len(), DEFAULT_FALSE_POSITIVE_RATE);
        let nhashes = optimal_hashes(bits.len(), capacity);
        let hashers = [
            SipHasher13::new_with_key(&HASHER_SEEDS[0]),
            SipHasher13::new_with_key(&HASHER_SEEDS[1]),
        ];

        Self {
            bits,
            nhashes,
            hashers,
            key: PhantomData,
        }
    }
}

impl<K> From<BloomFilter<K>> for Vec<u8> {
    fn from(other: BloomFilter<K>) -> Vec<u8> {
        other.bits.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::iter;

    fn key() -> String {
        let rng = fastrand::Rng::new();
        iter::repeat_with(|| rng.alphanumeric()).take(32).collect()
    }

    fn items(size: usize) -> Vec<String> {
        let mut items = HashSet::<String>::new();
        for _ in 0..size {
            items.insert(key());
        }
        items.into_iter().collect()
    }

    #[test]
    fn test_bloom_filter() {
        let n = 1024;
        let items = items(n);
        let mut bf = BloomFilter::<String>::new(items.len());

        // Test inclusion.
        for item in items.iter() {
            bf.insert(item);

            assert_eq!(
                bf.contains(item),
                true,
                "item {} should result in a positive inclusion",
                item,
            );
        }

        // Test false negatives.
        for _ in 0..n {
            let item = key();
            let exists = bf.contains(&item);

            if items.contains(&item) {
                assert_eq!(exists, true, "item {} resulted in a false negative", item);
            }
        }
    }

    #[test]
    fn test_with_size() {
        let bf = BloomFilter::<String>::with_size(32 * 1024); // 32 KB

        assert_eq!(bf.bits(), 32 * 1024 * 8);
    }

    #[test]
    fn test_union() {
        let a_items = items(128);
        let mut a = BloomFilter::<String>::new(a_items.len());
        for item in &a_items {
            a.insert(item);
        }

        let b_items = items(128);
        let mut b = BloomFilter::new(b_items.len());
        for item in &b_items {
            b.insert(item);
        }

        let union = a.union(&b);
        for item in a_items.iter().chain(b_items.iter()) {
            assert!(union.contains(item));
        }
    }

    #[test]
    fn test_intersection() {
        let mut a = BloomFilter::<u8>::new(3);
        let mut b = a.clone();

        a.insert(&1);
        a.insert(&2);
        a.insert(&3);

        b.insert(&3);
        b.insert(&4);
        b.insert(&5);

        let intersection = a.intersection(&b);

        assert!(!intersection.contains(&1));
        assert!(!intersection.contains(&2));
        assert!(intersection.contains(&3));
        assert!(!intersection.contains(&4));
        assert!(!intersection.contains(&5));
    }

    #[test]
    fn test_count() {
        let mut a = BloomFilter::<u16>::new(4096);

        for i in 0..12 {
            a.insert(&i);
        }
        assert_eq!(a.count(), 12);

        for i in 0..2048 {
            a.insert(&i);
        }
        assert_eq!(a.count(), 2048);
    }

    #[test]
    fn test_similarity_and_overlap_small() {
        let mut a = BloomFilter::<i32>::new(4096);
        let mut b = BloomFilter::<i32>::new(4096);

        for i in 0..1024 {
            a.insert(&i);
        }
        for i in 1024..2048 {
            b.insert(&i);
        }
        assert!(BloomFilter::similarity(&a, &b) < 0.08);
        assert!(BloomFilter::overlap(&a, &b) < 0.16);
        assert_eq!(BloomFilter::similarity(&a, &a), 1.0);
        assert_eq!(BloomFilter::similarity(&b, &b), 1.0);
    }

    #[test]
    fn test_similarity_and_overlap_medium() {
        let mut a = BloomFilter::<i32>::new(2048);
        let mut b = BloomFilter::<i32>::new(2048);

        for i in 0..128 {
            a.insert(&i);
        }
        for i in 64..128 {
            b.insert(&i);
        }
        assert_eq!(a.similarity(&b), 0.5);
        assert_eq!(a.overlap(&b), 1.0);
    }

    #[test]
    fn test_similarity_and_overlap_large() {
        let mut a = BloomFilter::<i32>::new(4096);
        let mut b = BloomFilter::<i32>::new(4096);

        for i in 0..128 {
            a.insert(&i);
        }
        for i in 64..192 {
            b.insert(&i);
        }
        assert_eq!(a.similarity(&b), 1. / 3.);
        assert_eq!(a.overlap(&b), 0.5);
    }

    #[test]
    fn test_optimal_bits() {
        assert_eq!(optimal_bits(10, 0.04), 67);
        assert_eq!(optimal_bits(5000, 0.01), 47926);
        assert_eq!(optimal_bits(100000, 0.01), 958506);
    }

    #[test]
    fn test_optimal_hashes() {
        assert_eq!(optimal_hashes(67, 10), 5);
        assert_eq!(optimal_hashes(47926, 5000), 7);
        assert_eq!(optimal_hashes(958506, 100000), 7);
    }

    #[test]
    fn test_optimal_capacity() {
        assert_eq!(optimal_capacity(optimal_bits(128, 0.01), 0.01), 128);
        assert_eq!(optimal_capacity(optimal_bits(84198, 0.03), 0.03), 84198);
        assert_eq!(optimal_capacity(optimal_bits(958472, 0.04), 0.04), 958472);
    }

    #[test]
    fn test_raw() {
        let size = 2 ^ 14;
        let mut a = BloomFilter::<String>::with_size(size);

        for item in items(2 ^ 10).iter() {
            a.insert(item);
        }

        let bytes: Vec<u8> = a.clone().into();
        let b = BloomFilter::from(bytes);

        assert_eq!(a, b);
        assert_eq!(a.bits(), b.bits());
        assert_eq!(a.hashes(), b.hashes());
    }
}
