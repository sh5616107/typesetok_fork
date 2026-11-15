//! Integration tests for Phase 4 Snake Column Breaking.

use tok_typeset::{
    DistributionMode, FlowMode, SectionColumnSpec, SnakeColumnBreaker, SnakeColumnConfig,
};
use tok_typeset::{LineBox, Mpt};

fn create_test_lines(count: usize, height: f32) -> Vec<LineBox> {
    (0..count)
        .map(|i| LineBox {
            line_index: i,
            paragraph_id: None,
            baseline_y: (i as f32) * height,
            height,
            width: 200.0,
            glyphs: Vec::new(),
            text: format!("Line {} content", i),
            is_rtl: false,
            fonts: Vec::new(),
            is_drop_cap: false,
            x_offset: 0.0,
        })
        .collect()
}

#[test]
fn test_single_to_two_to_single_column_transitions() {
    // Simulate section 1: single column
    let config1 = SnakeColumnConfig::new(1, Mpt::from_pt(600.0), 400.0);
    let breaker1 = SnakeColumnBreaker::new(config1);
    let lines1 = create_test_lines(10, 12.0);
    let result1 = breaker1.break_into_columns(lines1).unwrap();
    
    assert_eq!(result1.columns.len(), 1);
    assert_eq!(result1.columns[0].len(), 10);
    
    // Simulate section 2: two columns
    let config2 = SnakeColumnConfig::new(2, Mpt::from_pt(600.0), 200.0);
    let breaker2 = SnakeColumnBreaker::new(config2);
    let lines2 = create_test_lines(20, 12.0);
    let result2 = breaker2.break_into_columns(lines2).unwrap();
    
    assert_eq!(result2.columns.len(), 2);
    assert_eq!(result2.columns[0].len() + result2.columns[1].len(), 20);
    
    // Simulate section 3: back to single column
    let config3 = SnakeColumnConfig::new(1, Mpt::from_pt(600.0), 400.0);
    let breaker3 = SnakeColumnBreaker::new(config3);
    let lines3 = create_test_lines(15, 12.0);
    let result3 = breaker3.break_into_columns(lines3).unwrap();
    
    assert_eq!(result3.columns.len(), 1);
    assert_eq!(result3.columns[0].len(), 15);
}

#[test]
fn test_three_column_magazine_layout() {
    let config = SnakeColumnConfig::new(3, Mpt::from_pt(800.0), 150.0)
        .with_distribution_mode(DistributionMode::Balanced);
    let breaker = SnakeColumnBreaker::new(config);
    
    // Create 60 lines (should distribute ~20 per column)
    let lines = create_test_lines(60, 12.0);
    let result = breaker.break_into_columns(lines).unwrap();
    
    assert_eq!(result.columns.len(), 3);
    
    // Check that all lines are distributed
    let total_lines: usize = result.columns.iter().map(|c| c.len()).sum();
    assert_eq!(total_lines, 60);
    
    // In balanced mode, columns should be roughly equal
    for col in &result.columns {
        assert!(col.len() >= 18 && col.len() <= 22);
    }
}

#[test]
fn test_empty_columns_short_content() {
    let config = SnakeColumnConfig::new(4, Mpt::from_pt(600.0), 120.0)
        .with_distribution_mode(DistributionMode::Sequential);
    let breaker = SnakeColumnBreaker::new(config);
    
    // Only 2 lines, but 4 columns requested
    let lines = create_test_lines(2, 12.0);
    let result = breaker.break_into_columns(lines).unwrap();
    
    assert_eq!(result.columns.len(), 4);
    assert_eq!(result.columns[0].len(), 1);
    assert_eq!(result.columns[1].len(), 1);
    assert_eq!(result.columns[2].len(), 0); // Empty
    assert_eq!(result.columns[3].len(), 0); // Empty
}

#[test]
fn test_very_long_content_spanning_multiple_pages() {
    let config = SnakeColumnConfig::new(2, Mpt::from_pt(600.0), 200.0)
        .with_distribution_mode(DistributionMode::Balanced);
    let breaker = SnakeColumnBreaker::new(config);
    
    // Simulate 200 lines - would span multiple pages in real scenario
    let lines = create_test_lines(200, 12.0);
    let result = breaker.break_into_columns(lines).unwrap();
    
    assert_eq!(result.columns.len(), 2);
    
    let total_lines: usize = result.columns.iter().map(|c| c.len()).sum();
    assert_eq!(total_lines, 200);
    
    // Columns should be balanced (within 2 lines)
    let diff = (result.columns[0].len() as i32 - result.columns[1].len() as i32).abs();
    assert!(diff <= 2);
}

