// Copyright 2022 Stefan Zobel
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//!
//! This module provides a few different implementations of cryptographically **insecure** random
//! number generators suitable for numeric simulations.
//!
//! The default algorithm which is used in the thread-local generator [ThreadLocalPrng](ThreadLocalPrng) is
//! <a href=https://github.com/tylov/STC/blob/master/docs/crandom_api.md>Tyge Løvset's stc64 generator</a>
//! which is implemented in [Stc64](Stc64).
//!
//! Another fast high quality algorithm is <a href=https://arxiv.org/pdf/1805.01407.pdf>Blackman and Vigna's (2019) xoshiro256**</a>
//! which is provided by [XoShiRo256StarStar](XoShiRo256StarStar),
//!
//! For applications that use tuples of consecutively generated values, it may be desirable
//! to use a generator that is k-dimensionally equidistributed such that k is at least as
//! large as the length of the tuples being generated.
//! The generator [Lcg64Xor1024Mix](Lcg64Xor1024Mix), which is a Rust port of Java's
//! <a href=https://github.com/openjdk/jdk/blob/master/src/jdk.random/share/classes/jdk/random/L64X1024MixRandom.java>L64X1024MixRandom</a>
//! algorithm is provably 16-dimensionally equidistributed. This generator has a much larger
//! period (2<sup>64</sup>(2<sup>1024</sup>&minus;1)) and state space (1088 bits) than the
//! other generators and is about 3 to 4 times slower than [Stc64](Stc64).
//!
//! All of these algorithms have good performance in statistical tests and so far no major issues
//! are known. **None** of them is cryptographically secure. A weakness of the current implementation
//! is that all of them can only be seeded by a single `i64` which is theoretically insufficient
//! for the state space these generators have. However, this should hardly be detectable in actual
//! simulations.
//!

use crate::bit_mix::lea_mix64;
use crate::seed::black_hole;
use crate::xor_shift_128plus::XorShift128Plus;
use core::cell::UnsafeCell;
use core::ptr::NonNull;

const DOUBLE_NORM: f64 = 1.0f64 / (1i64 << 53) as f64;
const FLOAT_NORM: f32 = 1.0f32 / (1i32 << 24) as f32;

/// A generator of uniform pseudorandom values.
///
/// Implementors need to supply an implementation of the [`next_long`](Self::next_long) method.
///
/// The default implementations in this trait are efficient for methods that need
/// at least 33 bits of randomness but somehow wasteful for the other methods because
/// it dissipates valuable random bits piled up in the call to `next_long()` whenever
/// less than 33 random bits are needed for the result type.
pub trait PseudoRandom {
    /// Returns a uniformly distributed signed 64-bit integer.
    fn next_long(&mut self) -> i64;

    /// Returns a uniformly distributed signed 32-bit integer.
    #[inline]
    fn next_int(&mut self) -> i32 {
        ((self.next_long() as u64 >> 32) as i64) as i32
    }

    /// Returns a uniformly distributed 64-bit floating point value.
    #[inline]
    fn next_double(&mut self) -> f64 {
        ((self.next_long() as u64 >> 11) as i64) as f64 * DOUBLE_NORM
    }

    /// Returns a uniformly distributed 32-bit floating point value.
    #[inline]
    fn next_float(&mut self) -> f32 {
        ((self.next_long() as u64 >> 40) as i64) as f32 * FLOAT_NORM
    }

    /// Returns an equi-distributed boolean value.
    #[inline]
    fn next_bool(&mut self) -> bool {
        self.next_long() < 0i64
    }

    /// Returns a uniformly distributed `i64` value between `0` (inclusive) and `n` (exclusive)
    /// where `n` is the **strictly positive** bound on the random number to be returned.
    /// This method panics when `n` is `<= 0`.
    ///
    /// # Panics
    ///
    /// Panics if `n` is zero or negative.
    #[inline]
    fn next_long_up_to(&mut self, n: i64) -> i64 {
        if n <= 0i64 {
            panic!("n must be strictly positive");
        }
        let n_minus1 = n - 1i64;
        let mut x = self.next_long();
        if (n & n_minus1) == 0i64 {
            // power of two shortcut
            return x & n_minus1;
        }
        // rejection-based algorithm to get uniform longs
        let mut y = (x as u64 >> 1) as i64;
        loop {
            x = y % n;
            if y + n_minus1 - x < 0i64 {
                break;
            }
            y = (self.next_long() as u64 >> 1) as i64;
        }

        x
    }

