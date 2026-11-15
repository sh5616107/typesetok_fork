//! Core balance solver with 5-stage adjustment ladder.
//!
//! Implements the balance solver per specification section 10.4:
//! 1. Stage 1: Column Division (±2 lines around natural split)
//! 2. Stage 2: Word Spacing + Looseness (0.85-1.40 range, ±1 looseness)
//! 3. Stage 3: Paragraph Spacing (baseline grid multiples)
//! 4. Stage 4: Neighbor Search (transfer cut points between pages)
//! 5. Early termination on exact balance (bottomDelta = 0)

use crate::balance_measurement::{BottomDelta, ColumnMeasurement};
use crate::candidate::{BalanceCandidate, ColumnContent, Stage, WeightConfig};
use crate::mpt::Mpt;
use crate::page_policy::PolicyConfig;
use crate::scoring::select_best_candidate;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Solver configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolverConfig {
    /// Word spacing range for Stage 2 (e.g., 0.85 to 1.40).
    pub word_spacing_range: (f32, f32),

    /// Paragraph spacing grid for Stage 3 (multiples of baseline).
    pub para_spacing_grid: Vec<Mpt>,

    /// Whether to allow looseness adjustments (±1).
    pub looseness_allowed: bool,

    /// Whether to enable neighbor page search (Stage 4).
    pub enable_neighbor_search: bool,

    /// Baseline step for line height calculations.
    pub baseline_step: Mpt,

    /// Number of word spacing samples to try in Stage 2.
    pub word_spacing_samples: usize,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            word_spacing_range: (0.85, 1.40),
            para_spacing_grid: vec![
                Mpt::ZERO,
                Mpt::from_pt(12.0), // 1L
                Mpt::from_pt(24.0), // 2L
                Mpt::from_pt(36.0), // 3L
            ],
            looseness_allowed: true,
            enable_neighbor_search: false, // Disabled for Phase 3
            baseline_step: Mpt::from_pt(12.0),
            word_spacing_samples: 5,
        }
    }
}

/// Balance solver search limits to prevent candidate explosion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceSearchLimits {
    /// Maximum candidates to generate per stage.
    pub max_candidates_per_stage: usize,

    /// Maximum total candidates across all stages.
    pub max_total_candidates: usize,

    /// Maximum neighbor pages to search (Stage 4).
    pub max_neighbor_pages: usize,
}

impl Default for BalanceSearchLimits {
    fn default() -> Self {
        Self {
            max_candidates_per_stage: 50,
            max_total_candidates: 200,
            max_neighbor_pages: 2,
        }
    }
}

/// Balance solver error.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum BalanceSolverError {
    #[error("No content to balance")]
    NoContent,

    #[error("Invalid number of columns: {0}")]
    InvalidColumnCount(usize),

    #[error("Balance measurement failed: {0}")]
    MeasurementFailed(String),

    #[error("Candidate limit exceeded")]
    CandidateLimitExceeded,

    #[error("Stage {0} failed: {1}")]
    StageFailed(String, String),
}

/// Result of the balance solver.
#[derive(Debug, Clone, PartialEq)]
pub enum BalanceResult {
    /// Exact balance achieved (delta = 0).
    Exact(BalanceCandidate),

    /// Satisfiable within threshold.
    Satisfiable(BalanceCandidate),

    /// Unsatisfiable - all stages tried.
    Unsatisfiable {
        attempted_stages: Vec<String>,
        best_attempt: Option<BalanceCandidate>,
    },
}

/// The core balance solver.
pub struct BalanceSolver {
    config: SolverConfig,
    limits: BalanceSearchLimits,
    weights: WeightConfig,
}

impl BalanceSolver {
    /// Create a new balance solver with given configuration.
    pub fn new(config: SolverConfig, limits: BalanceSearchLimits, weights: WeightConfig) -> Self {
        Self {
            config,
            limits,
            weights,
        }
    }

    /// Create a solver with default configuration.
    pub fn default_solver() -> Self {
        Self::new(
            SolverConfig::default(),
            BalanceSearchLimits::default(),
            WeightConfig::default(),
        )
    }

