use crate::constants::{INF_BAD, TWO};
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord, Integer, Scaled
};

use std::cmp::Ordering::{Equal, Greater, Less};

// Part 7: Arithmetic with scaled dimensions

#[macro_export]
macro_rules! odd {
    ($a:expr) => {
        $a & 1 == 1
    };
}

// Section 100
#[macro_export]
macro_rules! half {
    ($x:expr) => {
        if odd!($x) {
            ($x + 1) / 2
        }
        else {
            $x / 2
        }
    };
}

impl Global {
    // Section 102
    pub(crate) fn round_decimals(&self, mut k: usize) -> Scaled {
        let mut a = 0;
        while k > 0 {
            k -= 1;
            a = (a + (self.dig[k] as Integer)*TWO) / 10;
        }
        (a + 1) / 2
    }
}

// Section 105
#[macro_export]
macro_rules! nx_plus_y {
    ($($args:expr),*) => {
        mult_and_add($($args),*, 0x3fff_ffff)?
    };
}

#[macro_export]
macro_rules! mult_integers {
    ($($args:expr),*) => {
        mult_and_add($($args),*, 0, 0x7fff_ffff)?
    };
}

pub(crate) fn mult_and_add(mut n: Integer, mut x: Scaled, y: Scaled, max_answer: Scaled) -> TeXResult<Scaled> {
    if n < 0 {
        x = -x;
        n = -n;
    }
    match n {
        0 => Ok(y),
        _ => {
            if (x <= (max_answer - y) / n) && (-x <= (max_answer + y) / n) {
                Ok(x*n + y)
            }
            else {
                Err(TeXError::Arith)
            }
        }
    }
}

// Section 106
// Remainder is not a global variable in this implementation.
pub(crate) fn x_over_n(mut x: Scaled, mut n: Integer) -> TeXResult<(Scaled, Scaled)> {
    let mut negative = false;
    if n == 0 {
        return Err(TeXError::Arith);
    }
    if n < 0 {
        x = -x;
        n = -n;
        negative = true
    }
    let (quo, mut rem) = match x.cmp(&0) {
        Greater | Equal => (x / n, x % n),
        Less => (-((-x) / n), -((-x) % n))
    };

    if negative {
        rem = -rem;
    }

    Ok((quo, rem))
}

// Section 107
pub(crate) fn xn_over_d(mut x: Scaled, n: Integer, d: Integer) -> TeXResult<(Scaled, Scaled)> {
    let positive = match x.cmp(&0) {
        Greater | Equal => true,
        Less => {
            x = -x;
            false
        }
    };

    let t = (x % 32768) * n;
    let mut u = (x / 32768) * n + (t / 32768);
    let v = (u % d) * 32768 + (t % 32768);
    if u / d >= 32768 {
        return Err(TeXError::Arith);
    }
    u = 32768 * (u / d) + (v / d);
    match positive {
        true => Ok((u, v % d)),
        false => Ok((-u, -(v % d)))
    }
}

// Section 108
pub(crate) fn badness(t: Scaled, s: Scaled) -> HalfWord {
    if t == 0 {
        0
    }
    else if s <= 0 {
        INF_BAD
    }
    else {
        let r = if t <= 7_230_584 {
            (t * 297) / s
        }
        else if s >= 1_663_497 {
            t / (s / 297)
        }
        else {
            t
        };

        if r > 1290 {
            INF_BAD
        }
        else {
            (r*r*r + 0x20000) / 0x40000
        }
    }
}
