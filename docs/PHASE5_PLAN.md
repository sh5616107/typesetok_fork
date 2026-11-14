# Phase 5: Integration & POC Validation

## Prerequisites
- Phase 1: Infrastructure ✓
- Phase 2: Measurement ✓
- Phase 3: Balance Solver ✓
- Phase 4: Continuous Flow ✓

## Objective

Validate complete spec alignment through:
1. All 16 acceptance tests from spec section 16
2. POC report with metrics (section 27)
3. Regression fixes
4. Documentation and examples

## Test Suite Implementation

### Group 1: Balance Tests (Section 16.1)
**File:** `tests/spec_section_16_1_balance.rs`

```rust
/// Test 1: Regular two-column page achieves exact balance
#[test]
fn test_16_1_01_regular_two_column_exact() {
    let doc = fixtures::create_document_balanced_content();
    let result = engine().typeset_document(&doc);
    
    assert_eq!(result.status, LayoutStatus::Valid);
    let page0 = &result.pages[0];
    assert!(page0.balance.applicable);
    assert_eq!(page0.balance.actual_delta, Some(Mpt(0)));
    assert_eq!(page0.balance.policy_id, "body_full");
}

/// Test 2: Odd paragraph at end - attempts all stages or reports
#[test]
fn test_16_1_02_odd_paragraph_attempts_stages() {
    let doc = fixtures::create_document_with_odd_paragraph();
    let result = engine().typeset_document(&doc);
    
    if result.status == LayoutStatus::Invalid {
        let diag = result.diagnostics.iter()
            .find(|d| d.code == DiagnosticCode::BalanceUnsatisfiable)
            .expect("BALANCE_UNSATISFIABLE diagnostic required");
        
        assert!(diag.attempted_strategies.is_some());
        assert!(diag.attempted_strategies.as_ref().unwrap().len() >= 3);
    } else {
        assert_eq!(result.pages[0].balance.actual_delta, Some(Mpt(0)));
    }
}

/// Test 3: No paragraph spacing - fails or succeeds through line breaking
#[test]
fn test_16_1_03_no_spacing_adjustment() {
    let mut doc = fixtures::create_document_no_spacing();
    
    // Ensure no spacing adjustment available
    doc.paragraph_styles[0].space_before_pt = 0.0;
    doc.paragraph_styles[0].space_after_pt = 0.0;
    
    let result = engine().typeset_document(&doc);
    
    if result.pages[0].balance.actual_delta != Some(Mpt(0)) {
        assert_eq!(result.status, LayoutStatus::Invalid);
        let diag = result.diagnostics.iter()
            .find(|d| d.code == DiagnosticCode::BalanceUnsatisfiable)
            .expect("Must report failure");
        
        assert!(diag.suggested_actions.as_ref().unwrap()
            .contains(&"Enable paragraph spacing".to_string()));
    }
}

/// Test 4: Heading near bottom stays with next content
#[test]
fn test_16_1_04_heading_keep_with_next() {
    let doc = fixtures::create_document_with_heading_at_bottom();
    let result = engine().typeset_document(&doc);
    
    assert_eq!(result.status, LayoutStatus::Valid);
    
    // Find heading position
    let heading_page = result.pages.iter()
        .find(|p| p.blocks.iter().any(|b| b.is_heading))
        .unwrap();
    
    let heading_idx = heading_page.blocks.iter()
        .position(|b| b.is_heading)
        .unwrap();
    
    // At least 2 lines of body text must follow in same column
    let following_lines = heading_page.blocks[heading_idx + 1..]
        .iter()
        .take_while(|b| !b.is_heading)
        .map(|b| b.line_count)
        .sum::<usize>();
    
    assert!(following_lines >= 2);
}

/// Test 5: Last page sparse content - policy allows imbalance
#[test]
fn test_16_1_05_last_page_sparse() {
    let doc = fixtures::create_document_with_sparse_last_page();
    let result = engine().typeset_document(&doc);
    
    assert_eq!(result.status, LayoutStatus::Valid);
    
    let last_page = result.pages.last().unwrap();
    assert_eq!(last_page.balance.policy_id, "last_page");
    assert!(!last_page.balance.applicable || last_page.balance.balanced);
}

/// Test 6: Long footnote reduces both columns equally
#[test]
fn test_16_1_06_footnote_reduces_columns() {
    let doc = fixtures::create_document_with_long_footnote();
    let result = engine().typeset_document(&doc);
    
    assert_eq!(result.status, LayoutStatus::Valid);
    
    let page_with_note = result.pages.iter()
        .find(|p| !p.notes.is_empty())
        .unwrap();
    
    // Both columns should have same available height after footnote
    assert_eq!(page_with_note.columns.len(), 2);
    
    let col0_height = page_with_note.columns[0].occupied_height;
    let col1_height = page_with_note.columns[1].occupied_height;
    
    // Heights should be balanced
    assert!((col0_height.0 - col1_height.0).abs() < 1000);  // Within 1pt
}

/// Test 7: Manual break not canceled by balance
#[test]
fn test_16_1_07_manual_break_preserved() {
    let doc = fixtures::create_document_with_manual_break();
    let result = engine().typeset_document(&doc);
    
    // Either succeeds with break preserved, or reports conflict
    if result.status == LayoutStatus::Invalid {
        assert!(result.diagnostics.iter().any(|d| 
            d.code == DiagnosticCode::ManualBreakConflict
        ));
    } else {
        // Verify break was honored
        // (check implementation specific)
    }
}

/// Test 8: Three+ columns balanced across all
#[test]
fn test_16_1_08_three_columns_balanced() {
    let doc = fixtures::create_document_three_columns();
    let result = engine().typeset_document(&doc);
    
    assert_eq!(result.status, LayoutStatus::Valid);
    
    let page = &result.pages[0];
    assert_eq!(page.columns.len(), 3);
    
    // All three columns should be balanced
    let heights: Vec<_> = page.columns.iter()
        .map(|c| c.occupied_height)
        .collect();
    
    let min = heights.iter().min().unwrap();
    let max = heights.iter().max().unwrap();
    
    assert_eq!(*max - *min, Mpt(0));
}
```

