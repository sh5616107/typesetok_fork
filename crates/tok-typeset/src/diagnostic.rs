//! Diagnostic system for layout validation and reporting.
//!
//! Implements the 11 mandatory diagnostic codes from the specification (section 12.1),
//! providing structured error reporting with severity levels and remediation suggestions.

use serde::{Deserialize, Serialize};

/// Diagnostic code identifying the type of layout issue.
///
/// Corresponds to the 11 mandatory codes in specification section 12.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosticCode {
    /// Column balance could not be satisfied within search limits.
    BalanceUnsatisfiable,
    /// Search algorithm reached iteration or candidate limit.
    SearchLimitReached,
    /// Hard constraints cannot be simultaneously satisfied.
    HardConstraintConflict,
    /// Footnote or marginal note could not be placed on the required page.
    NotePlacementFailed,
    /// Required glyph missing from font.
    FontGlyphMissing,
    /// Line content overflows available width.
    LineOverflow,
    /// Content overflows page boundaries.
    PageOverflow,
    /// Manual page/column break conflicts with balance requirements.
    ManualBreakConflict,
    /// Layout did not stabilize after maximum reflow iterations.
    LayoutNotStable,
    /// Configuration parameters are invalid or inconsistent.
    ConfigInvalid,
    /// Cross-reference target does not exist.
    XrefTargetMissing,
}

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    /// Informational message (does not affect validity).
    Info,
    /// Warning that should be reviewed but does not prevent layout.
    Warning,
    /// Error that makes the layout invalid.
    Error,
}

/// A diagnostic message describing a layout issue.
///
/// Diagnostics provide structured information about problems encountered during
/// typesetting, including context (page, section, anchor) and suggested remedies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// The diagnostic code.
    pub code: DiagnosticCode,

    /// Severity level.
    pub severity: DiagnosticSeverity,

    /// Human-readable description of the issue.
    pub message: String,

    /// Zero-based page index where the issue occurred (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_index: Option<usize>,

    /// Section identifier where the issue occurred (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_id: Option<String>,

    /// Anchor IDs related to this diagnostic (e.g., cross-reference targets).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anchor_ids: Vec<String>,

    /// Strategies attempted to resolve the issue.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempted_strategies: Vec<String>,

    /// Suggested actions to resolve the issue.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggested_actions: Vec<String>,
}

impl Diagnostic {
    /// Create a new diagnostic with the given code and message.
    pub fn new(
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            page_index: None,
            section_id: None,
            anchor_ids: Vec::new(),
            attempted_strategies: Vec::new(),
            suggested_actions: Vec::new(),
        }
    }

    /// Builder: set the page index.
    pub fn with_page(mut self, page_index: usize) -> Self {
        self.page_index = Some(page_index);
        self
    }

    /// Builder: set the section ID.
    pub fn with_section(mut self, section_id: impl Into<String>) -> Self {
        self.section_id = Some(section_id.into());
        self
    }

    /// Builder: add an anchor ID.
    pub fn with_anchor(mut self, anchor_id: impl Into<String>) -> Self {
        self.anchor_ids.push(anchor_id.into());
        self
    }

    /// Builder: add multiple anchor IDs.
    pub fn with_anchors(mut self, anchor_ids: impl IntoIterator<Item = String>) -> Self {
        self.anchor_ids.extend(anchor_ids);
        self
    }

    /// Builder: add an attempted strategy.
    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.attempted_strategies.push(strategy.into());
        self
    }

    /// Builder: add multiple attempted strategies.
    pub fn with_strategies(mut self, strategies: impl IntoIterator<Item = String>) -> Self {
        self.attempted_strategies.extend(strategies);
        self
    }

    /// Builder: add a suggested action.
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.suggested_actions.push(action.into());
        self
    }

    /// Builder: add multiple suggested actions.
    pub fn with_actions(mut self, actions: impl IntoIterator<Item = String>) -> Self {
        self.suggested_actions.extend(actions);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder() {
        let diag = Diagnostic::new(
            DiagnosticCode::BalanceUnsatisfiable,
            DiagnosticSeverity::Error,
            "Could not balance columns",
        )
        .with_page(5)
        .with_section("chapter-3")
        .with_anchor("fn-ref-42")
        .with_strategy("Tried different column divisions")
        .with_action("Consider adjusting paragraph spacing");

        assert_eq!(diag.code, DiagnosticCode::BalanceUnsatisfiable);
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.page_index, Some(5));
        assert_eq!(diag.section_id, Some("chapter-3".to_string()));
        assert_eq!(diag.anchor_ids, vec!["fn-ref-42".to_string()]);
        assert_eq!(diag.attempted_strategies.len(), 1);
        assert_eq!(diag.suggested_actions.len(), 1);
    }

    #[test]
    fn test_diagnostic_code_serialization() {
        let code = DiagnosticCode::FontGlyphMissing;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, r#""FONT_GLYPH_MISSING""#);

        let deserialized: DiagnosticCode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, code);
    }

    #[test]
    fn test_all_diagnostic_codes_present() {
        // Ensure all 11 mandatory codes exist
        let codes = [
            DiagnosticCode::BalanceUnsatisfiable,
            DiagnosticCode::SearchLimitReached,
            DiagnosticCode::HardConstraintConflict,
            DiagnosticCode::NotePlacementFailed,
            DiagnosticCode::FontGlyphMissing,
            DiagnosticCode::LineOverflow,
            DiagnosticCode::PageOverflow,
            DiagnosticCode::ManualBreakConflict,
            DiagnosticCode::LayoutNotStable,
            DiagnosticCode::ConfigInvalid,
            DiagnosticCode::XrefTargetMissing,
        ];

        assert_eq!(codes.len(), 11, "Expected 11 mandatory diagnostic codes");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(DiagnosticSeverity::Info < DiagnosticSeverity::Warning);
        assert!(DiagnosticSeverity::Warning < DiagnosticSeverity::Error);
    }
}
