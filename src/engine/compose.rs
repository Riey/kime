// Copyright 2012-2015 The Rust Project Developers.
// Copyright 2017 The UNIC Project Developers.
//
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Conjoining Jamo composition to/decomposition from Hangul syllables.
//!
//! Reference: Section 3.12 Conjoining Jamo Behavior, Unicode 9.0.0
//! <https://www.unicode.org/versions/Unicode9.0.0/ch03.pdf>

use core::char;

pub const S_BASE: u32 = 0xAC00;
pub const L_BASE: u32 = 0x1100;
pub const V_BASE: u32 = 0x1161;
pub const T_BASE: u32 = 0x11A7;
pub const L_COUNT: u32 = 19;
pub const V_COUNT: u32 = 21;
pub const T_COUNT: u32 = 28;
pub const N_COUNT: u32 = V_COUNT * T_COUNT;
pub const S_COUNT: u32 = L_COUNT * N_COUNT;

/// Whether the character is a (precomposed) Hangul Syllable
pub fn is_syllable(ch: char) -> bool {
    let cp = ch as u32;
    cp >= S_BASE && cp < (S_BASE + S_COUNT)
}

/// Decompose a precomposed Hangul syllable
// FIXME: This is a workaround, we should use `F` instead of `&mut F`
#[allow(unsafe_code)]
#[inline]
pub fn decompose_syllable(syllable: char) -> (char, char, char) {
    let si = syllable as u32 - S_BASE;

    let li = si / N_COUNT;
    unsafe {
        let cho = char::from_u32_unchecked(L_BASE + li);

        let vi = (si % N_COUNT) / T_COUNT;
        let jung = char::from_u32_unchecked(V_BASE + vi);

        let ti = si % T_COUNT;

        let jong = if ti > 0 {
            char::from_u32_unchecked(T_BASE + ti)
        } else {
            '\0'
        };

        (cho, jung, jong)
    }
}

/// Compose a pair of Hangul Jamo
#[allow(unsafe_code)]
#[inline]
pub fn compose_syllable(jamo1: char, jamo2: char) -> Option<char> {
    let l = jamo1 as u32;
    let v = jamo2 as u32;
    // Compose an LPart and a VPart
    if L_BASE <= l && l < (L_BASE + L_COUNT) // l should be an L choseong jamo
        && V_BASE <= v && v < (V_BASE + V_COUNT)
    {
        // v should be a V jungseong jamo
        let r = S_BASE + (l - L_BASE) * N_COUNT + (v - V_BASE) * T_COUNT;
        return unsafe { Some(char::from_u32_unchecked(r)) };
    }
    // Compose an LVPart and a TPart
    if S_BASE <= l && l <= (S_BASE+S_COUNT-T_COUNT) // l should be a syllable block
        && T_BASE <= v && v < (T_BASE+T_COUNT) // v should be a T jongseong jamo
        && (l - S_BASE) % T_COUNT == 0
    {
        // l should be an LV syllable block (not LVT)
        let r = l + (v - T_BASE);
        return unsafe { Some(char::from_u32_unchecked(r)) };
    }
    None
}