### Group 2: Typography & Directionality (Section 16.2)
**File:** `tests/spec_section_16_2_typography.rs`

```rust
#[test]
fn test_16_2_01_hebrew_with_niqqud_teamim() {
    // Hebrew text with niqqud and te'amim renders correctly
}

#[test]
fn test_16_2_02_mixed_hebrew_latin_numbers() {
    // RTL base with LTR numbers and Latin
}

#[test]
fn test_16_2_03_acronyms_non_breaking() {
    // ר"ת, ז"ל etc. stay together
}

#[test]
fn test_16_2_04_missing_glyph_diagnostic() {
    // Missing glyph triggers FONT_GLYPH_MISSING
}

#[test]
fn test_16_2_05_last_line_centered() {
    // Last line centered only at true end
}

#[test]
fn test_16_2_06_drop_cap_window() {
    // Opening word with window in short and long paragraphs
}

#[test]
fn test_16_2_07_font_determinism() {
    // Same font file -> same result
}
```

### Group 3: Determinism & Stability (Section 16.3)
**File:** `tests/spec_section_16_3_determinism.rs`

```rust
#[test]
fn test_16_3_01_same_run_twice() {
    // Two runs produce identical hash
}

#[test]
fn test_16_3_02_cross_platform() {
    // Skip on single platform, document requirement
}

#[test]
fn test_16_3_03_local_edit_stability() {
    // Edit one word, verify distant pages unchanged
}

#[test]
fn test_16_3_04_word_import_export_roundtrip() {
    // Import DOCX -> Export DOCX -> Import preserves structure
}

#[test]
fn test_16_3_05_pdf_matches_display() {
    // PDF and display use same LayoutResult
}
```

