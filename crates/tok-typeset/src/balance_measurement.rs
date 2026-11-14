//! Balance measurement module for column endpoint calculation.
//!
//! Implements bottom delta calculation per specification section 8.1:
//! bottomDelta = |bottom_of_column_1 - bottom_of_column_2|
//!
//! Measures the vertical position where each column ends and computes
//! the difference between the highest and lowest ending positions.

use crate::mpt::Mpt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for balance measurement operations.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum BalanceError {
    /// All columns are empty, cannot measure balance.
    #[error("All columns are empty")]
    AllColumnsEmpty,
    
    /// Baseline step is zero or negative, cannot calculate delta in lines.
    #[error("Baseline step must be positive, got {0}")]
    InvalidBaselineStep(Mpt),
    
    /// No measurements provided.
    #[error("No measurements provided")]
    NoMeasurements,
    
    /// Invalid measurement data.
    #[error("Invalid measurement: {0}")]
    InvalidMeasurement(String),
}

/// Measurement of column ending positions for balance calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnMeasurement {
    /// Column index within the page (0-based).
    pub column_index: usize,
    
    /// Y-coordinate of the last baseline in this column (None if empty).
    pub last_baseline_y: Option<Mpt>,
    
    /// Number of lines in this column.
    pub line_count: usize,
    
    /// Whether this column is empty.
    pub is_empty: bool,
}

impl ColumnMeasurement {
    /// Create a new column measurement.
    pub fn new(column_index: usize) -> Self {
        Self {
            column_index,
            last_baseline_y: None,
            line_count: 0,
            is_empty: true,
        }
    }

    /// Set the last baseline Y coordinate.
    pub fn with_last_baseline_y(mut self, y: Mpt) -> Self {
        self.last_baseline_y = Some(y);
        self.is_empty = false;
        self
    }

    /// Set the line count.
    pub fn with_line_count(mut self, count: usize) -> Self {
        self.line_count = count;
        self
    }
}

/// Result of balance calculation between columns.
///
/// Per specification section 8.1:
/// - bottomDelta = |bottom_of_column_1 - bottom_of_column_2|
/// - Empty columns are filtered out before calculation
/// - delta_lines = delta / baseline_step
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BottomDelta {
    /// Minimum bottom position among non-empty columns.
    pub min_bottom: Mpt,
    
    /// Maximum bottom position among non-empty columns.
    pub max_bottom: Mpt,
    
    /// Absolute difference: |max_bottom - min_bottom|.
    pub delta: Mpt,
    
    /// Delta expressed in line units (for diagnostics).
    pub delta_lines: f32,
    
    /// Number of empty columns that were excluded.
    pub empty_column_count: usize,
}

impl BottomDelta {
    /// Calculate from column measurements per spec section 8.1.
    ///
    /// Steps:
    /// 1. Filter out empty columns
    /// 2. Find min and max of lastBaselineY among remaining columns
    /// 3. Calculate delta = max - min
    /// 4. Calculate delta_lines = delta / baseline_step
    ///
    /// # Errors
    /// - Returns `BalanceError::AllColumnsEmpty` if all columns are empty
    /// - Returns `BalanceError::InvalidBaselineStep` if baseline_step is <= 0
    /// - Returns `BalanceError::NoMeasurements` if measurements is empty
    pub fn from_columns(
        measurements: &[ColumnMeasurement],
        baseline_step: Mpt,
    ) -> Result<Self, BalanceError> {
        // Validate inputs
        if measurements.is_empty() {
            return Err(BalanceError::NoMeasurements);
        }

        if baseline_step.raw() <= 0 {
            return Err(BalanceError::InvalidBaselineStep(baseline_step));
        }

        // Filter non-empty columns and collect their last baseline positions
        let non_empty: Vec<Mpt> = measurements
            .iter()
            .filter(|m| !m.is_empty)
            .filter_map(|m| m.last_baseline_y)
            .collect();

        let empty_column_count = measurements.len() - non_empty.len();

        // Check if all columns are empty
        if non_empty.is_empty() {
            return Err(BalanceError::AllColumnsEmpty);
        }

        // Find min and max
        let min_bottom = *non_empty.iter().min().unwrap();
        let max_bottom = *non_empty.iter().max().unwrap();

        // Calculate delta
        let delta = Mpt::from_raw((max_bottom.raw() - min_bottom.raw()).abs());

        // Calculate delta in lines
        let delta_lines = delta.to_pt() / baseline_step.to_pt();

        Ok(Self {
            min_bottom,
            max_bottom,
            delta,
            delta_lines,
            empty_column_count,
        })
    }

