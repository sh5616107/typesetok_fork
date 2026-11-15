//! LayoutResult contract for complete layout representation.
//!
//! Implements the full LayoutResult structure from specification section 12,
//! providing a single source of truth for layout computation results.

use crate::diagnostic::Diagnostic;
use crate::mpt::Mpt;
use serde::{Deserialize, Serialize};

/// Overall status of the layout result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutStatus {
    /// Layout is valid and meets all requirements.
    Valid,

    /// Layout is valid but has warnings that should be reviewed.
    ValidWithWarnings,

    /// Layout is invalid due to unsatisfiable constraints or errors.
    Invalid,
}

/// Information about column balance on a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceInfo {
    /// Whether balance is applicable for this page.
    pub applicable: bool,

    /// Target bottom delta (0 for balanced pages).
    pub target_delta: Mpt,

    /// Actual bottom delta achieved (None if not computed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_delta: Option<Mpt>,

    /// Actual delta expressed in lines (for diagnostics).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_delta_lines: Option<f32>,

    /// Whether the balance requirement was satisfied.
    pub balanced: bool,

    /// Policy ID that determined this page's balance requirements.
    pub policy_id: String,
}

impl BalanceInfo {
    /// Create a new balance info for a page where balance is not applicable.
    pub fn not_applicable(policy_id: impl Into<String>) -> Self {
        Self {
            applicable: false,
            target_delta: Mpt::ZERO,
            actual_delta: None,
            actual_delta_lines: None,
            balanced: true, // Automatically satisfied when not applicable
            policy_id: policy_id.into(),
        }
    }

    /// Create a new balance info for a page where balance is applicable.
    pub fn applicable(target_delta: Mpt, policy_id: impl Into<String>) -> Self {
        Self {
            applicable: true,
            target_delta,
            actual_delta: None,
            actual_delta_lines: None,
            balanced: false,
            policy_id: policy_id.into(),
        }
    }

    /// Set the actual delta achieved.
    pub fn with_actual_delta(mut self, actual_delta: Mpt, actual_delta_lines: f32) -> Self {
        self.actual_delta = Some(actual_delta);
        self.actual_delta_lines = Some(actual_delta_lines);
        self
    }

    /// Mark whether the balance was satisfied.
    pub fn with_balanced(mut self, balanced: bool) -> Self {
        self.balanced = balanced;
        self
    }
}

/// Layout information for a single column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnLayout {
    /// Column index within the page (0-based).
    pub column_index: usize,

    /// Y-coordinate of the last baseline in this column (in mpt).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_baseline_y: Option<Mpt>,

    /// Height of the column (in mpt).
    pub height: Mpt,

    /// Whether this column is empty.
    pub is_empty: bool,
}

/// Layout information for a single page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageLayout {
    /// Zero-based page index.
    pub page_index: usize,

    /// Balance information for this page.
    pub balance: BalanceInfo,

    /// Layout information for each column on this page.
    pub columns: Vec<ColumnLayout>,

    /// Page dimensions (x, y, width, height in mpt).
    pub dimensions: (Mpt, Mpt, Mpt, Mpt),

    /// Whether this is a blank page (e.g., verso forcing page).
    #[serde(default)]
    pub is_blank: bool,
}

/// Overall layout metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutMetrics {
    /// Total number of pages.
    pub page_count: usize,

    /// Number of pages where balance is applicable.
    pub balanced_page_count: usize,

    /// Number of pages where balance was satisfied.
    pub satisfied_balance_count: usize,

    /// Total number of diagnostics.
    pub diagnostic_count: usize,

    /// Number of error-level diagnostics.
    pub error_count: usize,

    /// Number of warning-level diagnostics.
    pub warning_count: usize,
}

impl LayoutMetrics {
    /// Create metrics from a layout result.
    pub fn from_layout(pages: &[PageLayout], diagnostics: &[Diagnostic]) -> Self {
        let balanced_page_count = pages.iter().filter(|p| p.balance.applicable).count();
        let satisfied_balance_count = pages
            .iter()
            .filter(|p| p.balance.applicable && p.balance.balanced)
            .count();

        let error_count = diagnostics
            .iter()
            .filter(|d| d.severity == crate::diagnostic::DiagnosticSeverity::Error)
            .count();
        let warning_count = diagnostics
            .iter()
            .filter(|d| d.severity == crate::diagnostic::DiagnosticSeverity::Warning)
            .count();

        Self {
            page_count: pages.len(),
            balanced_page_count,
            satisfied_balance_count,
            diagnostic_count: diagnostics.len(),
            error_count,
            warning_count,
        }
    }
}

/// A trace entry recording a decision made during layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionTrace {
    /// Stage or component that made the decision.
    pub stage: String,

    /// Description of the decision.
    pub decision: String,

    /// Context information (page, column, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context: Vec<(String, String)>,
}

/// Complete layout result matching specification section 12.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutResult {
    /// Engine version that produced this result.
    pub engine_version: String,

    /// Hash of the configuration used.
    pub config_hash: String,

    /// Hash of the document content.
    pub document_hash: String,

    /// Overall status of the layout.
    pub status: LayoutStatus,

    /// Layout information for each page.
    pub pages: Vec<PageLayout>,

    /// Diagnostics generated during layout.
    pub diagnostics: Vec<Diagnostic>,

    /// Overall layout metrics.
    pub metrics: LayoutMetrics,

    /// Optional decision trace for debugging.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Vec<DecisionTrace>>,

    /// Optional waiver records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waivers: Option<Vec<crate::waiver::WaiverRecord>>,
}