    /// Returns a uniformly distributed `i32` value between `0` (inclusive) and `n` (exclusive)
    /// where `n` is the **strictly positive** bound on the random number to be returned.
    /// This method panics when `n` is `<= 0`.
    ///
    /// # Panics
    ///
    /// Panics if `n` is zero or negative.
    #[inline]
    fn next_int_up_to(&mut self, n: i32) -> i32 {
        self.next_long_up_to(n as i64) as i32
    }

    /// Returns a signed 64-bit integer which is uniformly distributed in the interval [min, max].
    ///
    /// # Panics
    ///
    /// Panics if `max < min`.
    #[inline]
    fn next_long_from_interval(&mut self, min: i64, max: i64) -> i64 {
        min + self.next_long_up_to((max - min) + 1i64)
    }

    /// Returns a signed 32-bit integer which is uniformly distributed in the interval [min, max].
    ///
    /// # Panics
    ///
    /// Panics if `max < min`.
    #[inline]
    fn next_int_from_interval(&mut self, min: i32, max: i32) -> i32 {
        self.next_long_from_interval(min as i64, max as i64) as i32
    }

    /// Returns a 64-bit floating point value which is uniformly distributed in the interval [min, max).
    #[inline]
    fn next_double_from_interval(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_double()
    }

    /// Returns a 32-bit floating point value which is uniformly distributed in the interval [min, max).
    #[inline]
    fn next_float_from_interval(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_float()
    }

    /// Generates random bytes and places them into the user-supplied `bytes` slice.
    /// The number of random bytes produced is equal to the length of the slice.
    #[inline]
    fn next_bytes(&mut self, bytes: &mut [u8]) {
        let len = bytes.len();
        let mut i: usize = 0usize;
        loop {
            if i == len {
                break;
            }
            let mut rnd = self.next_long();
            let mut n = len - i;
            n = n.min(8);
            loop {
                if n == 0 {
                    break;
                }
                n -= 1;
                bytes[i] = rnd as u8;
                i += 1;
                rnd >>= 8;
            }
        }
    }

    /// Generates random signed 64-bit integers and places them into the user-supplied `longs`
    /// slice. The number of random integers produced is equal to the length of the slice.
    #[inline]
    fn next_longs(&mut self, longs: &mut [i64]) {
        for l in longs {
            *l = self.next_long();
        }
    }

    /// Generates random 64-bit floating point values and places them into the user-supplied
    /// `doubles` slice. The number of random integers produced is equal to the length of the slice.
    #[inline]
    fn next_doubles(&mut self, doubles: &mut [f64]) {
        for d in doubles {
            *d = self.next_double();
        }
    }

    /// Generates two standard normal distributed 64-bit floating point values and stores
    /// them into the user-supplied `out` array. Here, "standard normal" means `N(0, 1)`, i.e.
    /// a normal distribution with expectation `0` and variance `1`.
    #[inline]
    fn next_gaussians(&mut self, out: &mut [f64; 2]) {
        // Marsaglia's polar method
        let mut u1: f64;
        let mut u2: f64;
        let mut q: f64;
        loop {
            u1 = 2.0f64 * self.next_double() - 1.0f64; // between -1 and 1
            u2 = 2.0f64 * self.next_double() - 1.0f64; // between -1 and 1
            q = u1 * u1 + u2 * u2;
            if q < 1.0f64 && q != 0.0f64 {
                break;
            }
        }
        let p = (-2.0f64 * q.ln() / q).sqrt();
        out[0] = u1 * p;
        out[1] = u2 * p;
    }
}

/// Implement `PseudoRandom` for references to a `PseudoRandom`.
impl<'a, R: PseudoRandom + ?Sized> PseudoRandom for &'a mut R {
    #[inline(always)]
    fn next_long(&mut self) -> i64 {
        (**self).next_long()
    }
}

/// Implement `PseudoRandom` for boxed references to a `PseudoRandom`.
impl<R: PseudoRandom + ?Sized> PseudoRandom for Box<R> {
    #[inline(always)]
    fn next_long(&mut self) -> i64 {
        (**self).next_long()
    }
}

/// The 256-bit generator `Stc64` is Tyge Løvset's improved variation of
/// `Sfc64`. See
/// <https://github.com/tylov/STC/blob/master/include/stc/crandom.h>.
///
/// This generator has a guaranteed period of at least 2<sup>64</sup>
/// and an average period of 2<sup>255</sup>.
///
/// This is the fastest generator supplied in this crate.
#[derive(Debug, Clone)]
pub struct Stc64 {
    s0: i64,
    s1: i64,
    s2: i64,
    s3: i64,
    seq: i64,
}

