// Copyright (c) 2017-2021 Fabian Schuiki

//! Time values
//!
//! This module implements time arithmetic.

use crate::ty::{time_ty, Type};
use num::{traits::*, BigInt, BigRational};
use std::fmt::{Debug, Display};

/// A constant time value.
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeValue {
    /// The real time value, in seconds, as a rational number.
    pub time: BigRational,
    /// The number of delta steps.
    pub delta: usize,
    /// The number of epsilon steps.
    pub epsilon: usize,
}

impl TimeValue {
    /// Create a new time.
    pub fn new(time: BigRational, delta: usize, epsilon: usize) -> Self {
        TimeValue {
            time,
            delta,
            epsilon,
        }
    }

    /// Create the zero time.
    pub fn zero() -> Self {
        TimeValue {
            time: BigRational::zero(),
            delta: 0,
            epsilon: 0,
        }
    }

    /// Get the type of the value.
    pub fn ty(&self) -> Type {
        time_ty()
    }

    /// Get the physical time of the time.
    pub fn time(&self) -> &BigRational {
        &self.time
    }

    /// Get the delta time of the time.
    pub fn delta(&self) -> usize {
        self.delta
    }

    /// Get the epsilon time of the time.
    pub fn epsilon(&self) -> usize {
        self.epsilon
    }

    /// Check whether all components of this time are zero.
    pub fn is_zero(&self) -> bool {
        self.time.is_zero() && self.delta.is_zero() && self.epsilon.is_zero()
    }
}

impl Display for TimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write_ratio_as_si(&self.time, f)?;
        if !self.delta.is_zero() {
            write!(f, " {}d", self.delta)?;
        }
        if !self.epsilon.is_zero() {
            write!(f, " {}e", self.epsilon)?;
        }
        Ok(())
    }
}

impl Debug for TimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

fn write_ratio_as_si(ratio: &BigRational, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    if ratio.is_zero() {
        return write!(f, "0s");
    }
    let prefices = ["", "m", "u", "n", "p", "f", "a"];
    let mut scaled = ratio.clone();
    let mut prefix = 0;
    let mut shift = 0;
    while prefix + 1 < prefices.len() && shift < 9 {
        if scaled >= One::one() {
            if scaled.is_integer() {
                break;
            } else {
                shift += 3;
            }
        } else {
            prefix += 1;
        }
        scaled = scaled * BigRational::from_integer(BigInt::from(1000));
    }
    let rounded = format!("{}", scaled.round());
    if shift > 0 {
        write!(
            f,
            "{}.{}{}s",
            &rounded[..rounded.len() - shift],
            &rounded[rounded.len() - shift..],
            prefices[prefix]
        )?;
    } else {
        write!(f, "{}{}s", rounded, prefices[prefix])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_formatting() {
        let make = |num: usize, denom: usize, delta: usize, epsilon: usize| {
            format!(
                "{}",
                TimeValue::new(
                    BigRational::new(num.into(), denom.into()),
                    delta.into(),
                    epsilon.into()
                )
            )
        };
        assert_eq!(make(0, 1, 0, 0), "0s");
        assert_eq!(make(0, 1, 0, 1), "0s 1e");
        assert_eq!(make(0, 1, 1, 0), "0s 1d");
        assert_eq!(make(0, 1, 1, 1), "0s 1d 1e");

        assert_eq!(make(1, 1, 0, 0), "1s");
        assert_eq!(make(1, 1, 0, 1), "1s 1e");
        assert_eq!(make(1, 1, 1, 0), "1s 1d");
        assert_eq!(make(1, 1, 1, 1), "1s 1d 1e");

        assert_eq!(make(1, 10, 0, 0), "100ms");
        assert_eq!(make(1, 100, 0, 0), "10ms");
        assert_eq!(make(1, 1000, 0, 0), "1ms");
        assert_eq!(make(1, 10000, 0, 0), "100us");
        assert_eq!(make(1, 100000, 0, 0), "10us");
        assert_eq!(make(1, 1000000, 0, 0), "1us");
        assert_eq!(make(1, 10000000, 0, 0), "100ns");
        assert_eq!(make(1, 100000000, 0, 0), "10ns");
        assert_eq!(make(1, 1000000000, 0, 0), "1ns");
        assert_eq!(make(1, 10000000000, 0, 0), "100ps");
        assert_eq!(make(1, 100000000000, 0, 0), "10ps");
        assert_eq!(make(1, 1000000000000, 0, 0), "1ps");
        assert_eq!(make(1, 10000000000000, 0, 0), "100fs");
        assert_eq!(make(1, 100000000000000, 0, 0), "10fs");
        assert_eq!(make(1, 1000000000000000, 0, 0), "1fs");
        assert_eq!(make(1, 10000000000000000, 0, 0), "100as");
        assert_eq!(make(1, 100000000000000000, 0, 0), "10as");
        assert_eq!(make(1, 1000000000000000000, 0, 0), "1as");

        assert_eq!(make(500, 1, 0, 0), "500s");
        assert_eq!(make(50, 1, 0, 0), "50s");
        assert_eq!(make(5, 1, 0, 0), "5s");
        assert_eq!(make(5, 10, 0, 0), "500ms");
        assert_eq!(make(5, 100, 0, 0), "50ms");
        assert_eq!(make(5, 1000, 0, 0), "5ms");

        assert_eq!(make(1, 3, 0, 0), "333.333333333ms");
    }
}
