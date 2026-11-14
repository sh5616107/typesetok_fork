# Spec Alignment Project

## Goal
Align TypesetOK implementation with formal specification document (0-5.md) for professional Hebrew typesetting and page balancing.

## Status: Phase 1 - Infrastructure (In Progress)

### Overview
This multi-phase project adapts TypesetOK to match the requirements of a formal typesetting specification focused on:
- Deterministic integer coordinate system (mpt)
- Mandatory column balance with `bottomDelta = 0`
- Page policy system with 7 policy types
- Comprehensive diagnostic and waiver system
- Full LayoutResult contract

### Phases

#### Phase 1: Infrastructure ✅ (Weeks 1-2) - IN PROGRESS
**Branch:** `feature/spec-alignment-phase1`

**Objectives:**
1. Integer coordinate system (Mpt type)
2. LayoutResult contract (sections 12 & 13 of spec)
3. Diagnostic codes (11 mandatory codes)
4. Page policy system (7 policy types)
5. Waiver system
6. Configuration and versioning

**Status:**
- [x] Branch created
- [x] Mpt type implemented
- [x] LayoutResult structure defined
- [x] Diagnostic system complete
- [x] Page policy system
- [x] Waiver system
- [x] Tests passing

**Non-Goals for Phase 1:**
- Actual balance measurement (Phase 2)
- Balance solver (Phase 2)
- Continuous flow column splitting (Phase 3)

---

#### Phase 2: Measurement & Policy (Weeks 3-4) - PLANNED
**Objectives:**
1. Measure `lastBaselineY` for each column
2. Calculate `bottomDelta` and `bottomDeltaLines`
3. Implement page policy assignment
4. Basic balance validation tests

**Key Files:**
- `crates/tok-typeset/src/balance_measurement.rs`
- Extend `engine.rs` with measurement hooks
- Add tests from spec section 16.1

---

#### Phase 3: Balance Solver (Weeks 5-7) - PLANNED
**Objectives:**
1. Implement 5-stage adjustment ladder (spec section 7.5):
   - Stage 1: Different column divisions
   - Stage 2: Word spacing + looseness
   - Stage 3: Paragraph spacing
   - Stage 4: Cut point transfer to neighbors
   - Stage 5: Diagnostic if no solution
2. Candidate generation and scoring
3. Weighted soft score evaluation (section 9)

**Key Files:**
- `crates/tok-typeset/src/balance_solver.rs`
- `crates/tok-typeset/src/candidate.rs`
- `crates/tok-typeset/src/scoring.rs`

---

#### Phase 4: Continuous Flow (Weeks 8-9) - PLANNED
**Objectives:**
1. Single flow distributed across N columns (magazine layout)
2. SnakeColumnBreaker for line-to-column distribution
3. Sequential vs balanced column filling
4. Integration with balance solver

**Key Files:**
- `crates/tok-typeset/src/snake_column_breaker.rs`
- Extend `ParagraphStyle` with `column_count` and `column_gutter_pt`

---

#### Phase 5: Integration & Testing (Weeks 10-11) - PLANNED
**Objectives:**
1. Run all 16 acceptance tests from spec section 16
2. POC report with metrics (section 27)
3. Fix regressions
4. Documentation

---

## Design Decisions

### Coordinate System: Mpt (Millipoints)
**Spec requirement:** All coordinates as integers in millipoints (1/1000 pt).

**Implementation:**
```rust
pub struct Mpt(i32);

impl Mpt {
    pub fn from_pt(pt: f32) -> Self { Mpt((pt * 1000.0).round() as i32) }
    pub fn to_pt(self) -> f32 { self.0 as f32 / 1000.0 }
}
```

**Migration strategy:**
- Phase 1: Add Mpt type, use internally in new structures
- Backward compat: Keep f32 accessors on existing types
- Gradual migration: Update callsites incrementally

### LayoutResult Contract
**Spec requirement:** Single source of truth for both screen and PDF.

**Structure:**
```rust
pub struct LayoutResult {
    pub engine_version: String,
    pub config_hash: String,
    pub status: LayoutStatus,
    pub pages: Vec<PageLayout>,
    pub diagnostics: Vec<Diagnostic>,
    // ...
}

pub struct PageLayout {
    pub balance: BalanceInfo,
    // ...
}
```

### Page Policy System
**Spec requirement:** Explicit policy per page type, not inferred from content.

**Policies:**
1. `body_full`: Regular full page, `targetDelta = 0`, no empty columns
2. `body_partial`: Partial page, configurable balance
3. `last_page`: Balance not applicable by default
4. `chapter_opening`: Per master page definition
5. `part_opening`: Not applicable by default
6. `title_page`: Not applicable
7. `forced_break`: Inherits from original policy

## Testing Strategy

### Phase 1 Tests
- [x] Unit: Mpt arithmetic
- [x] Unit: Diagnostic code enum
- [x] Unit: Policy resolution
- [ ] Integration: Create LayoutResult from PageLayoutBox
- [ ] Regression: All 699 existing tests pass

### Phase 2 Tests
- [ ] Balance measurement accuracy
- [ ] Policy assignment correctness
- [ ] Spec section 16.1 tests 1-3

### Phase 3 Tests
- [ ] Candidate generation determinism
- [ ] 5-stage ladder coverage
- [ ] Spec section 16.1 tests 4-8

## Open Questions

From spec section 26 (require explicit decision):
- [ ] Search limit defaults (max_candidates, max_dp_states)
- [ ] Default word spacing range (e.g., 85%-140%)
- [ ] Default paragraph spacing range
- [ ] Last page policy default
- [ ] Waiver approval workflow

## References

- Spec document: `0-5.md` (version 0.5, October 2026)
- Key sections:
  - Section 2: Principles (integer coordinates, determinism)
  - Section 7.5: Balance policy table
  - Section 8: Definition of "columns end equally"
  - Section 10: Mandatory balance algorithm
  - Section 12: LayoutResult contract
  - Section 16: Acceptance tests

## Progress Tracking

| Phase | Status | Start | End | Notes |
|-------|--------|-------|-----|-------|
| Phase 1 | 🟡 In Progress | 2026-11-15 | - | Workflow running, coder active |
| Phase 2 | 📋 Planned | - | - | PHASE2_PLAN.md ready |
| Phase 3 | 📋 Planned | - | - | PHASE3_PLAN.md ready |
| Phase 4 | 📋 Planned | - | - | PHASE4_PLAN.md ready |
| Phase 5 | 📋 Planned | - | - | PHASE5_PLAN.md ready |

## Documentation

- [x] SPEC_ALIGNMENT.md - Main tracking document
- [x] docs/PHASE2_PLAN.md - Balance measurement & policy
- [x] docs/PHASE3_PLAN.md - Balance solver (5-stage ladder)
- [x] docs/PHASE4_PLAN.md - Continuous flow columns
- [x] docs/PHASE5_PLAN.md - Integration & POC validation

---

*Last updated: 2026-11-15*
