//! Millipoint (mpt) integer coordinate system.
//!
//! The specification requires all coordinates to be represented as integers in millipoints
//! (1/1000 of a point) for deterministic layout computation across platforms.
//!
//! This module provides the `Mpt` newtype wrapping `i32`, with conversion traits and
//! arithmetic operations.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

/// A length in millipoints (1/1000 pt).
///
/// # Examples
///
/// ```
/// use tok_typeset::Mpt;
///
/// let width = Mpt::from_pt(12.5);
/// assert_eq!(width.to_pt(), 12.5);
/// assert_eq!(width.0, 12500);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Mpt(pub i32);

impl Mpt {
    /// Create a millipoint value from points.
    ///
    /// Rounds to the nearest integer millipoint.
    #[inline]
    pub fn from_pt(pt: f32) -> Self {
        Mpt((pt * 1000.0).round() as i32)
    }

    /// Convert millipoints to points.
    #[inline]
    pub fn to_pt(self) -> f32 {
        self.0 as f32 / 1000.0
    }

    /// Create a millipoint value from an integer.
    #[inline]
    pub const fn from_raw(mpt: i32) -> Self {
        Mpt(mpt)
    }

    /// Get the raw millipoint value.
    #[inline]
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Check if the value is finite (always true for i32, provided for API consistency).
    #[inline]
    pub const fn is_finite(self) -> bool {
        true
    }

    /// Check if the value is within a reasonable range for typesetting.
    ///
    /// Returns true if the absolute value is less than 1,000,000 pt (1 billion mpt).
    /// This is approximately 352 meters, far beyond any practical page dimension.
    #[inline]
    pub fn in_range(self) -> bool {
        self.0.abs() < 1_000_000_000
    }

    /// Zero millipoints.
    pub const ZERO: Mpt = Mpt(0);
}

impl From<f32> for Mpt {
    fn from(pt: f32) -> Self {
        Mpt::from_pt(pt)
    }
}

impl From<Mpt> for f32 {
    fn from(mpt: Mpt) -> Self {
        mpt.to_pt()
    }
}

impl Add for Mpt {
    type Output = Mpt;

    fn add(self, rhs: Mpt) -> Mpt {
        Mpt(self.0 + rhs.0)
    }
}

impl Sub for Mpt {
    type Output = Mpt;

    fn sub(self, rhs: Mpt) -> Mpt {
        Mpt(self.0 - rhs.0)
    }
}

impl Mul<i32> for Mpt {
    type Output = Mpt;

    fn mul(self, rhs: i32) -> Mpt {
        Mpt(self.0 * rhs)
    }
}

impl Div<i32> for Mpt {
    type Output = Mpt;

    fn div(self, rhs: i32) -> Mpt {
        Mpt(self.0 / rhs)
    }
}

impl std::fmt::Display for Mpt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}pt", self.to_pt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpt_conversion() {
        let mpt = Mpt::from_pt(12.5);
        assert_eq!(mpt.0, 12500);
        assert_eq!(mpt.to_pt(), 12.5);
    }

    #[test]
    fn test_mpt_arithmetic() {
        let a = Mpt::from_pt(10.0);
        let b = Mpt::from_pt(5.0);

        assert_eq!((a + b).to_pt(), 15.0);
        assert_eq!((a - b).to_pt(), 5.0);
        assert_eq!((a * 2).to_pt(), 20.0);
        assert_eq!((a / 2).to_pt(), 5.0);
    }

    #[test]
    fn test_mpt_range() {
        assert!(Mpt::from_pt(100.0).in_range());
        assert!(Mpt::from_pt(10000.0).in_range());
        assert!(Mpt::from_raw(999_999_999).in_range());
        assert!(!Mpt::from_raw(1_000_000_000).in_range());
    }

    #[test]
    fn test_mpt_ordering() {
        let a = Mpt::from_pt(5.0);
        let b = Mpt::from_pt(10.0);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Mpt::from_pt(5.0));
    }

    #[test]
    fn test_mpt_display() {
        let mpt = Mpt::from_pt(12.5);
        assert_eq!(format!("{}", mpt), "12.5pt");
    }

    #[test]
    fn test_mpt_serde() {
        let mpt = Mpt::from_pt(42.25);
        let json = serde_json::to_string(&mpt).unwrap();
        assert_eq!(json, "42250");

        let deserialized: Mpt = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, mpt);
    }
}