impl PseudoRandom for Stc64 {
    #[inline]
    fn next_long(&mut self) -> i64 {
        let xb = self.s1;
        let xc = self.s2;

        self.s3 = self.s3.wrapping_add(self.seq);
        let rnd = (self.s0 ^ self.s3).wrapping_add(xb);

        self.s0 = xb ^ (xb as u64 >> 11) as i64;
        self.s1 = xc.wrapping_add(xc << 3);
        self.s2 = ((xc << 24) | (xc as u64 >> 40) as i64).wrapping_add(rnd);

        rnd
    }
}

impl Stc64 {
    /// Creates a new [Stc64](Stc64) initialized with a random seed.
    #[inline]
    pub fn new() -> Self {
        Stc64::internal_new(XorShift128Plus::new().next_long())
    }

    /// Creates a new [Stc64](Stc64) initialized with the given `seed`.
    #[inline]
    pub fn new_from(seed: i64) -> Self {
        Stc64::internal_new(XorShift128Plus::new_from(seed).next_long())
    }

    #[inline]
    fn internal_new(seed: i64) -> Self {
        let mut instance = Stc64 {
            s0: seed,
            s1: seed.wrapping_add(0x26aa069ea2fb1a4di64),
            s2: seed.wrapping_add(0x70c72c95cd592d04i64),
            s3: seed.wrapping_add(0x504f333d3aa0b359i64),
            // seq must be odd
            seq: (((seed.wrapping_add(0x3504f333d3aa0b37i64)) << 1) | 1i64),
        };
        instance.escape();
        instance
    }

    #[inline]
    fn escape(&mut self) {
        let mut l: i64 = 0i64;
        for _ in 0..12 {
            l = self.next_long();
        }
        if l == 0i64 {
            black_hole(l as u8);
        }
    }
}

impl Default for Stc64 {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// 256-bit `xoshiro256**` pseudo random generator suggested by
/// <a href=https://arxiv.org/pdf/1805.01407.pdf>David Blackman and Sebastiano
/// Vigna (2019)</a>. It is about 40% faster than `XorShift64Star` despite
/// having a 4 times larger state space.
///
/// This generator has a period of 2<sup>256</sup>&nbsp;&minus;&nbsp;1.
///
/// This generator is 4-dimensionally equidistributed.
///
/// This generator is almost as fast as [Stc64](Stc64).
#[derive(Debug, Clone)]
pub struct XoShiRo256StarStar {
    x0: i64,
    x1: i64,
    x2: i64,
    x3: i64,
}

impl PseudoRandom for XoShiRo256StarStar {
    #[inline]
    fn next_long(&mut self) -> i64 {
        let s1 = self.x1;
        let t = s1 << 17;
        let x = s1.wrapping_add(s1 << 2);
        let mut rnd = (x << 7) | (x as u64 >> 57) as i64;
        rnd = rnd.wrapping_add(rnd << 3);

        self.x2 ^= self.x0;
        self.x3 ^= s1;
        self.x1 ^= self.x2;
        let s3 = self.x3;
        self.x0 ^= s3;

        self.x2 ^= t;
        self.x3 = (s3 << 45) | (s3 as u64 >> 19) as i64;

        rnd
    }
}

impl XoShiRo256StarStar {
    /// Creates a new [XoShiRo256StarStar](XoShiRo256StarStar) initialized with a random seed.
    #[inline]
    pub fn new() -> Self {
        XoShiRo256StarStar::internal_new(&mut XorShift128Plus::new())
    }

    /// Creates a new [XoShiRo256StarStar](XoShiRo256StarStar) initialized with the given `seed`.
    #[inline]
    pub fn new_from(seed: i64) -> Self {
        XoShiRo256StarStar::internal_new(&mut XorShift128Plus::new_from(seed))
    }

    #[inline]
    fn internal_new(seeder: &mut XorShift128Plus) -> Self {
        let mut instance = XoShiRo256StarStar {
            x0: seeder.next_long(),
            x1: seeder.next_long(),
            x2: seeder.next_long(),
            x3: seeder.next_long(),
        };
        instance.escape();
        instance
    }

