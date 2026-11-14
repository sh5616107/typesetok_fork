# Phase 1: Spec Alignment Infrastructure

## Overview

Phase 1 introduces foundational infrastructure for aligning TypesetOK with the formal specification document. This phase focuses on data structures, types, and contracts without implementing the actual balance measurement or solver logic (which are reserved for Phases 2 and 3).

## New Modules

### 1. `mpt.rs` - Integer Coordinate System

**Purpose**: Provides deterministic integer-based coordinates in millipoints (1/1000 pt).

**Key Types**:
- `Mpt(i32)`: Newtype wrapping an i32 representing millipoints
- Conversion traits: `From<f32>`, `Into<f32>`
- Arithmetic operations: `Add`, `Sub`, `Mul<i32>`, `Div<i32>`
- Serde support for serialization

**Example**:
```rust
use tok_typeset::Mpt;

let width = Mpt::from_pt(12.5);
assert_eq!(width.0, 12500);
assert_eq!(width.to_pt(), 12.5);

let sum = Mpt::from_pt(10.0) + Mpt::from_pt(5.0);
assert_eq!(sum.to_pt(), 15.0);
```

**Backward Compatibility**: 
- Existing `PhysicalPoint` and `PhysicalRect` retain f32 fields
- New methods: `from_mpt()` and `to_mpt()` for conversion
- Gradual migration strategy allows incremental adoption

---

### 2. `diagnostic.rs` - Diagnostic System

**Purpose**: Structured error reporting with 11 mandatory diagnostic codes from the specification.

**Key Types**:
- `DiagnosticCode`: Enum with 11 mandatory codes
  - `BalanceUnsatisfiable`
  - `SearchLimitReached`
  - `HardConstraintConflict`
  - `NotePlacementFailed`
  - `FontGlyphMissing`
  - `LineOverflow`
  - `PageOverflow`
  - `ManualBreakConflict`
  - `LayoutNotStable`
  - `ConfigInvalid`
  - `XrefTargetMissing`
- `DiagnosticSeverity`: `Info`, `Warning`, `Error`
- `Diagnostic`: Complete diagnostic message with context

**Example**:
```rust
use tok_typeset::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

let diag = Diagnostic::new(
    DiagnosticCode::BalanceUnsatisfiable,
    DiagnosticSeverity::Error,
    "Could not balance columns on page 5",
)
.with_page(5)
.with_strategy("Tried different column divisions")
.with_action("Consider adjusting paragraph spacing");
```

---

### 3. `page_policy.rs` - Page Policy System

**Purpose**: Defines 7 page policy types with specific balance requirements.

**Key Types**:
- `PagePolicy`: Enum with 7 policies
  - `BodyFull`: Regular page, targetDelta=0, no empty columns
  - `BodyPartial`: Partial page, configurable
  - `LastPage`: Balance not applicable by default
  - `ChapterOpening`: Per master page config
  - `PartOpening`: Not applicable by default
  - `TitlePage`: Not applicable
  - `ForcedBreak`: Inherits from original policy
- `PolicyConfig`: Configuration for each policy
- `PolicyResolver`: Resolves appropriate policy

**Example**:
```rust
use tok_typeset::{PolicyResolver, PagePolicy, Mpt};

let resolver = PolicyResolver::new();
let config = resolver.resolve(PagePolicy::BodyFull);

assert!(config.balance_applicable);
assert_eq!(config.target_delta, Mpt::ZERO);
assert!(config.is_satisfied(Mpt::from_pt(0.3))); // Within threshold
```

---

### 4. `waiver.rs` - Waiver System

**Purpose**: Manages approved exceptions to diagnostic rules.

**Key Types**:
- `WaiverScope`: `Page`, `Section`, `Document`
- `WaiverRecord`: Individual waiver with metadata
- `WaiverRegistry`: Collection of waivers with matching logic

**Example**:
```rust
use tok_typeset::{WaiverRecord, WaiverRegistry, WaiverScope, DiagnosticCode};

let mut registry = WaiverRegistry::new();

let waiver = WaiverRecord::new(
    "w-001",
    DiagnosticCode::FontGlyphMissing,
    "editor@example.com",
    "Rare marks not required",
    WaiverScope::Document,
);
registry.add(waiver);

assert!(registry.is_waived(DiagnosticCode::FontGlyphMissing, None, &[]));
```

---

### 5. `layout_result.rs` - LayoutResult Contract

**Purpose**: Complete layout representation matching specification section 12.

**Key Types**:
- `LayoutStatus`: `Valid`, `ValidWithWarnings`, `Invalid`
- `BalanceInfo`: Column balance information per page
- `ColumnLayout`: Individual column layout data
- `PageLayout`: Complete page layout including balance state
- `LayoutMetrics`: Summary statistics
- `DecisionTrace`: Optional debugging trace
- `LayoutResult`: Top-level result structure

**Example**:
```rust
use tok_typeset::{LayoutResult, PageLayout, BalanceInfo, Mpt};

let pages = vec![
    PageLayout {
        page_index: 0,
        balance: BalanceInfo::applicable(Mpt::ZERO, "body_full")
            .with_actual_delta(Mpt::from_pt(0.3), 0.5)
            .with_balanced(true),
        columns: vec![],
        dimensions: (Mpt::ZERO, Mpt::ZERO, Mpt::from_pt(595.0), Mpt::from_pt(842.0)),
        is_blank: false,
    },
];

let result = LayoutResult::new(
    "0.35.2",
    "config-hash",
    "doc-hash",
    pages,
    vec![],
);

assert_eq!(result.status, tok_typeset::LayoutStatus::Valid);
```

