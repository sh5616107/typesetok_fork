//! Snake Column Breaker for continuous flow column layout (Phase 4).
//!
//! Implements magazine-style column breaking where a single content stream
//! flows seamlessly across multiple columns in reading order. Supports:
//! - Sequential distribution: fill columns in order
//! - Balanced distribution: equal column heights using Phase 3 balance_solver
//! - Dynamic column count changes at section boundaries
//! - RTL and LTR reading orders

use crate::balance_solver::{BalanceResult, BalanceSolver, LineData};
use crate::geometry::LineBox;
use crate::mpt::Mpt;
use crate::page_policy::PolicyConfig;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Distribution mode for snake columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DistributionMode {
    /// Fill columns sequentially in reading order.
    Sequential,
    /// Balance column heights equally using the balance solver.
    #[default]
    Balanced,
}

/// Configuration for snake column breaking.
#[derive(Debug, Clone, PartialEq)]
pub struct SnakeColumnConfig {
    /// Number of columns to create.
    pub column_count: usize,
    
    /// Available height per column (in Mpt).
    pub available_height: Mpt,
    
    /// Available width per column (in pt).
    pub column_width: f32,
    
    /// Distribution mode.
    pub distribution_mode: DistributionMode,
    
    /// Baseline step for line height calculations.
    pub baseline_step: Mpt,
}

impl SnakeColumnConfig {
    /// Create a new configuration.
    pub fn new(column_count: usize, available_height: Mpt, column_width: f32) -> Self {
        Self {
            column_count,
            available_height,
            column_width,
            distribution_mode: DistributionMode::Balanced,
            baseline_step: Mpt::from_pt(12.0),
        }
    }
    
    /// Set distribution mode.
    pub fn with_distribution_mode(mut self, mode: DistributionMode) -> Self {
        self.distribution_mode = mode;
        self
    }
    
    /// Set baseline step.
    pub fn with_baseline_step(mut self, step: Mpt) -> Self {
        self.baseline_step = step;
        self
    }
}

/// Result of snake column breaking.
#[derive(Debug, Clone, PartialEq)]
pub struct SnakeColumnResult {
    /// Lines distributed across columns.
    /// Each inner Vec represents lines in one column.
    pub columns: Vec<Vec<LineBox>>,
    
    /// Column allocation details.
    pub column_heights: Vec<Mpt>,
    
    /// Whether balance was achieved (for Balanced mode).
    pub is_balanced: bool,
    
    /// Bottom delta for balanced mode (0 for Sequential).
    pub bottom_delta: Mpt,
}

/// Error from snake column breaking.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum SnakeColumnError {
    #[error("No content to distribute")]
    NoContent,
    
    #[error("Invalid column count: {0}")]
    InvalidColumnCount(usize),
    
    #[error("Balance solver failed: {0}")]
    BalanceSolverFailed(String),
    
    #[error("Insufficient height: available {available}, required {required}")]
    InsufficientHeight { available: Mpt, required: Mpt },
}

/// Snake column breaker for continuous flow layouts.
pub struct SnakeColumnBreaker {
    config: SnakeColumnConfig,
    solver: Option<BalanceSolver>,
    policy: PolicyConfig,
}

impl SnakeColumnBreaker {
    /// Create a new snake column breaker.
    pub fn new(config: SnakeColumnConfig) -> Self {
        Self {
            config,
            solver: None,
            policy: PolicyConfig::body_full(),
        }
    }
    
    /// Create a breaker with custom balance solver and policy.
    pub fn with_solver(mut self, solver: BalanceSolver, policy: PolicyConfig) -> Self {
        self.solver = Some(solver);
        self.policy = policy;
        self
    }
    
    /// Break content into snake columns.
    pub fn break_into_columns(&self, lines: Vec<LineBox>) -> Result<SnakeColumnResult, SnakeColumnError> {
        if lines.is_empty() {
            return Err(SnakeColumnError::NoContent);
        }
        
        if self.config.column_count == 0 {
            return Err(SnakeColumnError::InvalidColumnCount(0));
        }
        
        match self.config.distribution_mode {
            DistributionMode::Sequential => self.break_sequential(lines),
            DistributionMode::Balanced => self.break_balanced(lines),
        }
    }
    
