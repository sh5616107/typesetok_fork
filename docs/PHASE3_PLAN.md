# Phase 3: Balance Solver

## Prerequisites
- Phase 1: Infrastructure (Mpt, LayoutResult, Diagnostics, Policy)
- Phase 2: Measurement (BottomDelta calculation, policy assignment)

## Objectives

Implement the 5-stage adjustment ladder from spec section 10.4 to achieve `bottomDelta = 0`.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│           BalanceSolver (Main Entry)                │
│  solve_page_balance(content, policy, config)        │
└─────────────────────┬───────────────────────────────┘
                      │
         ┌────────────┼────────────┐
         │            │            │
         ▼            ▼            ▼
    ┌────────┐  ┌──────────┐  ┌─────────┐
    │Stage 1 │  │ Stage 2  │  │Stage 3  │ ...
    │Column  │  │Word      │  │Para     │
    │Division│  │Spacing   │  │Spacing  │
    └────────┘  └──────────┘  └─────────┘
         │            │            │
         └────────────┼────────────┘
                      ▼
            ┌──────────────────┐
            │ Candidate Pool   │
            │ Score & Select   │
            └──────────────────┘
```

## Core Components

### 1. BalanceSolver
**File:** `crates/tok-typeset/src/balance_solver.rs`

```rust
pub struct BalanceSolver {
    config: SolverConfig,
    search_limits: SearchLimits,
}

pub struct SolverConfig {
    pub word_spacing_range: (f32, f32),      // e.g., (0.85, 1.40)
    pub para_spacing_grid: Vec<Mpt>,         // e.g., [0, L, 2L, 3L]
    pub looseness_allowed: bool,
    pub enable_neighbor_search: bool,
}

pub struct SearchLimits {
    pub max_candidates_per_stage: usize,     // e.g., 50
    pub max_total_candidates: usize,         // e.g., 200
    pub max_neighbor_pages: usize,           // e.g., 2
}

impl BalanceSolver {
    pub fn solve(
        &self,
        content: &[LineBox],
        available_height: Mpt,
        num_columns: usize,
        policy: &PagePolicy,
    ) -> BalanceResult {
        let mut candidates = Vec::new();
        
        // Stage 1: Column division variations
        candidates.extend(self.stage1_column_divisions(content, ...)?);
        
        // Early filter: remove hard constraint violations
        candidates.retain(|c| c.satisfies_hard_constraints());
        
        // Check if we have exact balance
        if let Some(exact) = candidates.iter().find(|c| c.bottom_delta() == Mpt(0)) {
            return BalanceResult::Exact(exact.clone());
        }
        
        // Stage 2: Word spacing + looseness
        if candidates.is_empty() || self.should_try_stage2(&candidates) {
            candidates.extend(self.stage2_word_spacing_looseness(...)?);
        }
        
        // Stage 3: Paragraph spacing
        if !self.found_acceptable(&candidates, policy) {
            candidates.extend(self.stage3_paragraph_spacing(...)?);
        }
        
        // Stage 4: Neighbor cut point transfer (if enabled)
        if self.config.enable_neighbor_search && !self.found_acceptable(&candidates, policy) {
            candidates.extend(self.stage4_neighbor_search(...)?);
        }
        
        // Select best candidate by weighted soft score
        if let Some(best) = self.select_best_candidate(&candidates, policy) {
            BalanceResult::Satisfiable(best)
        } else {
            BalanceResult::Unsatisfiable {
                attempted_stages: vec!["stage1", "stage2", "stage3"],
                best_attempt: candidates.iter().min_by_key(|c| c.bottom_delta()),
            }
        }
    }
}
```

### 2. Candidate Structure
**File:** `crates/tok-typeset/src/candidate.rs`

```rust
#[derive(Clone, Debug)]
pub struct BalanceCandidate {
    pub columns: Vec<ColumnContent>,
    pub bottom_delta: Mpt,
    pub delta_lines: f32,
    
    // Soft quality metrics
    pub word_spacing_quality: f32,
    pub line_quality_score: f32,
    pub widow_orphan_violations: usize,
    pub para_spacing_deviation: f32,
    pub change_cost: f32,  // vs previous layout
    
    // Metadata
    pub stage: Stage,
    pub strategy: String,  // e.g., "word_spacing:0.92,looseness:-1"
}

