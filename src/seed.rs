// Copyright 2022 Stefan Zobel
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::bit_mix;
use parking_lot::lock_api::Mutex;
use parking_lot::{const_mutex, RawMutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static mut UNUSED: u8 = 0u8;

static mut SEED_UNIQUIFIER: i64 = 0x1ed8b55fac9deci64;

static LAST_SEED: Mutex<RawMutex, i64> = const_mutex(0i64);

// 2022-01-01 00:00:00 UTC
const EPOCH_OFFSET: Duration = Duration::new(1640995200, 0);

fn nano_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH + EPOCH_OFFSET)
        .expect("time went backwards!")
        .as_nanos() as i64
}

#[inline]
fn next_seed_uniquifier() -> i64 {
    // Pierre L'Ecuyer: "Tables of Linear Congruential Generators
    // of Different Sizes and Good Lattice Structure"
    unsafe {
        // SAFETY: this is safe because we are always
        // protected by the mutex acquired in raw_seed()
        SEED_UNIQUIFIER = SEED_UNIQUIFIER.wrapping_mul(0x106689d45497fdb5i64);
        SEED_UNIQUIFIER
    }
}

fn pseudo_random_seed() -> i64 {
    let seed = next_seed_uniquifier() ^ nano_time();
    bit_mix::stafford_mix13(seed)
}

#[inline]
pub(crate) fn black_hole(val: u8) {
    // SAFETY: this is safe because we really don't care what gets
    // written into the UNUSED location either by a single thread
    // or concurrently by multiple threads, neither will we ever
    // attempt to read that memory location
    unsafe {
        UNUSED = val;
    }
}

pub(crate) fn raw_seed() -> i64 {
    let mut guard = LAST_SEED.lock();
    let last_seed = &mut *guard;
    let mut seed = pseudo_random_seed();
    loop {
        if seed == 0i64 || seed == *last_seed {
            seed = pseudo_random_seed();
        } else {
            break;
        }
    }
    *last_seed = seed;
    seed
}

#[cfg(test)]
mod simple_seed_tests {
    use super::*;
    use chrono::offset::Utc;
    use chrono::DateTime;

    #[test]
    fn test_epoch_begin() {
        let begin = UNIX_EPOCH + EPOCH_OFFSET;
        let datetime: DateTime<Utc> = begin.into();
        let s = datetime.to_string();
        assert_eq!(s, "2022-01-01 00:00:00 UTC");
        println!("epoch begin: {}", s);
    }

    #[test]
    fn test_nano_time() {
        let fst = nano_time();
        assert!(fst > 518_400_000_000_000);
        println!("fst: {}", fst);
        let snd = nano_time();
        println!("snd: {}", snd);
        assert!(snd >= fst);
    }

    //    #[test] // runs into trouble when tests run multi-threaded
    #[allow(dead_code)]
    fn test_next_seed_uniquifier() {
        println!("SEED_UNIQUIFIER: {}", unsafe { SEED_UNIQUIFIER });
        let u1 = next_seed_uniquifier();
        let u2 = next_seed_uniquifier();
        let u3 = next_seed_uniquifier();
        assert_eq!(u1, 3447679086515839964i64);
        assert_eq!(u2, -2942033378085796212i64);
        assert_eq!(u3, 9105146733113736444i64);
        println!("u1: {}", u1);
        println!("u2: {}", u2);
        println!("u3: {}", u3);
    }
}
