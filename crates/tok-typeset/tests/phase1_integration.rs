//! Integration tests for Phase 1 spec alignment infrastructure.

use tok_typeset::{
    BalanceInfo, ColumnLayout, LayoutResult, Mpt, PageLayout, PageLayoutBox, PhysicalRect,
};

#[test]
fn test_create_layout_result_from_page_layout_box() {
    // Create a simple PageLayoutBox (existing structure)
    let page_box = PageLayoutBox::new(
        0,
        "א".to_string(),
        PhysicalRect::a4_portrait(),
        vec![],
        None,
    );

    // Convert to new LayoutResult structure
    let (x_mpt, y_mpt, width_mpt, height_mpt) = page_box.dimensions.to_mpt();

    let page_layout = PageLayout {
        page_index: page_box.page_index,
        balance: BalanceInfo::not_applicable("body_full"),
        columns: vec![ColumnLayout {
            column_index: 0,
            last_baseline_y: None,
            height: height_mpt,
            is_empty: page_box.frames.is_empty(),
        }],
        dimensions: (x_mpt, y_mpt, width_mpt, height_mpt),
        is_blank: page_box.is_blank,
    };

    let layout_result = LayoutResult::new(
        "0.35.2",
        "test-config-hash",
        "test-doc-hash",
        vec![page_layout],
        vec![],
    );

    // Verify the result
    assert_eq!(layout_result.pages.len(), 1);
    assert_eq!(layout_result.status, tok_typeset::LayoutStatus::Valid);
    assert_eq!(layout_result.metrics.page_count, 1);
    assert_eq!(layout_result.metrics.error_count, 0);

    // Verify serialization works
    let json = serde_json::to_string(&layout_result).expect("Serialization should succeed");
    let deserialized: LayoutResult =
        serde_json::from_str(&json).expect("Deserialization should succeed");

    assert_eq!(deserialized.pages.len(), 1);
    assert_eq!(
        deserialized.pages[0].page_index,
        layout_result.pages[0].page_index
    );
}

#[test]
fn test_mpt_conversion_roundtrip() {
    let original_rect = PhysicalRect::a4_portrait();
    let (x, y, w, h) = original_rect.to_mpt();

    // Verify conversion is accurate
    assert_eq!(x.to_pt(), original_rect.x);
    assert_eq!(y.to_pt(), original_rect.y);
    assert!((w.to_pt() - original_rect.width).abs() < 0.01);
    assert!((h.to_pt() - original_rect.height).abs() < 0.01);

    // Verify we can convert back
    let converted = PhysicalRect::from_mpt(x, y, w, h);
    assert_eq!(converted.x, original_rect.x);
    assert_eq!(converted.y, original_rect.y);
}

#[test]
fn test_diagnostic_system_integration() {
    use tok_typeset::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

    let diagnostic = Diagnostic::new(
        DiagnosticCode::BalanceUnsatisfiable,
        DiagnosticSeverity::Error,
        "Could not balance columns on page 5",
    )
    .with_page(5)
    .with_strategy("Tried different column divisions")
    .with_strategy("Adjusted word spacing")
    .with_action("Consider adjusting paragraph spacing")
    .with_action("Review manual break placements");

    // Create a layout result with this diagnostic
    let layout_result = LayoutResult::new("0.35.2", "hash", "doc", vec![], vec![diagnostic]);

    assert_eq!(
        layout_result.status,
        tok_typeset::LayoutStatus::Invalid // Has error
    );
    assert_eq!(layout_result.diagnostics.len(), 1);
    assert_eq!(layout_result.metrics.error_count, 1);
}

#[test]
fn test_policy_configuration() {
    use tok_typeset::PolicyResolver;

    let resolver = PolicyResolver::new();
    let body_full = resolver.resolve(tok_typeset::PagePolicy::BodyFull);

    assert!(body_full.balance_applicable);
    assert_eq!(body_full.target_delta, Mpt::ZERO);
    assert!(!body_full.empty_column_allowed);

    // Test satisfaction
    assert!(body_full.is_satisfied(Mpt::ZERO));
    assert!(body_full.is_satisfied(Mpt::from_pt(0.4)));
    assert!(!body_full.is_satisfied(Mpt::from_pt(1.0)));
}