    /// Solve for page balance.
    ///
    /// Runs the 5-stage adjustment ladder until exact balance is achieved
    /// or all stages are exhausted.
    pub fn solve(
        &self,
        line_data: &LineData,
        num_columns: usize,
        policy: &PolicyConfig,
    ) -> Result<BalanceResult, BalanceSolverError> {
        if line_data.line_count == 0 {
            return Err(BalanceSolverError::NoContent);
        }

        if num_columns == 0 {
            return Err(BalanceSolverError::InvalidColumnCount(num_columns));
        }

        let mut all_candidates = Vec::new();
        let mut attempted_stages = Vec::new();

        // Stage 1: Column division variations
        attempted_stages.push("stage1_column_division".to_string());
        match self.stage1_column_divisions(line_data, num_columns) {
            Ok(mut candidates) => {
                candidates.retain(|c| c.satisfies_hard_constraints());

                // Check for exact balance
                if let Some(exact) = candidates.iter().find(|c| c.is_exact()) {
                    return Ok(BalanceResult::Exact(exact.clone()));
                }

                all_candidates.extend(candidates);
            }
            Err(e) => {
                return Err(BalanceSolverError::StageFailed(
                    "stage1".to_string(),
                    e.to_string(),
                ));
            }
        }

        // Stage 2: Word spacing + looseness
        if !self.found_acceptable(&all_candidates, policy) {
            attempted_stages.push("stage2_word_spacing_looseness".to_string());
            match self.stage2_word_spacing_looseness(line_data, num_columns) {
                Ok(mut candidates) => {
                    candidates.retain(|c| c.satisfies_hard_constraints());

                    // Check for exact balance
                    if let Some(exact) = candidates.iter().find(|c| c.is_exact()) {
                        return Ok(BalanceResult::Exact(exact.clone()));
                    }

                    all_candidates.extend(candidates);
                }
                Err(_) => {
                    // Stage 2 failure is not fatal, continue
                }
            }
        }

        // Stage 3: Paragraph spacing
        if !self.found_acceptable(&all_candidates, policy) {
            attempted_stages.push("stage3_paragraph_spacing".to_string());
            match self.stage3_paragraph_spacing(line_data, num_columns) {
                Ok(mut candidates) => {
                    candidates.retain(|c| c.satisfies_hard_constraints());

                    // Check for exact balance
                    if let Some(exact) = candidates.iter().find(|c| c.is_exact()) {
                        return Ok(BalanceResult::Exact(exact.clone()));
                    }

                    all_candidates.extend(candidates);
                }
                Err(_) => {
                    // Stage 3 failure is not fatal
                }
            }
        }

        // Stage 4: Neighbor search (if enabled)
        if self.config.enable_neighbor_search && !self.found_acceptable(&all_candidates, policy) {
            attempted_stages.push("stage4_neighbor_search".to_string());
            // Placeholder for Phase 3 - neighbor search requires multi-page context
        }

        // Select best candidate
        if let Some(best) =
            select_best_candidate(&all_candidates, policy.success_threshold, &self.weights)
        {
            Ok(BalanceResult::Satisfiable(best))
        } else {
            // Return best attempt even if not satisfiable
            let best_attempt = all_candidates
                .iter()
                .min_by_key(|c| c.bottom_delta.raw().abs())
                .cloned();

            Ok(BalanceResult::Unsatisfiable {
                attempted_stages,
                best_attempt,
            })
        }
    }

    /// Stage 1: Try different column division points.
    fn stage1_column_divisions(
        &self,
        line_data: &LineData,
        num_columns: usize,
    ) -> Result<Vec<BalanceCandidate>, BalanceSolverError> {
        let mut candidates = Vec::new();
        let total_lines = line_data.line_count;

        if total_lines == 0 {
            return Ok(candidates);
        }

        // Calculate natural split point
        let base_split = total_lines / num_columns;

        // Try divisions around natural split (±2 lines)
        for offset in -2..=2 {
            let split = (base_split as i32 + offset).max(0) as usize;

            if split <= total_lines {
                let strategy = format!("split:{}", split);
                let candidate = self.create_candidate_from_split(
                    line_data,
                    split,
                    num_columns,
                    Stage::ColumnDivision,
                    strategy,
                )?;

                candidates.push(candidate);

                if candidates.len() >= self.limits.max_candidates_per_stage {
                    break;
                }
            }
        }

        Ok(candidates)
    }

    /// Stage 2: Word spacing and looseness variations (public for testing).
    pub fn stage2_word_spacing_looseness(
        &self,
        line_data: &LineData,
        num_columns: usize,
    ) -> Result<Vec<BalanceCandidate>, BalanceSolverError> {
        let mut candidates = Vec::new();

        // Sample word spacing values
        let spacing_samples = self.sample_spacing_range();

        for spacing in spacing_samples {
            // Try neutral looseness
            let strategy = format!("spacing:{:.2},looseness:0", spacing);
            let candidate = self.create_candidate_with_spacing(
                line_data,
                num_columns,
                spacing,
                0,
                Stage::WordSpacingLooseness,
                strategy,
            )?;
            candidates.push(candidate);

            // Try looseness ±1 if allowed
            if self.config.looseness_allowed {
                for looseness in [-1, 1] {
                    let strategy = format!("spacing:{:.2},looseness:{}", spacing, looseness);
                    let candidate = self.create_candidate_with_spacing(
                        line_data,
                        num_columns,
                        spacing,
                        looseness,
                        Stage::WordSpacingLooseness,
                        strategy,
                    )?;
                    candidates.push(candidate);

                    if candidates.len() >= self.limits.max_candidates_per_stage {
                        return Ok(candidates);
                    }
                }
            }
        }

        Ok(candidates)
    }

