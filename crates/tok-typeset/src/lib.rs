//! # TypesetOK Typesetting Engine (`tok-typeset`)
//!
//! Core layout, geometric constraint solving, and global pagination engine for TypesetOK.
//!
//! ## Stages Architecture
//! - **Stage 7 — Geometric Constraint Solver**: Uses Cassowary linear simplex solver to
//!   model page boundaries, margins, columns, non-overlap, dynamic L-shapes, and footnote bands
//!   as explicit linear constraints without hardcoded geometry percentages.
//! - **Stage 8 — Global Pagination & Multi-Page Flow**: Implements lookahead page-breaking
//!   with cubic badness and TeX-style penalty optimization (`PageBreakModel`, `GlobalPageBreaker`),
//!   eliminating stranded headings (`keep_with_next`), orphan lines (`orphan_control`), and widow
//!   lines (`widow_control`). Dynamically paginates multi-page footnote apparatus bands
//!   (`DynamicFootnotePaginator`) and coordinates parallel commentary streams across facing spreads
//!   (`CoPaginationEngine`, `SpreadMathematics`).

#![allow(clippy::field_reassign_with_default)]

pub mod bidi;
pub mod co_pagination;
pub mod diagnostics;
pub mod drop_cap;
pub mod engine;
pub mod fixtures;
pub mod font;
pub mod footnote_paginator;
pub mod gematria;
pub mod geometry;
pub mod hebrew_justify;
pub mod hit_test;
pub mod incremental;
pub mod knuth_plass;
pub mod layout_cache;
pub mod layout_classifier;
pub mod layout_family;
pub mod layout_features;
pub mod layout_template;
pub mod masoretic;
pub mod multi_flow;
pub mod optical_margin;
pub mod page_break;
pub mod polygon;
pub mod polygon_engine;
pub mod polygon_flow;
pub mod shaper;
pub mod solver;
pub mod template_synthesizer;

// Phase 1 spec alignment modules
pub mod config;
pub mod diagnostic;
pub mod layout_result;
pub mod mpt;
pub mod page_policy;
pub mod waiver;

// Phase 2 spec alignment modules
pub mod balance_measurement;

// Phase 3 spec alignment modules
pub mod balance_solver;
pub mod candidate;
pub mod scoring;

// Phase 4 spec alignment modules
pub mod snake_column_breaker;