#[test]
fn test_waiver_system_integration() {
    use tok_typeset::{
        Diagnostic, DiagnosticCode, DiagnosticSeverity, WaiverRecord, WaiverRegistry, WaiverScope,
    };

    let mut registry = WaiverRegistry::new();

    // Create a document-wide waiver for glyph missing
    let waiver = WaiverRecord::new(
        "w-001",
        DiagnosticCode::FontGlyphMissing,
        "editor@example.com",
        "Rare cantillation marks not required",
        WaiverScope::Document,
    );
    registry.add(waiver);

    // Check if a diagnostic is waived
    let diagnostic = Diagnostic::new(
        DiagnosticCode::FontGlyphMissing,
        DiagnosticSeverity::Warning,
        "Missing glyph U+0599",
    )
    .with_page(10);

    assert!(registry.is_waived(diagnostic.code, diagnostic.page_index, &[]));
}

#[test]
fn test_complete_layout_result_with_all_features() {
    use tok_typeset::{
        BalanceInfo, ColumnLayout, DecisionTrace, Diagnostic, DiagnosticCode, DiagnosticSeverity,
        LayoutResult, Mpt, PageLayout, WaiverRecord, WaiverScope,
    };

    // Create a multi-page layout result with diagnostics, trace, and waivers
    let pages = vec![
        PageLayout {
            page_index: 0,
            balance: BalanceInfo::applicable(Mpt::ZERO, "body_full")
                .with_actual_delta(Mpt::from_pt(0.3), 0.5)
                .with_balanced(true),
            columns: vec![
                ColumnLayout {
                    column_index: 0,
                    last_baseline_y: Some(Mpt::from_pt(800.0)),
                    height: Mpt::from_pt(820.0),
                    is_empty: false,
                },
                ColumnLayout {
                    column_index: 1,
                    last_baseline_y: Some(Mpt::from_pt(799.7)),
                    height: Mpt::from_pt(820.0),
                    is_empty: false,
                },
            ],
            dimensions: (
                Mpt::ZERO,
                Mpt::ZERO,
                Mpt::from_pt(595.28),
                Mpt::from_pt(841.89),
            ),
            is_blank: false,
        },
        PageLayout {
            page_index: 1,
            balance: BalanceInfo::not_applicable("last_page"),
            columns: vec![ColumnLayout {
                column_index: 0,
                last_baseline_y: Some(Mpt::from_pt(400.0)),
                height: Mpt::from_pt(820.0),
                is_empty: false,
            }],
            dimensions: (
                Mpt::ZERO,
                Mpt::ZERO,
                Mpt::from_pt(595.28),
                Mpt::from_pt(841.89),
            ),
            is_blank: false,
        },
    ];

    let diagnostics = vec![Diagnostic::new(
        DiagnosticCode::LineOverflow,
        DiagnosticSeverity::Warning,
        "Minor overflow on page 1",
    )
    .with_page(1)];

    let trace = vec![
        DecisionTrace {
            stage: "balance_solver".to_string(),
            decision: "Adjusted column division".to_string(),
            context: vec![("page".to_string(), "0".to_string())],
        },
        DecisionTrace {
            stage: "page_breaker".to_string(),
            decision: "Placed page break after paragraph 42".to_string(),
            context: vec![],
        },
    ];

    let waivers = vec![WaiverRecord::new(
        "w-001",
        DiagnosticCode::LineOverflow,
        "editor",
        "Acceptable overflow",
        WaiverScope::Page,
    )
    .with_page(1)];

    let result = LayoutResult::new("0.35.2", "config-hash", "doc-hash", pages, diagnostics)
        .with_trace(trace)
        .with_waivers(waivers);

    // Verify structure
    assert_eq!(result.pages.len(), 2);
    assert_eq!(
        result.status,
        tok_typeset::LayoutStatus::ValidWithWarnings
    );
    assert_eq!(result.metrics.page_count, 2);
    assert_eq!(result.metrics.balanced_page_count, 1);
    assert_eq!(result.metrics.satisfied_balance_count, 1);
    assert!(result.trace.is_some());
    assert_eq!(result.trace.as_ref().unwrap().len(), 2);
    assert!(result.waivers.is_some());
    assert_eq!(result.waivers.as_ref().unwrap().len(), 1);

    // Verify serialization
    let json = serde_json::to_string_pretty(&result).expect("Serialization should succeed");
    assert!(json.contains("body_full"));
    assert!(json.contains("balance_solver"));
    assert!(json.contains("LINE_OVERFLOW"));

    let deserialized: LayoutResult =
        serde_json::from_str(&json).expect("Deserialization should succeed");
    assert_eq!(deserialized.pages.len(), 2);
}