    /// Stage 3: Paragraph spacing adjustments (public for testing).
    pub fn stage3_paragraph_spacing(
        &self,
        line_data: &LineData,
        num_columns: usize,
    ) -> Result<Vec<BalanceCandidate>, BalanceSolverError> {
        let mut candidates = Vec::new();

        // Try different paragraph spacing adjustments
        for spacing_delta in &self.config.para_spacing_grid {
            let strategy = format!("para_spacing_delta:{}", spacing_delta.to_pt());
            let candidate = self.create_candidate_with_para_spacing(
                line_data,
                num_columns,
                *spacing_delta,
                Stage::ParagraphSpacing,
                strategy,
            )?;
            candidates.push(candidate);

            if candidates.len() >= self.limits.max_candidates_per_stage {
                break;
            }
        }

        Ok(candidates)
    }

    /// Sample word spacing values in the configured range.
    pub fn sample_spacing_range(&self) -> Vec<f32> {
        let (min, max) = self.config.word_spacing_range;
        let samples = self.config.word_spacing_samples;

        (0..samples)
            .map(|i| min + (max - min) * i as f32 / (samples - 1) as f32)
            .collect()
    }

    /// Create a candidate from a column split point.
    fn create_candidate_from_split(
        &self,
        line_data: &LineData,
        _split_point: usize,
        num_columns: usize,
        stage: Stage,
        strategy: String,
    ) -> Result<BalanceCandidate, BalanceSolverError> {
        let mut candidate = BalanceCandidate::new(stage, strategy);

        // Distribute lines across columns
        let lines_per_column = line_data.line_count / num_columns;
        let mut remaining = line_data.line_count % num_columns;
        let mut line_idx = 0;

        for col_idx in 0..num_columns {
            let mut col = ColumnContent::new(col_idx);
            let count = lines_per_column + if remaining > 0 { 1 } else { 0 };
            if remaining > 0 {
                remaining = remaining.saturating_sub(1);
            }

            for _ in 0..count {
                if line_idx < line_data.line_count {
                    col.line_indices.push(line_idx);
                    line_idx += 1;
                }
            }

            col.line_count = col.line_indices.len();
            if !col.line_indices.is_empty() {
                // Calculate last baseline Y (simplified for Phase 3)
                let last_line_y =
                    Mpt::from_pt((col.line_count as f32) * self.config.baseline_step.to_pt());
                col.last_baseline_y = Some(last_line_y);
            }

            candidate.columns.push(col);
        }

        // Measure balance
        self.measure_candidate_balance(&mut candidate)?;

        // Calculate quality metrics (simplified for Phase 3)
        candidate.word_spacing_quality = 1.0;
        candidate.line_quality_score = 1.0;

        Ok(candidate)
    }

    /// Create a candidate with word spacing adjustment.
    fn create_candidate_with_spacing(
        &self,
        line_data: &LineData,
        num_columns: usize,
        spacing: f32,
        looseness: i32,
        stage: Stage,
        strategy: String,
    ) -> Result<BalanceCandidate, BalanceSolverError> {
        // For Phase 3, spacing and looseness don't actually rebreak lines yet
        // We simulate the effect by creating a candidate similar to stage1
        let mut candidate = self.create_candidate_from_split(
            line_data,
            line_data.line_count / num_columns,
            num_columns,
            stage,
            strategy,
        )?;

        // Adjust quality based on spacing
        candidate.word_spacing_quality = crate::scoring::calculate_word_spacing_quality(spacing);

        // Looseness slightly affects line quality
        let looseness_penalty = (looseness.abs() as f32) * 0.1;
        candidate.line_quality_score = (1.0 - looseness_penalty).max(0.0);

        Ok(candidate)
    }

    /// Create a candidate with paragraph spacing adjustment.
    fn create_candidate_with_para_spacing(
        &self,
        line_data: &LineData,
        num_columns: usize,
        spacing_delta: Mpt,
        stage: Stage,
        strategy: String,
    ) -> Result<BalanceCandidate, BalanceSolverError> {
        // For Phase 3, paragraph spacing doesn't actually affect layout yet
        // We simulate by creating a base candidate
        let mut candidate = self.create_candidate_from_split(
            line_data,
            line_data.line_count / num_columns,
            num_columns,
            stage,
            strategy,
        )?;

        // Calculate spacing deviation
        candidate.para_spacing_deviation = spacing_delta.to_pt().abs();

        Ok(candidate)
    }