    /// Sequential distribution: fill columns in reading order.
    fn break_sequential(&self, lines: Vec<LineBox>) -> Result<SnakeColumnResult, SnakeColumnError> {
        let num_cols = self.config.column_count;
        let lines_per_col = lines.len().div_ceil(num_cols);
        
        let mut columns = Vec::new();
        let mut column_heights = Vec::new();
        
        for col_idx in 0..num_cols {
            let start = col_idx * lines_per_col;
            let end = ((col_idx + 1) * lines_per_col).min(lines.len());
            
            let col_lines: Vec<LineBox> = if start < lines.len() {
                lines[start..end].to_vec()
            } else {
                Vec::new()
            };
            
            // Calculate column height
            let height = if col_lines.is_empty() {
                Mpt::ZERO
            } else {
                Mpt::from_pt(col_lines.iter().map(|l| l.height).sum::<f32>())
            };
            
            column_heights.push(height);
            columns.push(col_lines);
        }
        
        Ok(SnakeColumnResult {
            columns,
            column_heights,
            is_balanced: false,
            bottom_delta: Mpt::ZERO,
        })
    }
    
    /// Balanced distribution: use balance solver for equal column heights.
    fn break_balanced(&self, lines: Vec<LineBox>) -> Result<SnakeColumnResult, SnakeColumnError> {
        let default_solver = BalanceSolver::default_solver();
        let solver = self.solver.as_ref().unwrap_or(&default_solver);
        
        let line_data = LineData::new(lines.len(), self.config.available_height);
        let num_cols = self.config.column_count;
        
        // Run balance solver
        let balance_result = solver
            .solve(&line_data, num_cols, &self.policy)
            .map_err(|e| SnakeColumnError::BalanceSolverFailed(e.to_string()))?;
        
        let (candidate, is_balanced) = match balance_result {
            BalanceResult::Exact(c) => (c, true),
            BalanceResult::Satisfiable(c) => (c, true),
            BalanceResult::Unsatisfiable { best_attempt, .. } => {
                match best_attempt {
                    Some(c) => (c, false),
                    None => return self.break_sequential(lines), // Fallback
                }
            }
        };
        
        // Distribute lines according to candidate's column allocation
        let mut columns = Vec::new();
        let mut column_heights = Vec::new();
        
        for col_content in &candidate.columns {
            let col_lines: Vec<LineBox> = col_content
                .line_indices
                .iter()
                .filter_map(|&idx| lines.get(idx).cloned())
                .collect();
            
            // Calculate actual column height
            let height = if col_lines.is_empty() {
                Mpt::ZERO
            } else {
                Mpt::from_pt(col_lines.iter().map(|l| l.height).sum::<f32>())
            };
            
            column_heights.push(height);
            columns.push(col_lines);
        }
        
        Ok(SnakeColumnResult {
            columns,
            column_heights,
            is_balanced,
            bottom_delta: candidate.bottom_delta,
        })
    }
}

/// Flow mode distinguishing parallel vs continuous column layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FlowMode {
    /// Parallel flows: independent content streams (Gemara + Rashi + Tosafot).
    #[default]
    Parallel,
    
    /// Continuous flow: single content stream across multiple columns (magazine style).
    Continuous,
}

/// Section-level column specification.
#[derive(Debug, Clone, PartialEq)]
pub struct SectionColumnSpec {
    /// Number of columns for this section.
    pub column_count: usize,
    
    /// Flow mode for this section.
    pub flow_mode: FlowMode,
    
    /// Distribution mode (only relevant for Continuous flow).
    pub distribution_mode: DistributionMode,
}

impl SectionColumnSpec {
    /// Create a specification for a section.
    pub fn new(column_count: usize, flow_mode: FlowMode) -> Self {
        Self {
            column_count,
            flow_mode,
            distribution_mode: DistributionMode::Balanced,
        }
    }
    
    /// Create parallel flow specification (existing behavior).
    pub fn parallel(column_count: usize) -> Self {
        Self::new(column_count, FlowMode::Parallel)
    }
    
    /// Create continuous flow specification (magazine style).
    pub fn continuous(column_count: usize) -> Self {
        Self::new(column_count, FlowMode::Continuous)
    }
    
    /// Set distribution mode.
    pub fn with_distribution_mode(mut self, mode: DistributionMode) -> Self {
        self.distribution_mode = mode;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_lines(count: usize) -> Vec<LineBox> {
        (0..count)
            .map(|i| LineBox {
                line_index: i,
                paragraph_id: None,
                baseline_y: (i as f32) * 12.0,
                height: 12.0,
                width: 200.0,
                glyphs: Vec::new(),
                text: format!("Line {}", i),
                is_rtl: false,
                fonts: Vec::new(),
                is_drop_cap: false,
                x_offset: 0.0,
            })
            .collect()
    }
    
    #[test]
    fn test_sequential_distribution() {
        let config = SnakeColumnConfig::new(2, Mpt::from_pt(600.0), 200.0)
            .with_distribution_mode(DistributionMode::Sequential);
        let breaker = SnakeColumnBreaker::new(config);
        
        let lines = create_test_lines(10);
        let result = breaker.break_into_columns(lines).unwrap();
        
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.columns[0].len(), 5);
        assert_eq!(result.columns[1].len(), 5);
        assert!(!result.is_balanced);
    }
    
