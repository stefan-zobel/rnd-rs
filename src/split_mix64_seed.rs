// Copyright 2022 Stefan Zobel
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::{bit_mix::rrxmrrxmsx, seed::raw_seed};
use parking_lot::lock_api::{Mutex, MutexGuard};
use parking_lot::{const_mutex, RawMutex};
use std::ops::DerefMut;

// the golden ratio scaled to 64 bits
pub(crate) const GOLDEN: i64 = 0x9e3779b97f4a7c15u64 as i64;

static STATE: Mutex<RawMutex, i64> = const_mutex(0i64);

#[inline]
fn ensure_state<'a>(guard: &'a mut MutexGuard<'_, RawMutex, i64>) -> &'a mut i64 {
    let state = guard.deref_mut();
    if *state == 0i64 {
        *state = raw_seed();
    }
    state
}

#[inline]
pub(crate) fn seed() -> i64 {
    let mut guard = STATE.lock();
    let state = ensure_state(&mut guard);
    *state = state.wrapping_add(GOLDEN);
    rrxmrrxmsx(*state)
}

#[inline]
pub(crate) fn seed_from(seed: i64) -> i64 {
    let mut s = seed;
    if s == 0i64 {
        s = 0xffea4f554090c1d1u64 as i64;
    }
    rrxmrrxmsx(s.wrapping_add(GOLDEN))
}