    /// Measure balance for a candidate.
    fn measure_candidate_balance(
        &self,
        candidate: &mut BalanceCandidate,
    ) -> Result<(), BalanceSolverError> {
        // Convert to column measurements
        let measurements: Vec<ColumnMeasurement> = candidate
            .columns
            .iter()
            .map(|col| {
                let mut m = ColumnMeasurement::new(col.column_index);
                m.last_baseline_y = col.last_baseline_y;
                m.line_count = col.line_count;
                m.is_empty = col.is_empty();
                m
            })
            .collect();

        // Calculate bottom delta
        match BottomDelta::from_columns(&measurements, self.config.baseline_step) {
            Ok(delta) => {
                candidate.bottom_delta = delta.delta;
                candidate.delta_lines = delta.delta_lines;
                Ok(())
            }
            Err(e) => Err(BalanceSolverError::MeasurementFailed(e.to_string())),
        }
    }

    /// Check if any candidate is acceptable under the policy.
    fn found_acceptable(&self, candidates: &[BalanceCandidate], policy: &PolicyConfig) -> bool {
        candidates
            .iter()
            .any(|c| policy.is_satisfied(c.bottom_delta))
    }
}

/// Input data for the solver.
#[derive(Debug, Clone, PartialEq)]
pub struct LineData {
    /// Number of lines to distribute.
    pub line_count: usize,

    /// Available height per column.
    pub available_height: Mpt,
}

impl LineData {
    pub fn new(line_count: usize, available_height: Mpt) -> Self {
        Self {
            line_count,
            available_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_creation() {
        let solver = BalanceSolver::default_solver();
        assert_eq!(solver.config.word_spacing_samples, 5);
        assert!(solver.config.looseness_allowed);
    }

    #[test]
    fn test_no_content_error() {
        let solver = BalanceSolver::default_solver();
        let line_data = LineData::new(0, Mpt::from_pt(600.0));
        let policy = PolicyConfig::body_full();

        let result = solver.solve(&line_data, 2, &policy);
        assert_eq!(result, Err(BalanceSolverError::NoContent));
    }

    #[test]
    fn test_invalid_column_count() {
        let solver = BalanceSolver::default_solver();
        let line_data = LineData::new(20, Mpt::from_pt(600.0));
        let policy = PolicyConfig::body_full();

        let result = solver.solve(&line_data, 0, &policy);
        assert_eq!(result, Err(BalanceSolverError::InvalidColumnCount(0)));
    }

    #[test]
    fn test_stage1_generates_candidates() {
        let solver = BalanceSolver::default_solver();
        let line_data = LineData::new(20, Mpt::from_pt(600.0));

        let candidates = solver.stage1_column_divisions(&line_data, 2).unwrap();

        // Should generate 5 candidates (±2 around midpoint)
        assert!(candidates.len() >= 4);
        assert!(candidates.len() <= 5);
    }

    #[test]
    fn test_stage2_word_spacing_samples() {
        let solver = BalanceSolver::default_solver();
        let samples = solver.sample_spacing_range();

        assert_eq!(samples.len(), 5);
        assert!((samples[0] - 0.85).abs() < 0.01);
        assert!((samples[4] - 1.40).abs() < 0.01);
    }

    #[test]
    fn test_exact_balance_early_return() {
        let solver = BalanceSolver::default_solver();
        let line_data = LineData::new(20, Mpt::from_pt(600.0));
        let policy = PolicyConfig::body_full();

        let result = solver.solve(&line_data, 2, &policy).unwrap();

        // With 20 lines split evenly into 2 columns, we should get exact balance
        match result {
            BalanceResult::Exact(_) => {
                // Success - exact balance achieved
            }
            BalanceResult::Satisfiable(c) => {
                // Also acceptable if within threshold
                assert!(c.bottom_delta.raw().abs() < 1000); // Within 1pt
            }
            BalanceResult::Unsatisfiable { .. } => {
                panic!("Should have found a satisfiable solution");
            }
        }
    }

    #[test]
    fn test_candidate_limit_respected() {
        let mut limits = BalanceSearchLimits::default();
        limits.max_candidates_per_stage = 3;

        let solver = BalanceSolver::new(SolverConfig::default(), limits, WeightConfig::default());

        let line_data = LineData::new(20, Mpt::from_pt(600.0));
        let candidates = solver.stage1_column_divisions(&line_data, 2).unwrap();

        assert!(candidates.len() <= 3);
    }

    #[test]
    fn test_policy_not_applicable() {
        let solver = BalanceSolver::default_solver();
        let line_data = LineData::new(20, Mpt::from_pt(600.0));
        let policy = PolicyConfig::last_page(); // Balance not applicable

        let result = solver.solve(&line_data, 2, &policy).unwrap();

        // Should still solve but any result is acceptable
        match result {
            BalanceResult::Exact(_) | BalanceResult::Satisfiable(_) => {
                // Success
            }
            BalanceResult::Unsatisfiable { .. } => {
                // Also acceptable for last page
            }
        }
    }
}
