//! Diagnostic waiver system.
//!
//! Implements the waiver mechanism from specification section 13, allowing
//! approved exceptions to diagnostic rules.

use crate::diagnostic::DiagnosticCode;
use serde::{Deserialize, Serialize};

/// Scope of a waiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WaiverScope {
    /// Waiver applies to a single page.
    Page,
    
    /// Waiver applies to an entire section.
    Section,
    
    /// Waiver applies to the entire document.
    Document,
}

/// A waiver record representing an approved exception to a diagnostic rule.
///
/// Waivers allow specific diagnostics to be suppressed when they have been
/// reviewed and accepted by an authorized user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaiverRecord {
    /// Unique identifier for this waiver.
    pub waiver_id: String,
    
    /// The diagnostic code being waived.
    pub diagnostic_code: DiagnosticCode,
    
    /// Zero-based page index (if scope is Page).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_index: Option<usize>,
    
    /// Anchor IDs related to this waiver.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anchor_ids: Vec<String>,
    
    /// User or system that approved this waiver.
    pub approved_by: String,
    
    /// Timestamp when the waiver was approved (ISO-8601 format).
    pub approved_at: String,
    
    /// Human-readable reason for the waiver.
    pub reason: String,
    
    /// Scope of the waiver.
    pub scope: WaiverScope,
}

impl WaiverRecord {
    /// Create a new waiver record.
    pub fn new(
        waiver_id: impl Into<String>,
        diagnostic_code: DiagnosticCode,
        approved_by: impl Into<String>,
        reason: impl Into<String>,
        scope: WaiverScope,
    ) -> Self {
        Self {
            waiver_id: waiver_id.into(),
            diagnostic_code,
            page_index: None,
            anchor_ids: Vec::new(),
            approved_by: approved_by.into(),
            approved_at: chrono::Utc::now().to_rfc3339(),
            reason: reason.into(),
            scope,
        }
    }

    /// Builder: set the page index.
    pub fn with_page(mut self, page_index: usize) -> Self {
        self.page_index = Some(page_index);
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

    /// Check if this waiver matches the given diagnostic context.
    pub fn matches(
        &self,
        code: DiagnosticCode,
        page_index: Option<usize>,
        anchor_ids: &[String],
    ) -> bool {
        // Code must match
        if self.diagnostic_code != code {
            return false;
        }

        // Check scope-specific matching
        match self.scope {
            WaiverScope::Document => {
                // Document-wide waiver matches any occurrence of the code
                true
            }
            WaiverScope::Section => {
                // Section waiver matches if any anchor overlaps
                if anchor_ids.is_empty() || self.anchor_ids.is_empty() {
                    return false;
                }
                anchor_ids.iter().any(|id| self.anchor_ids.contains(id))
            }
            WaiverScope::Page => {
                // Page waiver matches if page index matches
                match (self.page_index, page_index) {
                    (Some(waiver_page), Some(diag_page)) => waiver_page == diag_page,
                    _ => false,
                }
            }
        }
    }
}

/// Manages a collection of waivers and provides matching logic.
#[derive(Debug, Clone, Default)]
pub struct WaiverRegistry {
    waivers: Vec<WaiverRecord>,
}

impl WaiverRegistry {
    /// Create a new empty waiver registry.
    pub fn new() -> Self {
        Self {
            waivers: Vec::new(),
        }
    }

    /// Add a waiver to the registry.
    pub fn add(&mut self, waiver: WaiverRecord) {
        self.waivers.push(waiver);
    }

    /// Check if a diagnostic is waived.
    pub fn is_waived(
        &self,
        code: DiagnosticCode,
        page_index: Option<usize>,
        anchor_ids: &[String],
    ) -> bool {
        self.waivers
            .iter()
            .any(|w| w.matches(code, page_index, anchor_ids))
    }

    /// Find all waivers that match the given context.
    pub fn find_matching(
        &self,
        code: DiagnosticCode,
        page_index: Option<usize>,
        anchor_ids: &[String],
    ) -> Vec<&WaiverRecord> {
        self.waivers
            .iter()
            .filter(|w| w.matches(code, page_index, anchor_ids))
            .collect()
    }

