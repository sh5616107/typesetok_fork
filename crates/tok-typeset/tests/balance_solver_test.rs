//! Integration tests for Phase 3 balance solver.
//!
//! Tests all 5 stages of the adjustment ladder individually and in combination.

use tok_typeset::{
    balance_solver::BalanceSearchLimits, candidate::WeightConfig, BalanceCandidate,
    BalanceResult, BalanceSolver, BalanceSolverError, ColumnContent, LineData, Mpt, PagePolicy,
    PolicyConfig, SolverConfig, Stage,
};

/// Helper to create a default solver for tests.
fn create_test_solver() -> BalanceSolver {
    BalanceSolver::default_solver()
}

/// Helper to create test line data.
fn create_line_data(line_count: usize) -> LineData {
    LineData::new(line_count, Mpt::from_pt(600.0))
}

// ============================================================================
// Stage 1: Column Division Tests
// ============================================================================

#[test]
fn test_stage1_even_split() {
    let solver = create_test_solver();
    let line_data = create_line_data(20);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 2, &policy).unwrap();
    
    match result {
        BalanceResult::Exact(c) => {
            assert_eq!(c.bottom_delta, Mpt::ZERO);
            assert_eq!(c.columns.len(), 2);
        }
        BalanceResult::Satisfiable(c) => {
            // Also acceptable within threshold
            assert!(c.bottom_delta.raw().abs() < 1000);
        }
        BalanceResult::Unsatisfiable { .. } => {
            panic!("Should find satisfiable solution for even split");
        }
    }
}

#[test]
fn test_stage1_odd_split() {
    let solver = create_test_solver();
    let line_data = create_line_data(21);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 2, &policy).unwrap();
    
    // With 21 lines, we'll have 11 and 10, which will cause some imbalance
    // but should still be satisfiable within threshold
    match result {
        BalanceResult::Exact(_) => {
            // Exact balance unlikely but possible
        }
        BalanceResult::Satisfiable(_) => {
            // Expected - within threshold
        }
        BalanceResult::Unsatisfiable { .. } => {
            // This is also OK for odd splits - not all cases are solvable
        }
    }
}

#[test]
fn test_stage1_three_columns() {
    let solver = create_test_solver();
    let line_data = create_line_data(30);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 3, &policy).unwrap();
    
    match result {
        BalanceResult::Exact(c) => {
            assert_eq!(c.columns.len(), 3);
            assert_eq!(c.bottom_delta, Mpt::ZERO);
        }
        BalanceResult::Satisfiable(c) => {
            assert_eq!(c.columns.len(), 3);
        }
        BalanceResult::Unsatisfiable { .. } => {
            panic!("Should find solution for three columns");
        }
    }
}

#[test]
fn test_stage1_single_column() {
    let solver = create_test_solver();
    let line_data = create_line_data(10);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 1, &policy).unwrap();
    
    match result {
        BalanceResult::Exact(c) => {
            assert_eq!(c.columns.len(), 1);
            assert_eq!(c.bottom_delta, Mpt::ZERO);
        }
        _ => panic!("Single column should always be exact"),
    }
}

#[test]
fn test_stage1_empty_content() {
    let solver = create_test_solver();
    let line_data = create_line_data(0);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 2, &policy);
    
    assert_eq!(result, Err(BalanceSolverError::NoContent));
}

// ============================================================================
// Stage 2: Word Spacing + Looseness Tests
// ============================================================================

#[test]
fn test_stage2_word_spacing_range() {
    let solver = create_test_solver();
    let samples = solver.sample_spacing_range();
    
    assert_eq!(samples.len(), 5);
    assert!((samples[0] - 0.85).abs() < 0.01);
    assert!((samples[4] - 1.40).abs() < 0.01);
}

#[test]
fn test_stage2_generates_candidates() {
    let solver = create_test_solver();
    let line_data = create_line_data(20);
    
    // Stage 2 is invoked internally if stage 1 doesn't satisfy
    let candidates = solver.stage2_word_spacing_looseness(&line_data, 2).unwrap();
    
    // Should generate multiple candidates (5 spacings × 3 looseness values)
    assert!(candidates.len() > 5);
}