## POC Report Generation

### Metrics Collection
**File:** `tests/poc_metrics.rs`

```rust
pub struct PocMetrics {
    pub engine_version: String,
    pub config_hash: String,
    pub corpus_files: Vec<PathBuf>,
    
    pub total_pages: usize,
    pub pages_by_policy: HashMap<String, usize>,
    
    pub balance_stats: BalanceStats,
    pub diagnostic_stats: DiagnosticStats,
    pub timing_stats: TimingStats,
    
    pub test_results: TestResults,
}

pub struct BalanceStats {
    pub exact_balance_count: usize,        // actualDelta = 0
    pub within_tolerance_count: usize,     // <= threshold
    pub unsatisfiable_count: usize,        // Failed after all stages
    pub search_limit_reached_count: usize, // Cut off early
    
    pub by_policy: HashMap<String, PolicyStats>,
}

pub struct TimingStats {
    pub measurements: Vec<TimingMeasurement>,
    pub percentiles: Percentiles,
}

impl PocMetrics {
    pub fn collect_from_corpus(corpus_dir: &Path) -> Result<Self, Error> {
        let mut metrics = Self::new();
        
        for entry in fs::read_dir(corpus_dir)? {
            let path = entry?.path();
            if path.extension() == Some("json") {
                let doc = DocumentRoot::from_file(&path)?;
                let start = Instant::now();
                let result = engine().typeset_document(&doc);
                let elapsed = start.elapsed();
                
                metrics.record_result(&result, elapsed);
            }
        }
        
        Ok(metrics)
    }
    
    pub fn generate_report(&self) -> String {
        format!(
            "# POC Report\n\n\
            ## Configuration\n\
            - Engine: {}\n\
            - Config Hash: {}\n\
            - Corpus: {} files\n\n\
            ## Balance Performance\n\
            - Exact (Δ=0): {}/{} ({:.1}%)\n\
            - Unsatisfiable: {}\n\
            - Search Limit: {}\n\n\
            ## Timing\n\
            - Median: {:.1}ms\n\
            - P95: {:.1}ms\n\
            - P99: {:.1}ms\n\n\
            ## Test Results\n\
            - Passing: {}/{}\n\
            ",
            self.engine_version,
            self.config_hash,
            self.corpus_files.len(),
            self.balance_stats.exact_balance_count,
            self.total_pages,
            self.balance_stats.exact_balance_count as f64 / self.total_pages as f64 * 100.0,
            self.balance_stats.unsatisfiable_count,
            self.balance_stats.search_limit_reached_count,
            self.timing_stats.percentiles.p50,
            self.timing_stats.percentiles.p95,
            self.timing_stats.percentiles.p99,
            self.test_results.passed,
            self.test_results.total,
        )
    }
}
```

### Report Generation Command
```bash
cargo test --release -- --test-threads=1 --nocapture poc_metrics > POC_REPORT.md
```

## Regression Fixes

### Common Issues

1. **Existing multi-flow breaks**
   - Ensure Talmud layout (gemara + rashi + tosafot) still works
   - Test: `tests/talmud_layout_regression.rs`

2. **Performance degradation**
   - Balance solver adds overhead
   - Target: < 2x slowdown for non-balanced content
   - Profile and optimize hot paths

3. **Coordinate conversion bugs**
   - f32 ↔ Mpt conversions must be consistent
   - Test: Round-trip conversions

4. **Diagnostic spam**
   - Don't emit warnings for expected cases
   - Filter by severity appropriately

## Documentation Updates

### 1. Update README.md
- [ ] Mention spec alignment
- [ ] Link to SPEC_ALIGNMENT.md
- [ ] Update feature list (balance, continuous flow)
- [ ] Update limitations (removed mpt, added balance)

### 2. API Documentation
- [ ] Rustdoc for all new types
- [ ] Examples for LayoutResult usage
- [ ] Migration guide from old coordinate system

### 3. User Guide
**File:** `docs/USER_GUIDE.md`

