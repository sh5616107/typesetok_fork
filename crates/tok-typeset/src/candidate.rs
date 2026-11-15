//! Balance candidate structures for solver evaluation.
//!
//! Represents different layout variations generated during the 5-stage
//! adjustment ladder, with associated quality metrics for scoring.

use crate::mpt::Mpt;
use serde::{Deserialize, Serialize};

/// A potential balanced layout solution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceCandidate {
    /// The column content divisions.
    pub columns: Vec<ColumnContent>,

    /// Measured bottom delta (absolute difference).
    pub bottom_delta: Mpt,

    /// Delta expressed in baseline units.
    pub delta_lines: f32,

    // Soft quality metrics
    /// Word spacing quality (0.0 = worst, 1.0 = best).
    pub word_spacing_quality: f32,

    /// Line quality score (badness-based).
    pub line_quality_score: f32,

    /// Number of widow/orphan violations.
    pub widow_orphan_violations: usize,

    /// Deviation from ideal paragraph spacing.
    pub para_spacing_deviation: f32,

    /// Cost of change from previous layout.
    pub change_cost: f32,

    // Metadata
    /// Which stage generated this candidate.
    pub stage: Stage,

    /// Strategy description (e.g., "word_spacing:0.92,looseness:-1").
    pub strategy: String,
}

/// Column content definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnContent {
    /// Column index (0-based).
    pub column_index: usize,

    /// Line indices assigned to this column.
    pub line_indices: Vec<usize>,

    /// Last baseline Y coordinate.
    pub last_baseline_y: Option<Mpt>,

    /// Number of lines in this column.
    pub line_count: usize,
}

impl ColumnContent {
    /// Create a new empty column.
    pub fn new(column_index: usize) -> Self {
        Self {
            column_index,
            line_indices: Vec::new(),
            last_baseline_y: None,
            line_count: 0,
        }
    }

    /// Check if the column is empty.
    pub fn is_empty(&self) -> bool {
        self.line_count == 0
    }
}

/// The stage of the solver ladder that generated a candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    /// Stage 1: Column division variations.
    ColumnDivision,

    /// Stage 2: Word spacing + looseness.
    WordSpacingLooseness,

    /// Stage 3: Paragraph spacing adjustment.
    ParagraphSpacing,

    /// Stage 4: Neighbor cut point transfer.
    NeighborSearch,

    /// Stage 5: Reserved for future extensions.
    Extended,
}

impl BalanceCandidate {
    /// Create a new candidate with initial values.
    pub fn new(stage: Stage, strategy: String) -> Self {
        Self {
            columns: Vec::new(),
            bottom_delta: Mpt::ZERO,
            delta_lines: 0.0,
            word_spacing_quality: 1.0,
            line_quality_score: 1.0,
            widow_orphan_violations: 0,
            para_spacing_deviation: 0.0,
            change_cost: 0.0,
            stage,
            strategy,
        }
    }

    /// Check if this candidate satisfies hard constraints.
    pub fn satisfies_hard_constraints(&self) -> bool {
        // For Phase 3, we start with basic checks
        // Future: add overflow, keep rules, manual overrides

        // No columns should be empty for BodyFull (enforced by policy)
        // Lines should be in order
        if !self.columns.is_empty() {
            for col in &self.columns {
                // Check that line indices are sorted
                for window in col.line_indices.windows(2) {
                    if window[0] >= window[1] {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Check if this is an exact balance (delta = 0).
    pub fn is_exact(&self) -> bool {
        self.bottom_delta == Mpt::ZERO
    }

    /// Calculate soft score with given weights.
    pub fn soft_score(&self, weights: &WeightConfig) -> f32 {
        // Lower is better
        weights.word_spacing * (1.0 - self.word_spacing_quality)
            + weights.line_quality * (1.0 - self.line_quality_score)
            + weights.widow_orphan * self.widow_orphan_violations as f32
            + weights.para_spacing * self.para_spacing_deviation
            + weights.change_cost * self.change_cost
    }
}

/// Weights for soft quality metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeightConfig {
    pub word_spacing: f32,
    pub line_quality: f32,
    pub widow_orphan: f32,
    pub para_spacing: f32,
    pub change_cost: f32,
}

impl Default for WeightConfig {
    fn default() -> Self {
        Self {
            word_spacing: 0.30,
            line_quality: 0.30,
            widow_orphan: 0.20,
            para_spacing: 0.15,
            change_cost: 0.05,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_creation() {
        let candidate = BalanceCandidate::new(Stage::ColumnDivision, "split:10".to_string());

        assert_eq!(candidate.stage, Stage::ColumnDivision);
        assert_eq!(candidate.strategy, "split:10");
        assert_eq!(candidate.bottom_delta, Mpt::ZERO);
        assert!(candidate.is_exact());
    }

    #[test]
    fn test_hard_constraints_empty() {
        let candidate = BalanceCandidate::new(Stage::ColumnDivision, "test".to_string());
        assert!(candidate.satisfies_hard_constraints());
    }

    #[test]
    fn test_hard_constraints_ordered() {
        let mut candidate = BalanceCandidate::new(Stage::ColumnDivision, "test".to_string());
        candidate.columns.push(ColumnContent {
            column_index: 0,
            line_indices: vec![0, 1, 2],
            last_baseline_y: Some(Mpt::from_pt(100.0)),
            line_count: 3,
        });

        assert!(candidate.satisfies_hard_constraints());
    }

    #[test]
    fn test_hard_constraints_unordered() {
        let mut candidate = BalanceCandidate::new(Stage::ColumnDivision, "test".to_string());
        candidate.columns.push(ColumnContent {
            column_index: 0,
            line_indices: vec![0, 2, 1], // Out of order
            last_baseline_y: Some(Mpt::from_pt(100.0)),
            line_count: 3,
        });

        assert!(!candidate.satisfies_hard_constraints());
    }

    #[test]
    fn test_soft_score_calculation() {
        let mut candidate = BalanceCandidate::new(Stage::ColumnDivision, "test".to_string());
        candidate.word_spacing_quality = 0.9;
        candidate.line_quality_score = 0.8;
        candidate.widow_orphan_violations = 1;

        let weights = WeightConfig::default();
        let score = candidate.soft_score(&weights);

        // (1.0 - 0.9) * 0.30 + (1.0 - 0.8) * 0.30 + 1.0 * 0.20 + 0.0 + 0.0
        // = 0.03 + 0.06 + 0.20 = 0.29
        assert!((score - 0.29).abs() < 0.001);
    }

    #[test]
    fn test_column_content() {
        let mut col = ColumnContent::new(0);
        assert!(col.is_empty());

        col.line_indices = vec![0, 1, 2];
        col.line_count = 3;
        col.last_baseline_y = Some(Mpt::from_pt(100.0));

        assert!(!col.is_empty());
        assert_eq!(col.column_index, 0);
        assert_eq!(col.line_count, 3);
    }

    #[test]
    fn test_weight_config_default() {
        let weights = WeightConfig::default();
        let sum = weights.word_spacing
            + weights.line_quality
            + weights.widow_orphan
            + weights.para_spacing
            + weights.change_cost;

        // Should sum to 1.0 for normalized scoring
        assert!((sum - 1.0).abs() < 0.001);
    }
}