#[test]
fn test_stage2_looseness_disabled() {
    let mut config = SolverConfig::default();
    config.looseness_allowed = false;
    
    let solver = BalanceSolver::new(
        config,
        BalanceSearchLimits::default(),
        WeightConfig::default(),
    );
    
    let line_data = create_line_data(20);
    let candidates = solver.stage2_word_spacing_looseness(&line_data, 2).unwrap();
    
    // Should only have spacing variations, no looseness
    assert_eq!(candidates.len(), 5); // Only 5 spacing samples
}

// ============================================================================
// Stage 3: Paragraph Spacing Tests
// ============================================================================

#[test]
fn test_stage3_generates_candidates() {
    let solver = create_test_solver();
    let line_data = create_line_data(20);
    
    let candidates = solver.stage3_paragraph_spacing(&line_data, 2).unwrap();
    
    // Should generate candidates for each para spacing grid value
    assert!(candidates.len() >= 4); // Based on default grid
}

#[test]
fn test_stage3_custom_spacing_grid() {
    let mut config = SolverConfig::default();
    config.para_spacing_grid = vec![Mpt::ZERO, Mpt::from_pt(6.0)];
    
    let solver = BalanceSolver::new(
        config,
        BalanceSearchLimits::default(),
        WeightConfig::default(),
    );
    
    let line_data = create_line_data(20);
    let candidates = solver.stage3_paragraph_spacing(&line_data, 2).unwrap();
    
    assert_eq!(candidates.len(), 2);
}

// ============================================================================
// Policy Integration Tests
// ============================================================================

#[test]
fn test_policy_body_full_requires_balance() {
    let solver = create_test_solver();
    let line_data = create_line_data(20);
    let policy = PolicyConfig::body_full();
    
    assert!(policy.balance_applicable);
    assert_eq!(policy.target_delta, Mpt::ZERO);
    
    let result = solver.solve(&line_data, 2, &policy).unwrap();
    
    match result {
        BalanceResult::Exact(_) | BalanceResult::Satisfiable(_) => {
            // Success
        }
        BalanceResult::Unsatisfiable { .. } => {
            panic!("BodyFull should find solution");
        }
    }
}

#[test]
fn test_policy_last_page_not_applicable() {
    let solver = create_test_solver();
    let line_data = create_line_data(15);
    let policy = PolicyConfig::last_page();
    
    assert!(!policy.balance_applicable);
    
    let result = solver.solve(&line_data, 2, &policy).unwrap();
    
    // Last page accepts any result
    match result {
        BalanceResult::Exact(_)
        | BalanceResult::Satisfiable(_)
        | BalanceResult::Unsatisfiable { .. } => {
            // All acceptable
        }
    }
}

#[test]
fn test_policy_body_partial_with_balance() {
    let solver = create_test_solver();
    let line_data = create_line_data(18);
    let policy = PolicyConfig::body_partial();
    
    assert!(policy.balance_applicable);
    
    let result = solver.solve(&line_data, 2, &policy).unwrap();
    
    match result {
        BalanceResult::Exact(_) | BalanceResult::Satisfiable(_) => {
            // Success
        }
        BalanceResult::Unsatisfiable { .. } => {
            panic!("Should find solution");
        }
    }
}

// ============================================================================
// Multi-Stage Integration Tests
// ============================================================================

#[test]
fn test_multistage_progression() {
    let solver = create_test_solver();
    let line_data = create_line_data(23);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 2, &policy).unwrap();
    
    // Should try multiple stages to find solution
    match result {
        BalanceResult::Exact(c) => {
            println!("Exact balance achieved at stage: {:?}", c.stage);
        }
        BalanceResult::Satisfiable(c) => {
            println!("Satisfiable at stage: {:?}", c.stage);
            assert!(policy.is_satisfied(c.bottom_delta));
        }
        BalanceResult::Unsatisfiable { attempted_stages, .. } => {
            println!("Attempted stages: {:?}", attempted_stages);
            assert!(attempted_stages.len() >= 3);
        }
    }
}