pub use bidi::{BidiEngine, BidiRun};
pub use co_pagination::{
    calculate_typographic_split, ActiveSyncResult, ActiveSynchronizer, AnchorCascadeEngine,
    AnchorKey, AnchorPoint, BalanceTargetMode, CancellationToken, CascadeLayoutResult,
    CascadeNodeInput, CascadeReason, CascadeViolation, CascadedNodeEvent, CoPaginatedChunk,
    CoPaginatedCommentary, CoPaginatedPage, CoPaginationDocument, CoPaginationEngine,
    CoPaginationResult, ColumnAllocation, ColumnBalanceReport, HeightBalanceConfig,
    NonBlockingPaginator, PaginationEvent, SharedColumnHeightBalancer, SpreadMathematics,
    SyncRemedy, SynchronizerConfig, TemplateConfig, TemplateKind,
};
pub use diagnostics::RecomputeReport;
pub use drop_cap::{DropCapLayout, HebrewDropCapShaper};
pub use engine::{TypesettingEngine, TypesettingEngineConfig};
pub use font::{FontData, FontManager, FontMetrics};
pub use footnote_paginator::{
    DynamicFootnotePaginator, FootnotePageSlice, FootnotePaginatorConfig,
};
pub use gematria::{GematriaEngine, HEBREW_GERESH, HEBREW_GERSHAYIM};
pub use geometry::{
    BreakToken, GlyphBox, LineBox, PageLayoutBox, PhysicalPoint, PhysicalRect, TextFrameBox,
};
pub use hebrew_justify::{
    HebrewJustifier, JustificationTier, JustifiedLine, VariableLineLayout, AHALTERM_LETTERS,
};
pub use hit_test::{HitTestResult, HitTester};
pub use incremental::{
    IncrementalBreakDiagnostics, IncrementalPageBreaker, IncrementalTypesetDiagnostics,
};
pub use knuth_plass::{
    BrokenLine, KnuthPlassBreaker, LastLineConstraint, LayoutItem, LineSpan, MeasureProfile,
};
pub use layout_cache::{
    CacheMetricsSnapshot, CachedParagraphLayout, ParagraphCacheKey, ParagraphLayoutCache,
};
pub use layout_classifier::{ClassificationConfidence, ClassificationResult, DocumentClassifier};
pub use layout_family::LayoutFamily;
pub use layout_features::{DocumentLayoutFeatures, FlowFeature};
pub use layout_template::{LayoutTemplate, TemplateConstraintError};
pub use masoretic::{
    has_rare_cantillation_or_masoretic_marks, is_poetic_emet_accent, is_rare_cantillation_mark,
    resolve_masoretic_letter_styling, MasoreticTypographyConfig, NUN_HAFUKHA,
};
pub use multi_flow::{
    DynamicPageResult, DynamicTalmudPageResult, FlowGeometrySpec, FlowPlacementRole,
    MultiFlowSolver, SolvedFlowAllocation, SpreadSide,
};
pub use optical_margin::{OpticalMarginAligner, OpticalProtrusion};
pub use page_break::{
    compute_badness, score_page_break, BreakOpportunity, GlobalPageBreaker, PageBreakPenalty,
    PageSlice, PaginationLineInput,
};
pub use polygon::{ExclusionZone, GeometricPolygon, ScanlineSpan};
pub use polygon_engine::PolygonFrame;
pub use polygon_flow::{
    CommentaryAbsorptionSpec, MultiColumnPolygonFlowSynthesizer, SynthesizedPolygonFlow,
};
pub use shaper::{PositionedGlyph, ShapedRun, TextShaper};
pub use solver::{
    AbsorptionKind, ColumnFrameVariables, ColumnSpec, ColumnWidthDistribution,
    CommentaryAbsorptionConfig, CommentaryAbsorptionVariables, ConstraintRelation,
    ConstraintStrength, ConstraintVariable, FootnoteBandConfig, FootnoteBandVariables,
    GeometricConstraint, GeometricConstraintSolver, LShapeConfig, LShapeExpansionVariables,
    LinearExpression, LinearTerm, MultiColumnConstraintModel, PageConstraintModel,
    PageGeometryConfig, RectVariables, SolvedColumnAllocation, SolvedRect, SolverError,
};
pub use template_synthesizer::TemplateSynthesizer;

// Phase 1 spec alignment exports
pub use config::{engine_version, SearchLimits, TypesetConfig, WeightConfig, ENGINE_VERSION};
pub use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
pub use layout_result::{
    BalanceInfo, ColumnLayout, DecisionTrace, LayoutMetrics, LayoutResult, LayoutStatus, PageLayout,
};
pub use mpt::Mpt;
pub use page_policy::{PagePolicy, PolicyConfig, PolicyResolver};
pub use waiver::{WaiverRecord, WaiverRegistry, WaiverScope};

// Phase 2 spec alignment exports
pub use balance_measurement::{BalanceError, BottomDelta, ColumnMeasurement};

// Phase 3 spec alignment exports
pub use balance_solver::{
    BalanceResult, BalanceSearchLimits, BalanceSolver, BalanceSolverError, LineData, SolverConfig,
};
pub use candidate::{BalanceCandidate, ColumnContent, Stage};
pub use scoring::{
    calculate_line_quality, calculate_para_spacing_deviation, calculate_word_spacing_quality,
    count_widow_orphan_violations, select_best_candidate,
};

// Phase 4 spec alignment exports
pub use snake_column_breaker::{
    DistributionMode, FlowMode, SectionColumnSpec, SnakeColumnBreaker, SnakeColumnConfig,
    SnakeColumnError, SnakeColumnResult,
};