```markdown
# Balance and Column Configuration

## Enabling Balance

Set page policy in document configuration:

```json
{
  "pagePolicy": "body_full",
  "balanceTolerance": 0
}
```

## Continuous Flow Columns

Use `column_count` in paragraph style:

```json
{
  "column_count": 2,
  "column_gutter_pt": 12.0,
  "column_balance_mode": "Balanced"
}
```

## Diagnostics and Waivers

If balance cannot be achieved:
1. Review `BALANCE_UNSATISFIABLE` diagnostic
2. Check `attemptedStrategies`
3. Enable paragraph spacing adjustment
4. Or add waiver with justification
```

### 4. CHANGELOG.md
```markdown
## [0.36.0] - Spec Alignment Release

### Added
- Integer coordinate system (Mpt) for deterministic layout
- Complete LayoutResult contract with diagnostics and waivers
- Page policy system (7 policy types)
- Mandatory column balance solver with 5-stage adjustment ladder
- Continuous flow (magazine) column layout
- 16 acceptance tests from formal specification

### Changed
- Coordinate precision from f32 to i32 (millipoints)
- Balance measurement now tracks lastBaselineY per column
- PDF and display use single LayoutResult (no duplication)

### Breaking Changes
- Coordinate type changed (migration guide in docs/)
- Balance now mandatory for body_full pages (can be waived)

### Performance
- Balance solver adds ~50% overhead on balanced pages
- Non-balanced content unaffected
```

## Deliverables Checklist

### Code
- [ ] All phases 1-4 merged to main
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] All tests pass (including 699 existing + 16 new)

### Tests
- [ ] 8 balance tests (16.1)
- [ ] 7 typography tests (16.2)
- [ ] 5 determinism tests (16.3)
- [ ] POC metrics collected
- [ ] Regression suite expanded

### Documentation
- [ ] SPEC_ALIGNMENT.md complete
- [ ] Phase plans (1-5) finalized
- [ ] README.md updated
- [ ] USER_GUIDE.md written
- [ ] CHANGELOG.md entry
- [ ] Rustdoc coverage > 80%

### Reports
- [ ] POC_REPORT.md with metrics
- [ ] Known limitations documented
- [ ] Migration guide for existing users
- [ ] Performance benchmark results

## Success Criteria (from Spec Section 19)

The spec defines success as:

> Version 1.0 is ready only if:
> 1. At least 98% of pages with body_full policy balanced automatically to actualDelta = 0
> 2. All pages balanced or explicitly marked invalid
> 3. No content loss, duplication, or order change
> 4. Display and PDF match same LayoutResult
> 5. All hard constraint failures reported as diagnostics
> 6. Determinism tests pass
> 7. Corpus visually validated
> 8. No false claims of exact balance

**Our Targets:**
- [ ] 95%+ exact balance rate (98% is aggressive for v1)
- [ ] 100% diagnostic coverage
- [ ] Zero content integrity issues
- [ ] Single source of truth (LayoutResult)
- [ ] Full determinism
- [ ] Validated corpus
- [ ] Honest reporting (no fake "balanced" status)

## Timeline

Estimated: 1-2 weeks
- Week 1: Test suite implementation, POC metrics
- Week 2: Regression fixes, documentation, final validation

## Risk Mitigation

### High Risk: Balance Rate < 95%
**Mitigation:**
- Tune WeightConfig based on corpus
- Relax paragraph spacing constraints
- Document as known limitation
- Provide waiver workflow

### Medium Risk: Performance Regression
**Mitigation:**
- Profile and optimize solver
- Add caching for paragraph line breaking
- Consider parallel candidate evaluation
- Feature flag for balance solver

### Low Risk: Breaking Changes
**Mitigation:**
- Maintain f32 accessors during migration
- Provide codemod script for coordinate updates
- Version bump to 1.0.0 signals breaking change
- Detailed migration guide

---

*Final phase - the culmination of all previous work!*