impl BalanceCandidate {
    pub fn satisfies_hard_constraints(&self) -> bool {
        // No text overflow
        // All lines in order
        // Footnotes with reference
        // Keep rules honored
        // Manual overrides respected
        true  // placeholder
    }
    
    pub fn bottom_delta(&self) -> Mpt {
        self.bottom_delta
    }
    
    pub fn soft_score(&self, weights: &WeightConfig) -> f32 {
        weights.word_spacing * self.word_spacing_quality
            + weights.line_quality * self.line_quality_score
            + weights.widow_orphan * self.widow_orphan_violations as f32
            + weights.para_spacing * self.para_spacing_deviation
            + weights.change_cost * self.change_cost
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Stage {
    ColumnDivision,
    WordSpacingLooseness,
    ParagraphSpacing,
    NeighborSearch,
}
```

### 3. Stage Implementations

#### Stage 1: Column Division
```rust
impl BalanceSolver {
    fn stage1_column_divisions(
        &self,
        lines: &[LineBox],
        num_columns: usize,
    ) -> Result<Vec<BalanceCandidate>, SolverError> {
        let mut candidates = Vec::new();
        let total_lines = lines.len();
        
        // Try divisions around natural split point
        let base_split = total_lines / num_columns;
        
        for offset in -2..=2 {
            let split = (base_split as i32 + offset).max(0) as usize;
            if split < total_lines {
                let candidate = self.create_candidate_from_split(lines, split)?;
                candidates.push(candidate);
            }
        }
        
        Ok(candidates)
    }
}
```

#### Stage 2: Word Spacing + Looseness
```rust
impl BalanceSolver {
    fn stage2_word_spacing_looseness(
        &self,
        base_content: &[LineBox],
        paragraph_id: NodeId,
    ) -> Result<Vec<BalanceCandidate>, SolverError> {
        let mut candidates = Vec::new();
        
        // Sample word spacing values in range
        let spacing_samples = self.sample_spacing_range(5);
        
        for spacing in spacing_samples {
            // Try neutral looseness
            if let Ok(lines) = self.rebreak_paragraph(paragraph_id, spacing, 0) {
                candidates.push(self.create_candidate(lines, ...)?);
            }
            
            // Try looseness +1/-1 if allowed
            if self.config.looseness_allowed {
                for looseness in [-1, 1] {
                    if let Ok(lines) = self.rebreak_paragraph(paragraph_id, spacing, looseness) {
                        candidates.push(self.create_candidate(lines, ...)?);
                    }
                }
            }
        }
        
        Ok(candidates)
    }
    
    fn sample_spacing_range(&self, samples: usize) -> Vec<f32> {
        let (min, max) = self.config.word_spacing_range;
        (0..samples)
            .map(|i| min + (max - min) * i as f32 / (samples - 1) as f32)
            .collect()
    }
}
```

#### Stage 3: Paragraph Spacing
```rust
impl BalanceSolver {
    fn stage3_paragraph_spacing(
        &self,
        base_candidate: &BalanceCandidate,
        style: &ParagraphStyle,
    ) -> Result<Vec<BalanceCandidate>, SolverError> {
        let mut candidates = Vec::new();
        
        // Only if paragraph spacing is defined and non-zero
        if style.space_before_pt == 0.0 && style.space_after_pt == 0.0 {
            return Ok(candidates);  // Stage not applicable
        }
        
        // Try adding/removing multiples of baseline grid
        for spacing_delta in &self.config.para_spacing_grid {
            let modified = self.adjust_paragraph_spacing(
                base_candidate,
                *spacing_delta,
            )?;
            candidates.push(modified);
        }
        
        Ok(candidates)
    }
}
```

### 4. Scoring System
**File:** `crates/tok-typeset/src/scoring.rs`

```rust
pub struct WeightConfig {
    pub word_spacing: f32,
    pub line_quality: f32,
    pub widow_orphan: f32,
    pub para_spacing: f32,
    pub change_cost: f32,
    pub page_count: f32,
    // Sum should be 1.0 for normalized scoring
}

impl Default for WeightConfig {
    fn default() -> Self {
        Self {
            word_spacing: 0.25,
            line_quality: 0.25,
            widow_orphan: 0.20,
            para_spacing: 0.15,
            change_cost: 0.10,
            page_count: 0.05,
        }
    }
}

impl BalanceSolver {
    fn select_best_candidate(
        &self,
        candidates: &[BalanceCandidate],
        policy: &PagePolicy,
    ) -> Option<BalanceCandidate> {
        // Filter by balance threshold
        let acceptable: Vec<_> = candidates
            .iter()
            .filter(|c| c.bottom_delta <= policy.success_threshold())
            .collect();
        
        if acceptable.is_empty() {
            return None;
        }
        
        // Among acceptable, select by soft score
        acceptable
            .into_iter()
            .min_by(|a, b| {
                let score_a = a.soft_score(&self.weights);
                let score_b = b.soft_score(&self.weights);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }
}
```

## Integration with Engine

Update `crates/tok-typeset/src/engine.rs`:

```rust
impl TypesettingEngine {
    pub fn typeset_document_with_balance(&self, doc: &DocumentRoot) -> LayoutResult {
        let solver = BalanceSolver::new(self.config.balance_config.clone());
        
        // ... existing line breaking and column layout ...
        
        // NEW: Invoke balance solver for body_full pages
        if page_policy.balance_applicable() {
            match solver.solve(lines, available_height, num_columns, &page_policy) {
                BalanceResult::Exact(candidate) => {
                    columns = candidate.columns;
                    balance_info.balanced = true;
                    balance_info.actual_delta = Some(Mpt(0));
                }
                BalanceResult::Satisfiable(candidate) => {
                    columns = candidate.columns;
                    balance_info.balanced = true;
                    balance_info.actual_delta = Some(candidate.bottom_delta);
                }
                BalanceResult::Unsatisfiable { attempted_stages, best_attempt } => {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticCode::BalanceUnsatisfiable,
                        Severity::Error,
                    )
                    .with_attempted_strategies(attempted_stages)
                    .with_suggestion("Enable paragraph spacing or use waiver"));
                    
                    result.status = LayoutStatus::Invalid;
                }
            }
        }
        
        // ...
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_stage1_generates_variations() {
    let solver = BalanceSolver::new(...);
    let lines = create_test_lines(20);
    let candidates = solver.stage1_column_divisions(&lines, 2).unwrap();
    
    assert!(candidates.len() >= 5);  // At least -2..+2 around midpoint
}

#[test]
fn test_stage2_word_spacing_samples() {
    let solver = BalanceSolver::new(...);
    let samples = solver.sample_spacing_range(5);
    
    assert_eq!(samples.len(), 5);
    assert_eq!(samples[0], 0.85);
    assert_eq!(samples[4], 1.40);
}

#[test]
fn test_candidate_hard_constraint_filter() {
    let candidate = create_overflowing_candidate();
    assert!(!candidate.satisfies_hard_constraints());
}
```

### Integration Tests (Spec Section 16.1)

```rust
#[test]
fn test_spec_16_1_test_4_heading_with_next() {
    // Heading near bottom must keep with following lines
}

#[test]
fn test_spec_16_1_test_5_last_page_sparse() {
    // Last page policy applied, not forced to balance
}

#[test]
fn test_spec_16_1_test_6_long_footnote() {
    // Footnote area reduces both columns equally
}
```

## Performance Considerations

- **Limit candidate explosion:** max_candidates_per_stage
- **Early termination:** Stop at exact balance (delta = 0)
- **Caching:** Memoize paragraph line breaking with same spacing
- **Determinism:** Fixed sampling, sorted iteration order

## Non-Goals for Phase 3

- ❌ Continuous flow (Phase 4)
- ❌ Full POC corpus (Phase 5)

## Success Criteria

- [ ] All 5 stages implemented
- [ ] Exact balance achieved for test case 1
- [ ] Diagnostic with attempted strategies for test case 2
- [ ] Weighted soft score selection working
- [ ] Stage 3 respects paragraph spacing availability
- [ ] Spec tests 4-8 pass
- [ ] No regression in existing tests
- [ ] Performance within 2x of current (acceptable for phase 3)

## Timeline

Estimated: 2-3 weeks (6-10 work sessions)

---

*This is the most complex phase. Break it into sub-phases if needed.*