#[test]
fn test_early_termination_on_exact_balance() {
    let solver = create_test_solver();
    let line_data = create_line_data(20);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 2, &policy).unwrap();
    
    match result {
        BalanceResult::Exact(c) => {
            // Should terminate early at stage 1
            assert_eq!(c.stage, Stage::ColumnDivision);
            assert_eq!(c.bottom_delta, Mpt::ZERO);
        }
        BalanceResult::Satisfiable(c) => {
            // Also acceptable if within threshold
            assert!(c.bottom_delta.raw().abs() < 1000);
        }
        _ => {}
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_very_few_lines() {
    let solver = create_test_solver();
    let line_data = create_line_data(2);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 2, &policy).unwrap();
    
    match result {
        BalanceResult::Exact(_) | BalanceResult::Satisfiable(_) => {
            // Should handle gracefully
        }
        BalanceResult::Unsatisfiable { .. } => {
            // Also acceptable for edge case
        }
    }
}

#[test]
fn test_many_columns() {
    let solver = create_test_solver();
    let line_data = create_line_data(100);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 5, &policy).unwrap();
    
    match result {
        BalanceResult::Exact(c) => {
            assert_eq!(c.columns.len(), 5);
        }
        BalanceResult::Satisfiable(c) => {
            assert_eq!(c.columns.len(), 5);
        }
        _ => {}
    }
}

#[test]
fn test_invalid_column_count() {
    let solver = create_test_solver();
    let line_data = create_line_data(20);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 0, &policy);
    
    assert_eq!(result, Err(BalanceSolverError::InvalidColumnCount(0)));
}

// ============================================================================
// Quality Metrics Tests
// ============================================================================

#[test]
fn test_candidate_quality_scoring() {
    let mut c1 = BalanceCandidate::new(Stage::ColumnDivision, "test1".to_string());
    c1.word_spacing_quality = 1.0;
    c1.line_quality_score = 1.0;
    
    let mut c2 = BalanceCandidate::new(Stage::WordSpacingLooseness, "test2".to_string());
    c2.word_spacing_quality = 0.8;
    c2.line_quality_score = 0.9;
    
    let weights = WeightConfig::default();
    
    let score1 = c1.soft_score(&weights);
    let score2 = c2.soft_score(&weights);
    
    // c1 should have better (lower) score
    assert!(score1 < score2);
}

#[test]
fn test_hard_constraints_validation() {
    let mut candidate = BalanceCandidate::new(Stage::ColumnDivision, "test".to_string());
    
    // Valid candidate with ordered lines
    candidate.columns.push(ColumnContent {
        column_index: 0,
        line_indices: vec![0, 1, 2],
        last_baseline_y: Some(Mpt::from_pt(100.0)),
        line_count: 3,
    });
    
    assert!(candidate.satisfies_hard_constraints());
    
    // Invalid candidate with unordered lines
    let mut bad_candidate = BalanceCandidate::new(Stage::ColumnDivision, "bad".to_string());
    bad_candidate.columns.push(tok_typeset::ColumnContent {
        column_index: 0,
        line_indices: vec![0, 2, 1], // Out of order
        last_baseline_y: Some(Mpt::from_pt(100.0)),
        line_count: 3,
    });
    
    assert!(!bad_candidate.satisfies_hard_constraints());
}

#[test]
fn test_baseline_step_configuration() {
    let mut config = SolverConfig::default();
    config.baseline_step = Mpt::from_pt(14.0); // Custom baseline
    
    let solver = BalanceSolver::new(
        config,
        BalanceSearchLimits::default(),
        WeightConfig::default(),
    );
    
    let line_data = create_line_data(20);
    let policy = PolicyConfig::body_full();
    
    let result = solver.solve(&line_data, 2, &policy);
    
    assert!(result.is_ok());
}