    /// Get all waivers in the registry.
    pub fn all(&self) -> &[WaiverRecord] {
        &self.waivers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waiver_creation() {
        let waiver = WaiverRecord::new(
            "w-001",
            DiagnosticCode::BalanceUnsatisfiable,
            "user@example.com",
            "Known issue in chapter 5",
            WaiverScope::Section,
        )
        .with_anchor("section-5");

        assert_eq!(waiver.waiver_id, "w-001");
        assert_eq!(waiver.diagnostic_code, DiagnosticCode::BalanceUnsatisfiable);
        assert_eq!(waiver.scope, WaiverScope::Section);
        assert_eq!(waiver.anchor_ids, vec!["section-5"]);
    }

    #[test]
    fn test_document_waiver_matching() {
        let waiver = WaiverRecord::new(
            "w-doc",
            DiagnosticCode::FontGlyphMissing,
            "admin",
            "Missing rare glyphs accepted",
            WaiverScope::Document,
        );

        // Document waiver matches any occurrence
        assert!(waiver.matches(DiagnosticCode::FontGlyphMissing, None, &[]));
        assert!(waiver.matches(DiagnosticCode::FontGlyphMissing, Some(5), &[]));
        assert!(waiver.matches(
            DiagnosticCode::FontGlyphMissing,
            Some(10),
            &["anchor-1".to_string()]
        ));

        // Different code does not match
        assert!(!waiver.matches(DiagnosticCode::LineOverflow, None, &[]));
    }

    #[test]
    fn test_page_waiver_matching() {
        let waiver = WaiverRecord::new(
            "w-page",
            DiagnosticCode::LineOverflow,
            "editor",
            "Accepted overflow on page 3",
            WaiverScope::Page,
        )
        .with_page(3);

        // Matches only the specific page
        assert!(waiver.matches(DiagnosticCode::LineOverflow, Some(3), &[]));
        assert!(!waiver.matches(DiagnosticCode::LineOverflow, Some(4), &[]));
        assert!(!waiver.matches(DiagnosticCode::LineOverflow, None, &[]));
    }

    #[test]
    fn test_section_waiver_matching() {
        let waiver = WaiverRecord::new(
            "w-section",
            DiagnosticCode::ManualBreakConflict,
            "editor",
            "Accepted in appendix",
            WaiverScope::Section,
        )
        .with_anchors(vec!["appendix-a".to_string(), "appendix-b".to_string()]);

        // Matches if any anchor overlaps
        assert!(waiver.matches(
            DiagnosticCode::ManualBreakConflict,
            None,
            &["appendix-a".to_string()]
        ));
        assert!(waiver.matches(
            DiagnosticCode::ManualBreakConflict,
            None,
            &["other".to_string(), "appendix-b".to_string()]
        ));

        // Does not match without anchor overlap
        assert!(!waiver.matches(
            DiagnosticCode::ManualBreakConflict,
            None,
            &["chapter-1".to_string()]
        ));
        assert!(!waiver.matches(DiagnosticCode::ManualBreakConflict, None, &[]));
    }

    #[test]
    fn test_waiver_registry() {
        let mut registry = WaiverRegistry::new();

        let w1 = WaiverRecord::new(
            "w-1",
            DiagnosticCode::BalanceUnsatisfiable,
            "user",
            "reason 1",
            WaiverScope::Document,
        );
        let w2 = WaiverRecord::new(
            "w-2",
            DiagnosticCode::LineOverflow,
            "user",
            "reason 2",
            WaiverScope::Page,
        )
        .with_page(5);

        registry.add(w1);
        registry.add(w2);

        assert!(registry.is_waived(DiagnosticCode::BalanceUnsatisfiable, None, &[]));
        assert!(registry.is_waived(DiagnosticCode::LineOverflow, Some(5), &[]));
        assert!(!registry.is_waived(DiagnosticCode::LineOverflow, Some(6), &[]));
        assert!(!registry.is_waived(DiagnosticCode::PageOverflow, None, &[]));
    }
}