---

### 6. `config.rs` - Configuration and Versioning

**Purpose**: Engine configuration with deterministic hashing.

**Key Types**:
- `SearchLimits`: Algorithm limits (max_candidates, max_pages, lookback)
- `WeightConfig`: Soft scoring weights (placeholder values for Phase 3)
- `TypesetConfig`: Complete configuration
- `engine_version()`: Returns current engine version

**Example**:
```rust
use tok_typeset::{TypesetConfig, engine_version};

let config = TypesetConfig::default();
let hash = config.compute_hash();

println!("Engine version: {}", engine_version());
println!("Config hash: {}", hash);
```

---

## Integration Points

### Backward Compatibility

**Geometry Module Enhanced**:
- `PhysicalPoint` and `PhysicalRect` retain f32 fields
- New conversion methods: `from_mpt()` and `to_mpt()`
- Existing code continues to work unchanged

**Example Migration Path**:
```rust
use tok_typeset::{PhysicalRect, Mpt};

// Old code (still works)
let rect = PhysicalRect::a4_portrait();
let width_f32 = rect.width;

// New code (opt-in)
let (x, y, w, h) = rect.to_mpt();
let width_mpt: Mpt = w;

// Conversion back
let rect2 = PhysicalRect::from_mpt(x, y, w, h);
```

### Feature Flag

**Current Status**: All new modules are unconditionally available (no feature flag required).

**Rationale**: Phase 1 is non-breaking and provides pure infrastructure. Feature flags may be added in later phases for experimental balance solvers.

---

## Testing

### Unit Tests

All modules include comprehensive unit tests:

- **mpt**: 6 tests (arithmetic, conversion, serialization, range validation)
- **diagnostic**: 4 tests (builder pattern, serialization, code completeness)
- **page_policy**: 6 tests (all 7 policies, satisfaction logic, inheritance)
- **waiver**: 5 tests (matching logic for all scopes, registry)
- **layout_result**: 5 tests (status determination, metrics, serialization)
- **config**: 5 tests (defaults, hashing, versioning)

**Run all Phase 1 tests**:
```bash
cargo test --package tok-typeset --lib mpt diagnostic page_policy waiver layout_result config
```

### Integration Tests

Integration test suite in `tests/phase1_integration.rs`:

1. **test_create_layout_result_from_page_layout_box**: Convert existing PageLayoutBox to LayoutResult
2. **test_mpt_conversion_roundtrip**: Verify Mpt ↔ f32 conversion accuracy
3. **test_diagnostic_system_integration**: Create layout with diagnostics
4. **test_policy_configuration**: Policy resolution and satisfaction
5. **test_waiver_system_integration**: Waiver matching logic
6. **test_complete_layout_result_with_all_features**: Full serialization roundtrip

**Run integration tests**:
```bash
cargo test --package tok-typeset --test phase1_integration
```

---

## Non-Goals for Phase 1

These features are explicitly deferred to later phases:

❌ **Balance Measurement**: Measuring `lastBaselineY` and computing `bottomDelta` → Phase 2
❌ **Balance Solver**: 5-stage adjustment ladder → Phase 3
❌ **Continuous Flow**: Magazine-style column distribution → Phase 4
❌ **Acceptance Tests**: Specification section 16 tests → Phase 5

---

## Dependencies Added

- **chrono**: v0.4 (for ISO-8601 timestamps in waivers)

All other dependencies were already present in the workspace.

---

## Public API

All new types are exported from `tok-typeset` crate root:

```rust
pub use config::{
    engine_version, SearchLimits, TypesetConfig, WeightConfig, ENGINE_VERSION,
};
pub use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
pub use layout_result::{
    BalanceInfo, ColumnLayout, DecisionTrace, LayoutMetrics, LayoutResult, LayoutStatus,
    PageLayout,
};
pub use mpt::Mpt;
pub use page_policy::{PagePolicy, PolicyConfig, PolicyResolver};
pub use waiver::{WaiverRecord, WaiverRegistry, WaiverScope};
```

---

## Next Steps

**Phase 2** (Weeks 3-4): Measurement & Policy
- Measure `lastBaselineY` for each column in real layouts
- Calculate `bottomDelta` and `bottomDeltaLines`
- Implement page policy assignment based on document structure
- Add measurement tests from spec section 16.1 (tests 1-3)

**Phase 3** (Weeks 5-7): Balance Solver
- Implement 5-stage adjustment ladder
- Candidate generation and scoring
- Weighted soft score evaluation

---

## Documentation

All public APIs are documented with rustdoc. Generate documentation:

```bash
cargo doc --package tok-typeset --open
```

---

## Summary

Phase 1 successfully establishes:
✅ Integer coordinate system (Mpt)
✅ Complete LayoutResult contract
✅ 11 mandatory diagnostic codes
✅ 7 page policy types
✅ Waiver system
✅ Configuration and versioning
✅ Backward compatibility
✅ Comprehensive test coverage

All existing tests continue to pass. No breaking changes to public APIs.