    /// Check if the delta is within the given threshold.
    pub fn is_within_threshold(&self, threshold: Mpt) -> bool {
        self.delta.raw().abs() <= threshold.raw().abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_columns() {
        // Two columns ending at the same baseline
        let measurements = vec![
            ColumnMeasurement::new(0)
                .with_last_baseline_y(Mpt::from_pt(100.0))
                .with_line_count(10),
            ColumnMeasurement::new(1)
                .with_last_baseline_y(Mpt::from_pt(100.0))
                .with_line_count(10),
        ];

        let result = BottomDelta::from_columns(&measurements, Mpt::from_pt(12.0)).unwrap();

        assert_eq!(result.min_bottom, Mpt::from_pt(100.0));
        assert_eq!(result.max_bottom, Mpt::from_pt(100.0));
        assert_eq!(result.delta, Mpt::ZERO);
        assert_eq!(result.delta_lines, 0.0);
        assert_eq!(result.empty_column_count, 0);
    }

    #[test]
    fn test_offset_columns() {
        // Columns ending at different baselines
        let measurements = vec![
            ColumnMeasurement::new(0)
                .with_last_baseline_y(Mpt::from_pt(100.0))
                .with_line_count(10),
            ColumnMeasurement::new(1)
                .with_last_baseline_y(Mpt::from_pt(112.0))
                .with_line_count(11),
        ];

        let baseline_step = Mpt::from_pt(12.0);
        let result = BottomDelta::from_columns(&measurements, baseline_step).unwrap();

        assert_eq!(result.min_bottom, Mpt::from_pt(100.0));
        assert_eq!(result.max_bottom, Mpt::from_pt(112.0));
        assert_eq!(result.delta, Mpt::from_pt(12.0));
        assert_eq!(result.delta_lines, 1.0);
        assert_eq!(result.empty_column_count, 0);
    }

    #[test]
    fn test_empty_column_handling() {
        // Three columns, one is empty
        let measurements = vec![
            ColumnMeasurement::new(0)
                .with_last_baseline_y(Mpt::from_pt(100.0))
                .with_line_count(10),
            ColumnMeasurement::new(1), // Empty
            ColumnMeasurement::new(2)
                .with_last_baseline_y(Mpt::from_pt(106.0))
                .with_line_count(11),
        ];

        let baseline_step = Mpt::from_pt(12.0);
        let result = BottomDelta::from_columns(&measurements, baseline_step).unwrap();

        assert_eq!(result.min_bottom, Mpt::from_pt(100.0));
        assert_eq!(result.max_bottom, Mpt::from_pt(106.0));
        assert_eq!(result.delta, Mpt::from_pt(6.0));
        assert_eq!(result.delta_lines, 0.5);
        assert_eq!(result.empty_column_count, 1);
    }

    #[test]
    fn test_all_empty_columns() {
        // All columns are empty
        let measurements = vec![ColumnMeasurement::new(0), ColumnMeasurement::new(1)];

        let result = BottomDelta::from_columns(&measurements, Mpt::from_pt(12.0));

        assert_eq!(result, Err(BalanceError::AllColumnsEmpty));
    }

    #[test]
    fn test_single_column() {
        // Only one column
        let measurements = vec![ColumnMeasurement::new(0)
            .with_last_baseline_y(Mpt::from_pt(100.0))
            .with_line_count(10)];

        let result = BottomDelta::from_columns(&measurements, Mpt::from_pt(12.0)).unwrap();

        assert_eq!(result.min_bottom, Mpt::from_pt(100.0));
        assert_eq!(result.max_bottom, Mpt::from_pt(100.0));
        assert_eq!(result.delta, Mpt::ZERO);
        assert_eq!(result.delta_lines, 0.0);
    }

    #[test]
    fn test_three_columns() {
        // Three columns with different endpoints
        let measurements = vec![
            ColumnMeasurement::new(0)
                .with_last_baseline_y(Mpt::from_pt(100.0))
                .with_line_count(10),
            ColumnMeasurement::new(1)
                .with_last_baseline_y(Mpt::from_pt(112.0))
                .with_line_count(11),
            ColumnMeasurement::new(2)
                .with_last_baseline_y(Mpt::from_pt(106.0))
                .with_line_count(10),
        ];

        let baseline_step = Mpt::from_pt(12.0);
        let result = BottomDelta::from_columns(&measurements, baseline_step).unwrap();

        assert_eq!(result.min_bottom, Mpt::from_pt(100.0));
        assert_eq!(result.max_bottom, Mpt::from_pt(112.0));
        assert_eq!(result.delta, Mpt::from_pt(12.0));
        assert_eq!(result.delta_lines, 1.0);
        assert_eq!(result.empty_column_count, 0);
    }

    #[test]
    fn test_zero_baseline_step_error() {
        let measurements = vec![
            ColumnMeasurement::new(0)
                .with_last_baseline_y(Mpt::from_pt(100.0))
                .with_line_count(10),
        ];

        let result = BottomDelta::from_columns(&measurements, Mpt::ZERO);

        assert_eq!(
            result,
            Err(BalanceError::InvalidBaselineStep(Mpt::ZERO))
        );
    }

    #[test]
    fn test_large_delta() {
        // Columns with large difference
        let measurements = vec![
            ColumnMeasurement::new(0)
                .with_last_baseline_y(Mpt::from_pt(50.0))
                .with_line_count(5),
            ColumnMeasurement::new(1)
                .with_last_baseline_y(Mpt::from_pt(200.0))
                .with_line_count(20),
        ];

        let baseline_step = Mpt::from_pt(10.0);
        let result = BottomDelta::from_columns(&measurements, baseline_step).unwrap();

        assert_eq!(result.delta, Mpt::from_pt(150.0));
        assert_eq!(result.delta_lines, 15.0);
    }

    #[test]
    fn test_no_measurements_error() {
        let measurements: Vec<ColumnMeasurement> = vec![];

        let result = BottomDelta::from_columns(&measurements, Mpt::from_pt(12.0));

        assert_eq!(result, Err(BalanceError::NoMeasurements));
    }

    #[test]
    fn test_is_within_threshold() {
        let delta = BottomDelta {
            min_bottom: Mpt::from_pt(100.0),
            max_bottom: Mpt::from_pt(105.0),
            delta: Mpt::from_pt(5.0),
            delta_lines: 0.5,
            empty_column_count: 0,
        };

        assert!(delta.is_within_threshold(Mpt::from_pt(5.0)));
        assert!(delta.is_within_threshold(Mpt::from_pt(10.0)));
        assert!(!delta.is_within_threshold(Mpt::from_pt(4.0)));
    }
}