impl LayoutResult {
    /// Create a new layout result.
    pub fn new(
        engine_version: impl Into<String>,
        config_hash: impl Into<String>,
        document_hash: impl Into<String>,
        pages: Vec<PageLayout>,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        let metrics = LayoutMetrics::from_layout(&pages, &diagnostics);

        // Determine overall status
        let status = if metrics.error_count > 0 {
            LayoutStatus::Invalid
        } else if metrics.warning_count > 0 {
            LayoutStatus::ValidWithWarnings
        } else {
            LayoutStatus::Valid
        };

        Self {
            engine_version: engine_version.into(),
            config_hash: config_hash.into(),
            document_hash: document_hash.into(),
            status,
            pages,
            diagnostics,
            metrics,
            trace: None,
            waivers: None,
        }
    }

    /// Add decision trace to the result.
    pub fn with_trace(mut self, trace: Vec<DecisionTrace>) -> Self {
        self.trace = Some(trace);
        self
    }

    /// Add waivers to the result.
    pub fn with_waivers(mut self, waivers: Vec<crate::waiver::WaiverRecord>) -> Self {
        self.waivers = Some(waivers);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

    #[test]
    fn test_balance_info_not_applicable() {
        let info = BalanceInfo::not_applicable("last_page");
        assert!(!info.applicable);
        assert!(info.balanced); // Automatically satisfied
        assert_eq!(info.policy_id, "last_page");
    }

    #[test]
    fn test_balance_info_applicable() {
        let info = BalanceInfo::applicable(Mpt::ZERO, "body_full")
            .with_actual_delta(Mpt::from_pt(0.3), 0.5)
            .with_balanced(true);

        assert!(info.applicable);
        assert_eq!(info.target_delta, Mpt::ZERO);
        assert_eq!(info.actual_delta, Some(Mpt::from_pt(0.3)));
        assert_eq!(info.actual_delta_lines, Some(0.5));
        assert!(info.balanced);
    }

    #[test]
    fn test_layout_metrics() {
        let pages = vec![
            PageLayout {
                page_index: 0,
                balance: BalanceInfo::applicable(Mpt::ZERO, "body_full").with_balanced(true),
                columns: vec![],
                dimensions: (
                    Mpt::ZERO,
                    Mpt::ZERO,
                    Mpt::from_pt(595.0),
                    Mpt::from_pt(842.0),
                ),
                is_blank: false,
            },
            PageLayout {
                page_index: 1,
                balance: BalanceInfo::not_applicable("last_page"),
                columns: vec![],
                dimensions: (
                    Mpt::ZERO,
                    Mpt::ZERO,
                    Mpt::from_pt(595.0),
                    Mpt::from_pt(842.0),
                ),
                is_blank: false,
            },
        ];

        let diagnostics = vec![
            Diagnostic::new(
                DiagnosticCode::LineOverflow,
                DiagnosticSeverity::Warning,
                "Line overflow on page 1",
            ),
            Diagnostic::new(
                DiagnosticCode::PageOverflow,
                DiagnosticSeverity::Error,
                "Page overflow",
            ),
        ];

        let metrics = LayoutMetrics::from_layout(&pages, &diagnostics);

        assert_eq!(metrics.page_count, 2);
        assert_eq!(metrics.balanced_page_count, 1);
        assert_eq!(metrics.satisfied_balance_count, 1);
        assert_eq!(metrics.diagnostic_count, 2);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.warning_count, 1);
    }

    #[test]
    fn test_layout_result_status() {
        let pages = vec![];

        // Valid result (no diagnostics)
        let result = LayoutResult::new("1.0", "hash1", "doc1", pages.clone(), vec![]);
        assert_eq!(result.status, LayoutStatus::Valid);

        // Valid with warnings
        let diags = vec![Diagnostic::new(
            DiagnosticCode::LineOverflow,
            DiagnosticSeverity::Warning,
            "Warning",
        )];
        let result = LayoutResult::new("1.0", "hash1", "doc1", pages.clone(), diags);
        assert_eq!(result.status, LayoutStatus::ValidWithWarnings);

        // Invalid (has errors)
        let diags = vec![Diagnostic::new(
            DiagnosticCode::PageOverflow,
            DiagnosticSeverity::Error,
            "Error",
        )];
        let result = LayoutResult::new("1.0", "hash1", "doc1", pages.clone(), diags);
        assert_eq!(result.status, LayoutStatus::Invalid);
    }

    #[test]
    fn test_layout_result_serialization() {
        let pages = vec![PageLayout {
            page_index: 0,
            balance: BalanceInfo::not_applicable("body_full"),
            columns: vec![ColumnLayout {
                column_index: 0,
                last_baseline_y: Some(Mpt::from_pt(100.0)),
                height: Mpt::from_pt(200.0),
                is_empty: false,
            }],
            dimensions: (
                Mpt::ZERO,
                Mpt::ZERO,
                Mpt::from_pt(595.0),
                Mpt::from_pt(842.0),
            ),
            is_blank: false,
        }];

        let result = LayoutResult::new("1.0", "config-hash", "doc-hash", pages, vec![]);

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: LayoutResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.engine_version, "1.0");
        assert_eq!(deserialized.pages.len(), 1);
        assert_eq!(deserialized.status, LayoutStatus::Valid);
    }
}
