// Copyright 2022 Stefan Zobel
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[inline]
pub (crate) const fn stafford_mix13(mut v: i64) -> i64 {
    v = (v ^ (v as u64 >> 30) as i64).wrapping_mul(0xbf58476d1ce4e5b9u64 as i64);
    v = (v ^ (v as u64 >> 27) as i64).wrapping_mul(0x94d049bb133111ebu64 as i64);
    v ^ (v as u64 >> 31) as i64
}

#[inline]
pub (crate) const fn stafford_mix04(mut v: i64) -> i32 {
    v = (v ^ (v as u64 >> 33) as i64).wrapping_mul(0x62a9d9ed799705f5i64);
    (((v ^ (v as u64 >> 28) as i64).wrapping_mul(0xcb24d0a5c88c35b3u64 as i64)) as u64 >> 32) as i32
}

#[inline]
pub (crate) const fn rrxmrrxmsx(mut v: i64) -> i64 {
    v ^= ((v as u64 >> 25) as i64 | (v << 39)) ^ ((v as u64 >> 50) as i64 | (v << 14));
    v = v.wrapping_mul(0xa24baed4963ee407u64 as i64);
    v ^= ((v as u64 >> 24) as i64 | (v << 40)) ^ ((v as u64 >> 49) as i64 | (v << 15));
    v = v.wrapping_mul(0x9fb21c651e98df25u64 as i64);
    v ^ (v as u64 >> 28) as i64
}

#[inline]
pub (crate) const fn xnasam(mut v: i64) -> i64 {
    v ^= 0x6a09e667f3bcc909i64;
    v ^= ((v as u64 >> 25) as i64 | (v << 39)) ^ ((v as u64 >> 47) as i64 | (v << 17));
    v = v.wrapping_mul(0x9e6c63d0676a9a99u64 as i64);
    v ^= (v as u64 >> 23) as i64 ^ (v as u64 >> 51) as i64;
    v = v.wrapping_mul(0x9e6d62d06f6a9a9bu64 as i64);
    v ^ ((v as u64 >> 23) as i64 ^ (v as u64 >> 51) as i64)
}

#[inline]
pub (crate) const fn lea_mix64(mut v: i64) -> i64 {
    v = (v ^ (v as u64 >> 32) as i64).wrapping_mul(0xdaba0b6eb09322e3u64 as i64);
    v = (v ^ (v as u64 >> 32) as i64).wrapping_mul(0xdaba0b6eb09322e3u64 as i64);
    v ^ (v as u64 >> 32) as i64
}

#[cfg(test)]
mod bit_mix_tests {
    use super::*;

    #[test]
    fn test_stafford_mix13() {
        let l1 = -3222165538581252362i64;
        let l2 = -6575083715474529190i64;
        let l3 = 5852023251876651789i64;
        let l4 = -2105427755758183442i64;
        let l5 = 8768027636157682880i64;
        let l1 = stafford_mix13(l1);
        let l2 = stafford_mix13(l2);
        let l3 = stafford_mix13(l3);
        let l4 = stafford_mix13(l4);
        let l5 = stafford_mix13(l5);
        println!("l1: {}", l1);
        println!("l2: {}", l2);
        println!("l3: {}", l3);
        println!("l4: {}", l4);
        println!("l5: {}", l5);
        assert_eq!(l1, -364480833079461142i64);
        assert_eq!(l2, -2195553744329346567i64);
        assert_eq!(l3, 8519880896858805593i64);
        assert_eq!(l4, 6042095800684428140i64);
        assert_eq!(l5, 7558165513295548834i64);
    }

    #[test]
    fn test_stafford_mix04() {
        let l1 = -3222165538581252362i64;
        let l2 = -6575083715474529190i64;
        let l3 = 5852023251876651789i64;
        let l4 = -2105427755758183442i64;
        let l5 = 8768027636157682880i64;
        let l1 = stafford_mix04(l1);
        let l2 = stafford_mix04(l2);
        let l3 = stafford_mix04(l3);
        let l4 = stafford_mix04(l4);
        let l5 = stafford_mix04(l5);
        println!("l1: {}", l1);
        println!("l2: {}", l2);
        println!("l3: {}", l3);
        println!("l4: {}", l4);
        println!("l5: {}", l5);
        assert_eq!(l1, -1774945203i32);
        assert_eq!(l2, -1767222429i32);
        assert_eq!(l3, 1317024783i32);
        assert_eq!(l4, -1566290268i32);
        assert_eq!(l5, -1588154649i32);
    }

    #[test]
    fn test_lea_mix64() {
        let l1 = -3222165538581252362i64;
        let l2 = -6575083715474529190i64;
        let l3 = 5852023251876651789i64;
        let l4 = -2105427755758183442i64;
        let l5 = 8768027636157682880i64;
        let l1 = lea_mix64(l1);
        let l2 = lea_mix64(l2);
        let l3 = lea_mix64(l3);
        let l4 = lea_mix64(l4);
        let l5 = lea_mix64(l5);
        println!("l1: {}", l1);
        println!("l2: {}", l2);
        println!("l3: {}", l3);
        println!("l4: {}", l4);
        println!("l5: {}", l5);
        assert_eq!(l1, -2558215071332148552i64);
        assert_eq!(l2, -2639391046102820563i64);
        assert_eq!(l3, -8069027239477572864i64);
        assert_eq!(l4, -5084964469365455950i64);
        assert_eq!(l5, -691027536433306030i64);
    }

    #[test]
    fn test_rrxmrrxmsx() {
        let l1 = 8437077494049660495i64;
        let l2 = 9148403451769538754i64;
        let l3 = -5204784484909980772i64;
        let l4 = 3756336564354234498i64;
        let l5 = -4784683118053631234i64;
        let l1 = rrxmrrxmsx(l1);
        let l2 = rrxmrrxmsx(l2);
        let l3 = rrxmrrxmsx(l3);
        let l4 = rrxmrrxmsx(l4);
        let l5 = rrxmrrxmsx(l5);
        println!("l1: {}", l1);
        println!("l2: {}", l2);
        println!("l3: {}", l3);
        println!("l4: {}", l4);
        println!("l5: {}", l5);
        assert_eq!(l1, -1074278017023486638i64);
        assert_eq!(l2, 791467320547438801i64);
        assert_eq!(l3, -6237844429596446841i64);
        assert_eq!(l4, 967439720212533408i64);
        assert_eq!(l5, 4511633004553689581i64);
    }

    #[test]
    fn test_xnasam() {
        let l1 = 8437077494049660495i64;
        let l2 = 9148403451769538754i64;
        let l3 = -5204784484909980772i64;
        let l4 = 3756336564354234498i64;
        let l5 = -4784683118053631234i64;
        let l1 = xnasam(l1);
        let l2 = xnasam(l2);
        let l3 = xnasam(l3);
        let l4 = xnasam(l4);
        let l5 = xnasam(l5);
        println!("l1: {}", l1);
        println!("l2: {}", l2);
        println!("l3: {}", l3);
        println!("l4: {}", l4);
        println!("l5: {}", l5);
        assert_eq!(l1, -8711258770756958679i64);
        assert_eq!(l2, -8469334982622011693i64);
        assert_eq!(l3, 3242515187903039772i64);
        assert_eq!(l4, -8485343714015871830i64);
        assert_eq!(l5, 4496690204852887599i64);
    }
}