    #[test]
    fn test_balanced_distribution() {
        let config = SnakeColumnConfig::new(2, Mpt::from_pt(600.0), 200.0)
            .with_distribution_mode(DistributionMode::Balanced);
        let breaker = SnakeColumnBreaker::new(config);
        
        let lines = create_test_lines(20);
        let result = breaker.break_into_columns(lines).unwrap();
        
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.columns[0].len() + result.columns[1].len(), 20);
        // Balanced mode should attempt to equalize
        let diff = (result.columns[0].len() as i32 - result.columns[1].len() as i32).abs();
        assert!(diff <= 2); // Allow small imbalance
    }
    
    #[test]
    fn test_three_columns() {
        let config = SnakeColumnConfig::new(3, Mpt::from_pt(600.0), 150.0)
            .with_distribution_mode(DistributionMode::Sequential);
        let breaker = SnakeColumnBreaker::new(config);
        
        let lines = create_test_lines(30);
        let result = breaker.break_into_columns(lines).unwrap();
        
        assert_eq!(result.columns.len(), 3);
        assert_eq!(result.columns[0].len(), 10);
        assert_eq!(result.columns[1].len(), 10);
        assert_eq!(result.columns[2].len(), 10);
    }
    
    #[test]
    fn test_empty_content() {
        let config = SnakeColumnConfig::new(2, Mpt::from_pt(600.0), 200.0);
        let breaker = SnakeColumnBreaker::new(config);
        
        let lines = Vec::new();
        let result = breaker.break_into_columns(lines);
        
        assert_eq!(result, Err(SnakeColumnError::NoContent));
    }
    
    #[test]
    fn test_zero_columns() {
        let config = SnakeColumnConfig::new(0, Mpt::from_pt(600.0), 200.0);
        let breaker = SnakeColumnBreaker::new(config);
        
        let lines = create_test_lines(10);
        let result = breaker.break_into_columns(lines);
        
        assert_eq!(result, Err(SnakeColumnError::InvalidColumnCount(0)));
    }
    
    #[test]
    fn test_single_column() {
        let config = SnakeColumnConfig::new(1, Mpt::from_pt(600.0), 400.0);
        let breaker = SnakeColumnBreaker::new(config);
        
        let lines = create_test_lines(15);
        let result = breaker.break_into_columns(lines).unwrap();
        
        assert_eq!(result.columns.len(), 1);
        assert_eq!(result.columns[0].len(), 15);
    }
    
    #[test]
    fn test_uneven_distribution() {
        let config = SnakeColumnConfig::new(3, Mpt::from_pt(600.0), 150.0)
            .with_distribution_mode(DistributionMode::Sequential);
        let breaker = SnakeColumnBreaker::new(config);
        
        let lines = create_test_lines(10);
        let result = breaker.break_into_columns(lines).unwrap();
        
        assert_eq!(result.columns.len(), 3);
        assert_eq!(result.columns[0].len(), 4);
        assert_eq!(result.columns[1].len(), 4);
        assert_eq!(result.columns[2].len(), 2);
    }
    
    #[test]
    fn test_empty_columns_when_few_lines() {
        let config = SnakeColumnConfig::new(5, Mpt::from_pt(600.0), 100.0)
            .with_distribution_mode(DistributionMode::Sequential);
        let breaker = SnakeColumnBreaker::new(config);
        
        let lines = create_test_lines(3);
        let result = breaker.break_into_columns(lines).unwrap();
        
        assert_eq!(result.columns.len(), 5);
        assert_eq!(result.columns[0].len(), 1);
        assert_eq!(result.columns[1].len(), 1);
        assert_eq!(result.columns[2].len(), 1);
        assert_eq!(result.columns[3].len(), 0);
        assert_eq!(result.columns[4].len(), 0);
    }
    
    #[test]
    fn test_flow_mode_enum() {
        assert_eq!(FlowMode::default(), FlowMode::Parallel);
        
        let parallel = FlowMode::Parallel;
        let continuous = FlowMode::Continuous;
        
        assert_ne!(parallel, continuous);
    }
    
    #[test]
    fn test_section_column_spec() {
        let spec = SectionColumnSpec::parallel(3);
        assert_eq!(spec.column_count, 3);
        assert_eq!(spec.flow_mode, FlowMode::Parallel);
        
        let spec = SectionColumnSpec::continuous(2)
            .with_distribution_mode(DistributionMode::Sequential);
        assert_eq!(spec.column_count, 2);
        assert_eq!(spec.flow_mode, FlowMode::Continuous);
        assert_eq!(spec.distribution_mode, DistributionMode::Sequential);
    }
}
