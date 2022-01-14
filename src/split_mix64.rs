// Copyright 2022 Stefan Zobel
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::{
    bit_mix::rrxmrrxmsx, bit_mix::stafford_mix04, bit_mix::xnasam, split_mix64_seed::seed,
    split_mix64_seed::seed_from, split_mix64_seed::GOLDEN,
};

pub(crate) struct SplitMix64 {
    state: i64,
    // Weyl generator step value
    gamma: i64,
}

impl SplitMix64 {
    #[inline]
    pub(crate) fn new() -> Self {
        SplitMix64::internal_new(seed())
    }

    #[allow(unused)]
    #[inline]
    pub(crate) fn new_from(seed: i64) -> Self {
        SplitMix64::internal_new(seed_from(seed))
    }

    #[inline]
    pub(crate) fn next_long(&mut self) -> i64 {
        self.state = self.state.wrapping_add(self.gamma);
        xnasam(self.state)
    }

    #[allow(unused)]
    #[inline]
    pub(crate) fn next_int(&mut self) -> i32 {
        self.state = self.state.wrapping_add(self.gamma);
        stafford_mix04(self.state)
    }

    #[inline]
    fn internal_new(seed: i64) -> Self {
        SplitMix64 {
            state: seed,
            gamma: mix_gamma(seed.wrapping_add(GOLDEN)),
        }
    }
}

#[inline]
const fn mix_gamma(mut v: i64) -> i64 {
    // force v to be odd
    v = rrxmrrxmsx(v) | 1i64;
    // try to support enough 01 and 10 transitions
    let n = (v ^ (v as u64 >> 1) as i64).count_ones();
    if n < 24 {
        v ^ 0xaaaaaaaaaaaaaaaau64 as i64
    } else {
        v
    }
}

impl Default for SplitMix64 {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
