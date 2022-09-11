# Bloomy

A minimal implementation of a Bloom filter in Rust.

Bloom filters are a space-efficient probabilistic data structure invented by
Burton Howard Bloom in the 1970s.

This crate combines ideas and code from various other Bloom filter
crates.

The underlying bit vector implementation is adapted from existing code by
Helge Wrede, Alexander Schultheiß and Lukas Simon.

In comparison with other crates, `bloomy` combines the following advantages:
* Computationally efficient by using a double hashing technique pioneered
by Adam Kirsch and Michael Mitzenmacher. You can find a copy of the paper in
the `docs/` folder.
* Has only a single dependency: `siphasher`, from which multiple hashers are
derived, and hence doesn't depend on the `bitvec` or `bit-vec` crates.
* Supports *union* and *intersection* operations.
* Supports *counting* items and *similarity* metrics.

Usage
-----
Add the following to your `Cargo.toml`:

    [dependencies]
    bloomy = "1"

Check the `examples/` folder for usage examples.

License
-------
Licensed under the MIT license.
