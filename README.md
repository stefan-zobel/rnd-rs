[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](https://github.com/stefan-zobel/rnd-rs)

# rnd-rs

A small Rust library of non-cryptographic PRNGs ported from Java.

This module provides a few different implementations of cryptographically **insecure** random number generators suitable for numeric simulations.

The default algorithm which is used in the thread-local generator [ThreadLocalPrng](https://github.com/stefan-zobel/rnd-rs/blob/master/src/pseudo_random.rs#L495)
is <a href=https://github.com/tylov/STC/blob/master/docs/crandom_api.md>Tyge Løvset's stc64 generator</a>
which is implemented in [Stc64](https://github.com/stefan-zobel/rnd-rs/blob/master/src/pseudo_random.rs#L246).

Another fast high quality algorithm is <a href=https://arxiv.org/pdf/1805.01407.pdf>Blackman and Vigna's (2019) xoshiro256**</a>
which is provided by [XoShiRo256StarStar](https://github.com/stefan-zobel/rnd-rs/blob/master/src/pseudo_random.rs#L328).

For applications that use tuples of consecutively generated values, it may be desirable to use a generator that is k-dimensionally
equidistributed such that k is at least as large as the length of the tuples being generated.
The generator [Lcg64Xor1024Mix](https://github.com/stefan-zobel/rnd-rs/blob/master/src/pseudo_random.rs#L422), which is a Rust port of Java's
<a href=https://github.com/openjdk/jdk/blob/master/src/jdk.random/share/classes/jdk/random/L64X1024MixRandom.java>L64X1024MixRandom</a>
algorithm is provably 16-dimensionally equidistributed. This generator has a much larger period (2<sup>64</sup>(2<sup>1024</sup>&minus;1))
and state space (1088 bits) than the other generators and is about 3 to 4 times slower than
[Stc64](https://github.com/stefan-zobel/rnd-rs/blob/master/src/pseudo_random.rs#L246).

All of these algorithms have good performance in statistical tests and so far no major issues are known. **None** of them is
cryptographically secure. A weakness of the current implementation is that all of them can only be seeded by a single `i64`
which is theoretically insufficient for the state space these generators have. However, this should hardly be detectable
in actual simulations.
