//! Integration tests for Phase 2: Balance Measurement & Policy Assignment.
//!
//! Tests basic balance calculation scenarios per specification section 8.1.

use tok_core::model::SectionNode;
use tok_typeset::{
    BalanceError, BalanceInfo, BottomDelta, ColumnLayout, ColumnMeasurement, Mpt, PagePolicy,
    TypesettingEngine, TypesettingEngineConfig,
};

#[test]
fn test_balance_exact_match() {
    // Two columns with identical endpoints should have zero delta
    let measurements = vec![
        ColumnMeasurement::new(0)
            .with_last_baseline_y(Mpt::from_pt(100.0))
            .with_line_count(10),
        ColumnMeasurement::new(1)
            .with_last_baseline_y(Mpt::from_pt(100.0))
            .with_line_count(10),
    ];

    let baseline_step = Mpt::from_pt(12.0);
    let result = BottomDelta::from_columns(&measurements, baseline_step).unwrap();

    assert_eq!(result.delta, Mpt::ZERO);
    assert_eq!(result.delta_lines, 0.0);
    assert!(result.is_within_threshold(Mpt::from_pt(0.5)));
}

#[test]
fn test_balance_odd_paragraph() {
    // Odd paragraph causes imbalance
    // Column 1 ends at 100pt (10 lines)
    // Column 2 ends at 112pt (11 lines)
    // Delta should be 12pt = 1 line
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

    assert_eq!(result.delta, Mpt::from_pt(12.0));
    assert_eq!(result.delta_lines, 1.0);
    assert_eq!(result.min_bottom, Mpt::from_pt(100.0));
    assert_eq!(result.max_bottom, Mpt::from_pt(112.0));
}

#[test]
fn test_balance_no_adjustment() {
    // No spacing adjustment available - columns have significant imbalance
    // This tests measurement only, not solving
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
    // This would not satisfy typical balance requirements
    assert!(!result.is_within_threshold(Mpt::from_pt(0.5)));
}

#[test]
fn test_balance_policy_assignment() {
    // Verify correct policy assignment for different scenarios
    let engine = TypesettingEngine::new(TypesettingEngineConfig::default());

    // Test BodyFull (default, middle page)
    let section = SectionNode::new("Test Section", "default");
    let policy = engine.assign_page_policy(1, &section, false, false);
    assert_eq!(policy, PagePolicy::BodyFull);

    // Test LastPage (is_last = true)
    let policy = engine.assign_page_policy(5, &section, true, false);
    assert_eq!(policy, PagePolicy::LastPage);

    // Test ChapterOpening (page_index = 0, chapter style)
    let chapter_section = SectionNode::new("Chapter 1", "chapter-first");
    let policy = engine.assign_page_policy(0, &chapter_section, false, false);
    assert_eq!(policy, PagePolicy::ChapterOpening);

    // Test ForcedBreak
    let policy = engine.assign_page_policy(2, &section, false, true);
    assert_eq!(policy, PagePolicy::ForcedBreak);
}

#[test]
fn test_populate_balance_info_applicable() {
    // Test balance info population for applicable pages
    let engine = TypesettingEngine::new(TypesettingEngineConfig::default());

    let columns = vec![
        ColumnLayout {
            column_index: 0,
            last_baseline_y: Some(Mpt::from_pt(100.0)),
            height: Mpt::from_pt(200.0),
            is_empty: false,
        },
        ColumnLayout {
            column_index: 1,
            last_baseline_y: Some(Mpt::from_pt(106.0)),
            height: Mpt::from_pt(200.0),
            is_empty: false,
        },
    ];

    let policy = PagePolicy::BodyFull;
    let baseline_step = Mpt::from_pt(12.0);

    let balance_info = engine.populate_balance_info(&columns, &policy, baseline_step);

    assert!(balance_info.applicable);
    assert_eq!(balance_info.target_delta, Mpt::ZERO);
    assert_eq!(balance_info.actual_delta, Some(Mpt::from_pt(6.0)));
    assert_eq!(balance_info.actual_delta_lines, Some(0.5));
    // Should be balanced since 6pt is within 0.5pt threshold? No, it's not
    assert!(!balance_info.balanced);
}

#[test]
fn test_populate_balance_info_not_applicable() {
    // Test balance info population for non-applicable pages (LastPage)
    let engine = TypesettingEngine::new(TypesettingEngineConfig::default());

    let columns = vec![ColumnLayout {
        column_index: 0,
        last_baseline_y: Some(Mpt::from_pt(100.0)),
        height: Mpt::from_pt(200.0),
        is_empty: false,
    }];

    let policy = PagePolicy::LastPage;
    let baseline_step = Mpt::from_pt(12.0);

    let balance_info = engine.populate_balance_info(&columns, &policy, baseline_step);

    assert!(!balance_info.applicable);
    assert!(balance_info.balanced); // Automatically satisfied when not applicable
    assert_eq!(balance_info.actual_delta, None);
}

#[test]
fn test_balance_with_empty_column() {
    // Test balance calculation when one column is empty
    let measurements = vec![
        ColumnMeasurement::new(0)
            .with_last_baseline_y(Mpt::from_pt(100.0))
            .with_line_count(10),
        ColumnMeasurement::new(1), // Empty column
    ];

    let baseline_step = Mpt::from_pt(12.0);
    let result = BottomDelta::from_columns(&measurements, baseline_step).unwrap();

    // Empty column should be filtered out, so delta is 0 (only one non-empty column)
    assert_eq!(result.delta, Mpt::ZERO);
    assert_eq!(result.empty_column_count, 1);
}

#[test]
fn test_balance_all_empty_columns() {
    // Test error case: all columns empty
    let measurements = vec![ColumnMeasurement::new(0), ColumnMeasurement::new(1)];

    let baseline_step = Mpt::from_pt(12.0);
    let result = BottomDelta::from_columns(&measurements, baseline_step);

    assert_eq!(result, Err(BalanceError::AllColumnsEmpty));
}

#[test]
fn test_balance_info_builder_pattern() {
    // Test the builder pattern for BalanceInfo
    let info = BalanceInfo::applicable(Mpt::ZERO, "body_full")
        .with_actual_delta(Mpt::from_pt(0.3), 0.5)
        .with_balanced(true);

    assert!(info.applicable);
    assert_eq!(info.target_delta, Mpt::ZERO);
    assert_eq!(info.actual_delta, Some(Mpt::from_pt(0.3)));
    assert_eq!(info.actual_delta_lines, Some(0.5));
    assert!(info.balanced);
    assert_eq!(info.policy_id, "body_full");
}

#[test]
fn test_three_column_balance() {
    // Test balance with three columns
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

    // Delta should be between min (100) and max (112) = 12pt
    assert_eq!(result.min_bottom, Mpt::from_pt(100.0));
    assert_eq!(result.max_bottom, Mpt::from_pt(112.0));
    assert_eq!(result.delta, Mpt::from_pt(12.0));
    assert_eq!(result.delta_lines, 1.0);
}
