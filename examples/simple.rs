//! A simple example showing the use of a Bloom filter.
use bloomy::BloomFilter;

fn main() {
    let capacity = 128;
    let mut bf = BloomFilter::new(capacity);

    bf.insert(&"foo");
    bf.insert(&"bar");

    bf.contains(&"foo"); // true
    bf.contains(&"bar"); // true
    bf.contains(&"baz"); // false

    bf.count(); // 2
}