    //noinspection ALL
    #[inline]
    fn escape(&mut self) {
        let mut l: i64 = 0i64;
        for _ in 0..20 {
            l = self.next_long();
        }
        if l == 0i64 {
            black_hole(l as u8);
        }
    }
}

impl Default for XoShiRo256StarStar {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/*
 * Multiplier used in the LCG portion of the algorithm. Chosen based on
 * research by Sebastiano Vigna and Guy Steele (2019). The spectral scores
 * for dimensions 2 through 8 for the multiplier 0xd1342543de82ef95 are
 * [0.958602, 0.937479, 0.870757, 0.822326, 0.820405, 0.813065, 0.760215].
 */
const M: i64 = 0xd1342543de82ef95u64 as i64;

/// The `L64X1024MixRandom` algorithm from JDK 17 which uses a linear
/// congruential generator (LCG) as a first subgenerator and a Xor-based
/// generator (xoroshiro1024) as a second subgenerator and then applies
/// a 64-bit mixing function identified by Doug Lea.
///
/// This generator has a 1088-bit state and a period of
/// 2<sup>64</sup>(2<sup>1024</sup>&minus;1).
///
/// This generator is 16-dimensionally equidistributed.
///
/// This is the slowest generator supplied in this crate.
#[derive(Debug, Clone)]
pub struct Lcg64Xor1024Mix {
    /*
     * The parameter that is used as an additive constant for the LCG.
     * Must be odd.
     */
    a: i64,
    /*
     * The per-instance state: s for the LCG; the array seed for the XBG;
     * pos is the rotating pointer into the array seed. At least one of
     * the 16 elements of the array seed must be nonzero.
     */
    s: i64,
    pos: usize,
    seed: [i64; 16],
}

impl PseudoRandom for Lcg64Xor1024Mix {
    #[inline]
    fn next_long(&mut self) -> i64 {
        // xoroshiro1024: part 1
        let p = self.pos;
        let mut s15 = self.seed[p];
        self.pos = (p + 1) & 15;
        let s0 = self.seed[self.pos];
        // compute result
        let rnd = lea_mix64(self.s.wrapping_add(s0));
        // update LCG sub-generator
        self.s = (M.wrapping_mul(self.s)).wrapping_add(self.a);
        // xoroshiro1024: part 2
        s15 ^= s0;
        self.seed[p] = ((s0 << 25) | (s0 as u64 >> 39) as i64) ^ s15 ^ (s15 << 27);
        self.seed[self.pos] = (s15 << 36) | (s15 as u64 >> 28) as i64;
        rnd
    }
}

impl Lcg64Xor1024Mix {
    /// Creates a new [Lcg64Xor1024Mix](Lcg64Xor1024Mix) initialized with a random seed.
    #[inline]
    pub fn new() -> Self {
        Lcg64Xor1024Mix::internal_new(&mut XorShift128Plus::new())
    }

    /// Creates a new [Lcg64Xor1024Mix](Lcg64Xor1024Mix) initialized with the given `seed`.
    #[inline]
    pub fn new_from(seed: i64) -> Self {
        Lcg64Xor1024Mix::internal_new(&mut XorShift128Plus::new_from(seed))
    }

    #[inline]
    fn internal_new(seeder: &mut XorShift128Plus) -> Self {
        let mut instance = Lcg64Xor1024Mix {
            a: seeder.next_long(),
            s: seeder.next_long(),
            pos: 15usize,
            seed: [0i64; 16],
        };
        for i in 0..16 {
            instance.seed[i] = seeder.next_long();
        }
        instance
    }
}

impl Default for Lcg64Xor1024Mix {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// A thread-local generator that wraps a [Stc64](Stc64) generator.
#[derive(Debug, Copy, Clone)]
pub struct ThreadLocalPrng {
    prng: NonNull<Stc64>,
}

impl PseudoRandom for ThreadLocalPrng {
    #[inline]
    fn next_long(&mut self) -> i64 {
        unsafe { self.prng.as_mut().next_long() }
    }
}

impl ThreadLocalPrng {
    /// Get a thread-local generator that wraps a [Stc64](Stc64) generator.
    #[inline]
    pub fn get() -> Self {
        let ptr = THREAD_LOCAL_PRNG_KEY.with(|t| t.get());
        let stc64 = NonNull::new(ptr).unwrap();
        ThreadLocalPrng { prng: stc64 }
    }
}

thread_local!(
    static THREAD_LOCAL_PRNG_KEY: UnsafeCell<Stc64> = {
        UnsafeCell::new(Stc64::new())
    }
);

impl Default for ThreadLocalPrng {
    fn default() -> ThreadLocalPrng {
        Self::get()
    }
}