#[test]
fn test_sequential_vs_balanced_distribution() {
    let lines = create_test_lines(21, 12.0);
    
    // Sequential mode
    let config_seq = SnakeColumnConfig::new(2, Mpt::from_pt(600.0), 200.0)
        .with_distribution_mode(DistributionMode::Sequential);
    let breaker_seq = SnakeColumnBreaker::new(config_seq);
    let result_seq = breaker_seq.break_into_columns(lines.clone()).unwrap();
    
    // Sequential fills columns in order
    assert_eq!(result_seq.columns[0].len(), 11); // ceiling(21/2)
    assert_eq!(result_seq.columns[1].len(), 10);
    assert!(!result_seq.is_balanced);
    
    // Balanced mode
    let config_bal = SnakeColumnConfig::new(2, Mpt::from_pt(600.0), 200.0)
        .with_distribution_mode(DistributionMode::Balanced);
    let breaker_bal = SnakeColumnBreaker::new(config_bal);
    let result_bal = breaker_bal.break_into_columns(lines).unwrap();
    
    // Balanced attempts to equalize
    let diff = (result_bal.columns[0].len() as i32 - result_bal.columns[1].len() as i32).abs();
    assert!(diff <= 1);
}

#[test]
fn test_section_column_spec_creation() {
    let parallel_spec = SectionColumnSpec::parallel(3);
    assert_eq!(parallel_spec.column_count, 3);
    assert_eq!(parallel_spec.flow_mode, FlowMode::Parallel);
    
    let continuous_spec = SectionColumnSpec::continuous(2)
        .with_distribution_mode(DistributionMode::Sequential);
    assert_eq!(continuous_spec.column_count, 2);
    assert_eq!(continuous_spec.flow_mode, FlowMode::Continuous);
    assert_eq!(continuous_spec.distribution_mode, DistributionMode::Sequential);
}

#[test]
fn test_flow_mode_compatibility() {
    // FlowMode should be distinguishable
    assert_ne!(FlowMode::Parallel, FlowMode::Continuous);
    
    // Default should be Parallel (preserve existing behavior)
    assert_eq!(FlowMode::default(), FlowMode::Parallel);
}

#[test]
fn test_rtl_content_handling() {
    let config = SnakeColumnConfig::new(2, Mpt::from_pt(600.0), 200.0);
    let breaker = SnakeColumnBreaker::new(config);
    
    // Create RTL lines (Hebrew)
    let lines: Vec<LineBox> = (0..10)
        .map(|i| LineBox {
            line_index: i,
            paragraph_id: None,
            baseline_y: (i as f32) * 12.0,
            height: 12.0,
            width: 200.0,
            glyphs: Vec::new(),
            text: format!("שורה {}", i), // Hebrew "Line"
            is_rtl: true,
            fonts: Vec::new(),
            is_drop_cap: false,
            x_offset: 0.0,
        })
        .collect();
    
    let result = breaker.break_into_columns(lines).unwrap();
    
    assert_eq!(result.columns.len(), 2);
    
    // Verify RTL property is preserved
    for col in &result.columns {
        for line in col {
            assert!(line.is_rtl);
        }
    }
}

#[test]
fn test_column_height_calculation() {
    let config = SnakeColumnConfig::new(3, Mpt::from_pt(600.0), 150.0)
        .with_distribution_mode(DistributionMode::Sequential);
    let breaker = SnakeColumnBreaker::new(config);
    
    // Create lines with varying heights
    let lines: Vec<LineBox> = vec![
        LineBox {
            line_index: 0,
            paragraph_id: None,
            baseline_y: 0.0,
            height: 12.0,
            width: 150.0,
            glyphs: Vec::new(),
            text: "Line 1".to_string(),
            is_rtl: false,
            fonts: Vec::new(),
            is_drop_cap: false,
            x_offset: 0.0,
        },
        LineBox {
            line_index: 1,
            paragraph_id: None,
            baseline_y: 12.0,
            height: 14.0, // Different height
            width: 150.0,
            glyphs: Vec::new(),
            text: "Line 2".to_string(),
            is_rtl: false,
            fonts: Vec::new(),
            is_drop_cap: false,
            x_offset: 0.0,
        },
        LineBox {
            line_index: 2,
            paragraph_id: None,
            baseline_y: 26.0,
            height: 18.0, // Even larger
            width: 150.0,
            glyphs: Vec::new(),
            text: "Line 3".to_string(),
            is_rtl: false,
            fonts: Vec::new(),
            is_drop_cap: false,
            x_offset: 0.0,
        },
    ];
    
    let result = breaker.break_into_columns(lines).unwrap();
    
    assert_eq!(result.columns.len(), 3);
    assert_eq!(result.column_heights.len(), 3);
    
    // Each column should have one line
    assert_eq!(result.columns[0].len(), 1);
    assert_eq!(result.columns[1].len(), 1);
    assert_eq!(result.columns[2].len(), 1);
    
    // Heights should match the line heights
    assert_eq!(result.column_heights[0], Mpt::from_pt(12.0));
    assert_eq!(result.column_heights[1], Mpt::from_pt(14.0));
    assert_eq!(result.column_heights[2], Mpt::from_pt(18.0));
}
