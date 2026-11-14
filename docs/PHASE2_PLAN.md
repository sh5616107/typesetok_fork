# Phase 2: Balance Measurement & Policy

## Prerequisites
- Phase 1 complete: Mpt, LayoutResult, Diagnostic, PagePolicy

## Objectives

### 1. Column Baseline Measurement
**File:** `crates/tok-typeset/src/balance_measurement.rs`

```rust
/// Measurement of column ending positions for balance calculation
pub struct ColumnMeasurement {
    pub column_index: usize,
    pub last_baseline_y: Option<Mpt>,
    pub line_count: usize,
    pub is_empty: bool,
}

/// Result of balance calculation between columns
pub struct BottomDelta {
    pub min_bottom: Mpt,
    pub max_bottom: Mpt,
    pub delta: Mpt,
    pub delta_lines: f32,
    pub empty_column_count: usize,
}

impl BottomDelta {
    /// Calculate from column measurements per spec section 8.1
    pub fn from_columns(
        measurements: &[ColumnMeasurement],
        baseline_step: Mpt,
    ) -> Result<Self, BalanceError> {
        // Filter out empty columns
        // Find min/max of lastBaselineY
        // Calculate delta = max - min
        // Calculate delta_lines = delta / baseline_step
    }
}
```

**Tests:**
- Exact match when columns end at same baseline
- Correct delta calculation with offset columns
- Empty column handling
- Invalid input (all empty, negative values)

### 2. Policy Assignment
**File:** Update `crates/tok-typeset/src/engine.rs`

```rust
impl TypesettingEngine {
    fn assign_page_policy(
        &self,
        page_index: usize,
        section: &SectionNode,
        is_last: bool,
        has_forced_break: bool,
    ) -> PagePolicy {
        // Logic matching spec section 7.5 table
        if is_last && is_content_sparse() {
            PagePolicy::LastPage
        } else if page_index == 0 && section.kind == SectionKind::Body {
            PagePolicy::ChapterOpening
        } else if has_forced_break {
            PagePolicy::ForcedBreak
        } else {
            PagePolicy::BodyFull
        }
    }
    
    fn populate_balance_info(
        &self,
        columns: &[ColumnLayout],
        policy: &PagePolicy,
        config: &TypesettingEngineConfig,
    ) -> BalanceInfo {
        let measurements: Vec<_> = columns
            .iter()
            .map(ColumnMeasurement::from_layout)
            .collect();
            
        let delta = BottomDelta::from_columns(
            &measurements,
            Mpt::from_pt(config.body_baseline_step_pt),
        );
        
        BalanceInfo {
            applicable: policy.balance_applicable(),
            target_delta: policy.target_delta(),
            actual_delta: Some(delta.delta),
            actual_delta_lines: Some(delta.delta_lines),
            balanced: delta.delta <= policy.success_threshold(),
            policy_id: policy.id(),
        }
    }
}
```

### 3. Diagnostic Integration

When balance fails:
```rust
if balance_info.applicable && !balance_info.balanced {
    diagnostics.push(Diagnostic::new(
        DiagnosticCode::BalanceUnsatisfiable,
        Severity::Error,
    )
    .with_page(page_index)
    .with_message(format!(
        "Columns not balanced: actualDelta = {} mpt ({:.2} lines), target = {} mpt",
        balance_info.actual_delta.unwrap().0,
        balance_info.actual_delta_lines.unwrap(),
        balance_info.target_delta.0,
    ))
    .with_strategy("Attempted: column division variations")
    .with_action("Enable paragraph spacing adjustment or use waiver"));
}
```

### 4. Acceptance Tests
**File:** `tests/spec_balance_basic.rs`

```rust
#[test]
fn test_balance_exact_match() {
    // Spec 16.1 Test 1: Regular two-column page
    let doc = create_test_document(/* enough for two equal columns */);
    let result = engine.typeset_document(&doc);
    
    assert_eq!(result.status, LayoutStatus::Valid);
    let page0 = &result.pages[0];
    assert!(page0.balance.applicable);
    assert_eq!(page0.balance.actual_delta, Some(Mpt(0)));
    assert!(page0.balance.balanced);
}

#[test]
fn test_balance_odd_paragraph() {
    // Spec 16.1 Test 2: Odd paragraph at end
    let doc = create_test_document_with_odd_paragraph();
    let result = engine.typeset_document(&doc);
    
    // Should attempt all balance stages or report failure
    if result.status == LayoutStatus::Invalid {
        assert!(result.diagnostics.iter().any(|d| 
            d.code == DiagnosticCode::BalanceUnsatisfiable
        ));
        assert!(result.diagnostics[0].attempted_strategies.is_some());
    } else {
        assert_eq!(result.pages[0].balance.actual_delta, Some(Mpt(0)));
    }
}

#[test]
fn test_balance_no_adjustment_available() {
    // Spec 16.1 Test 3: No paragraph spacing configured
    let mut doc = create_test_document(/* content */);
    doc.paragraph_styles[0].space_before_pt = 0.0;
    doc.paragraph_styles[0].space_after_pt = 0.0;
    // No space_before_range or space_after_range defined
    
    let result = engine.typeset_document(&doc);
    
    // Must either achieve balance through line breaking alone, or fail
    if result.pages[0].balance.actual_delta != Some(Mpt(0)) {
        assert_eq!(result.status, LayoutStatus::Invalid);
        assert!(result.diagnostics.iter().any(|d|
            d.code == DiagnosticCode::BalanceUnsatisfiable
        ));
    }
}
```

## Integration Points

### ColumnLayout Extension
Add to existing `ColumnLayout`:
```rust
pub struct ColumnLayout {
    pub column_index: usize,
    pub blocks: Vec<PositionedBlock>,
    
    // NEW: Balance measurement
    pub first_baseline_y: Option<Mpt>,
    pub last_baseline_y: Option<Mpt>,
    pub line_count: usize,
    pub occupied_height: Mpt,
}
```

### PageLayout Extension
Ensure `PageLayout` has:
```rust
pub struct PageLayout {
    // existing fields...
    
    pub balance: BalanceInfo,  // NEW in Phase 1
}
```

## Non-Goals for Phase 2

- ❌ Balance solver (that's Phase 3)
- ❌ Adjustment stages (Phase 3)
- ❌ Continuous flow (Phase 4)

Phase 2 only **measures and reports** balance status.
Phase 3 will **solve** balance problems.

## Success Criteria

- [ ] ColumnMeasurement correctly tracks lastBaselineY
- [ ] BottomDelta calculation matches spec section 8
- [ ] PagePolicy assigned correctly per section 7.5
- [ ] BalanceInfo populated in LayoutResult
- [ ] BALANCE_UNSATISFIABLE diagnostic when applicable
- [ ] All 3 basic balance tests pass
- [ ] Existing 699 tests still pass
- [ ] Documentation updated

## Timeline

Estimated: 1-2 weeks (3-5 work sessions)
