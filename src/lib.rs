//! A simple implementation of a Bloom filter, a space-efficient probabilistic
//! data structure.
//!
//! # Bloom Filters
//!
//! A Bloom filter is a space-efficient probabilistic data structure that is
//! used to test whether an element is a member of a set. It allows for queries
//! to return: "possibly in set" or "definitely not in set". Elements can be
//! added to the set, but not removed; the more elements that are added to the
//! set, the larger the probability of false positives. It has been shown that
//! fewer than 10 bits per element are required for a 1% false positive
//! probability, independent of the size or number of elements in the set.
//!
//! The provided implementation allows you to create a Bloom filter specifying
//! the approximate number of items expected to be inserted and an optional false
//! positive probability. It also allows you to approximate the total number of
//! items in the filter.
//!
//! # Enhanced Double Hashing
//!
//! Enhanced double hashing is used to set bit positions within a bit vector.
//! The choice for double hashing was shown to be effective without any loss in
//! the asymptotic false positive probability, leading to less computation and
//! potentially less need for randomness in practice, by Adam Kirsch and
//! Michael Mitzenmacher in a paper called *Less Hashing, Same Performance: Building
//! a Better Bloom Filter*.
//!
//! The enhanced double hash takes the form of the following formula:
//!
//! g<sub>i</sub>(x) = (H<sub>1</sub>(x) + iH<sub>2</sub>(x) + f(i)) mod m, where
//! H<sub>1</sub> and H<sub>2</sub> are SipHash instantiations, and f(i) = i<sup>3</sup>
//!
//! # Example
//!
//! ```
//! use bloomy::BloomFilter;
//!
//! let capacity = 32;
//! let mut filter = BloomFilter::new(capacity);
//!
//! filter.insert(&"foo");
//! filter.insert(&"bar");
//!
//! filter.contains(&"foo"); // true
//! filter.contains(&"bar"); // true
//! filter.contains(&"baz"); // false
//!
//! filter.count(); // 2
//! ```
#![warn(missing_docs)]
#![allow(clippy::bool_assert_comparison)]

pub mod bitvec;
pub mod bloom;

pub use bloom::BloomFilter;
