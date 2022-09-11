use std::iter;

use bloomy::BloomFilter;
use criterion::Criterion;

fn key() -> String {
    let rng = fastrand::Rng::new();
    iter::repeat_with(|| rng.alphanumeric()).take(32).collect()
}

fn populate(bf: &mut BloomFilter<String>, n: usize) {
    for _ in 0..n {
        let item = key();
        bf.insert(&item);
    }
}

fn bench_bloom_filter_insert(c: &mut Criterion) {
    c.bench_function("insert-1000", |b| {
        let mut bf = BloomFilter::new(1000);

        b.iter(|| {
            let item = key();
            bf.insert(&item);
        });
    });

    c.bench_function("insert-10000", |b| {
        let mut bf = BloomFilter::new(10000);

        b.iter(|| {
            let item = key();
            bf.insert(&item);
        });
    });
}

fn bench_bloom_filter_check(c: &mut Criterion) {
    c.bench_function("check-1000", |b| {
        let n = 1000;
        let mut bf = BloomFilter::new(n);
        populate(&mut bf, n);

        b.iter(|| {
            let item = key();
            bf.contains(&item);
        });
    });

    c.bench_function("check-10000", |b| {
        let n = 10000;
        let mut bf = BloomFilter::new(n);
        populate(&mut bf, n);

        b.iter(|| {
            let item = key();
            bf.contains(&item);
        });
    });
}

criterion::criterion_group!(benches, bench_bloom_filter_insert, bench_bloom_filter_check);
criterion::criterion_main!(benches);
