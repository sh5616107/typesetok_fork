//! Page balance policy system.
//!
//! Defines the 7 page policy types from specification section 7.5, each with
//! specific balance requirements and behavior.

use crate::mpt::Mpt;
use serde::{Deserialize, Serialize};

/// Page policy type determining balance requirements.
///
/// Corresponds to the 7 policy types in specification section 7.5 table.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PagePolicy {
    /// Regular full body page: `targetDelta = 0`, no empty columns allowed.
    BodyFull,

    /// Partial body page: `targetDelta = 0` if balance applicable, configurable empty columns.
    BodyPartial,

    /// Last page of document: balance not applicable by default, empty columns allowed.
    LastPage,

    /// Chapter opening page: balance rules defined by master page.
    ChapterOpening,

    /// Part opening page: balance not applicable by default.
    PartOpening,

    /// Title page: balance not applicable.
    TitlePage,

    /// Forced break: inherits balance rules from the page's original policy.
    ForcedBreak,
}

/// Configuration for a page policy.
///
/// Defines the balance target, applicability, and validation rules for a policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Unique identifier for this policy instance.
    pub policy_id: String,

    /// The policy type.
    pub policy: PagePolicy,

    /// Target bottom delta (0 for balanced pages).
    pub target_delta: Mpt,

    /// Whether balance is applicable for this policy.
    pub balance_applicable: bool,

    /// Whether empty columns are allowed.
    pub empty_column_allowed: bool,

    /// Maximum acceptable deviation from target (for success determination).
    pub success_threshold: Mpt,
}

impl PolicyConfig {
    /// Create a default configuration for BodyFull policy.
    pub fn body_full() -> Self {
        Self {
            policy_id: "body_full".to_string(),
            policy: PagePolicy::BodyFull,
            target_delta: Mpt::ZERO,
            balance_applicable: true,
            empty_column_allowed: false,
            success_threshold: Mpt::from_pt(0.5), // Half a point tolerance
        }
    }

    /// Create a default configuration for BodyPartial policy.
    pub fn body_partial() -> Self {
        Self {
            policy_id: "body_partial".to_string(),
            policy: PagePolicy::BodyPartial,
            target_delta: Mpt::ZERO,
            balance_applicable: true,
            empty_column_allowed: true, // Configurable
            success_threshold: Mpt::from_pt(0.5),
        }
    }

    /// Create a default configuration for LastPage policy.
    pub fn last_page() -> Self {
        Self {
            policy_id: "last_page".to_string(),
            policy: PagePolicy::LastPage,
            target_delta: Mpt::ZERO,
            balance_applicable: false, // Not applicable by default
            empty_column_allowed: true,
            success_threshold: Mpt::from_pt(0.5),
        }
    }

    /// Create a default configuration for ChapterOpening policy.
    ///
    /// Balance applicability is determined by the master page definition.
    pub fn chapter_opening(balance_applicable: bool) -> Self {
        Self {
            policy_id: "chapter_opening".to_string(),
            policy: PagePolicy::ChapterOpening,
            target_delta: Mpt::ZERO,
            balance_applicable,
            empty_column_allowed: true,
            success_threshold: Mpt::from_pt(0.5),
        }
    }

    /// Create a default configuration for PartOpening policy.
    pub fn part_opening() -> Self {
        Self {
            policy_id: "part_opening".to_string(),
            policy: PagePolicy::PartOpening,
            target_delta: Mpt::ZERO,
            balance_applicable: false, // Not applicable by default
            empty_column_allowed: true,
            success_threshold: Mpt::from_pt(0.5),
        }
    }

    /// Create a default configuration for TitlePage policy.
    pub fn title_page() -> Self {
        Self {
            policy_id: "title_page".to_string(),
            policy: PagePolicy::TitlePage,
            target_delta: Mpt::ZERO,
            balance_applicable: false, // Never applicable
            empty_column_allowed: true,
            success_threshold: Mpt::from_pt(0.5),
        }
    }

    /// Create a ForcedBreak policy that inherits from another policy.
    pub fn forced_break(inherited_from: &PolicyConfig) -> Self {
        Self {
            policy_id: format!("forced_break_{}", inherited_from.policy_id),
            policy: PagePolicy::ForcedBreak,
            target_delta: inherited_from.target_delta,
            balance_applicable: inherited_from.balance_applicable,
            empty_column_allowed: inherited_from.empty_column_allowed,
            success_threshold: inherited_from.success_threshold,
        }
    }

    /// Check if the actual delta satisfies this policy's requirements.
    pub fn is_satisfied(&self, actual_delta: Mpt) -> bool {
        if !self.balance_applicable {
            return true;
        }

        let deviation = (actual_delta - self.target_delta).raw().abs();
        deviation <= self.success_threshold.raw().abs()
    }
}

/// Resolves the appropriate policy for a page based on its characteristics.
pub struct PolicyResolver {
    default_body_full: PolicyConfig,
    default_body_partial: PolicyConfig,
    default_last_page: PolicyConfig,
    default_chapter_opening: PolicyConfig,
    default_part_opening: PolicyConfig,
    default_title_page: PolicyConfig,
}

impl PolicyResolver {
    /// Create a new policy resolver with default configurations.
    pub fn new() -> Self {
        Self {
            default_body_full: PolicyConfig::body_full(),
            default_body_partial: PolicyConfig::body_partial(),
            default_last_page: PolicyConfig::last_page(),
            default_chapter_opening: PolicyConfig::chapter_opening(true),
            default_part_opening: PolicyConfig::part_opening(),
            default_title_page: PolicyConfig::title_page(),
        }
    }

    /// Resolve the policy for a page.
    pub fn resolve(&self, policy: PagePolicy) -> PolicyConfig {
        match policy {
            PagePolicy::BodyFull => self.default_body_full.clone(),
            PagePolicy::BodyPartial => self.default_body_partial.clone(),
            PagePolicy::LastPage => self.default_last_page.clone(),
            PagePolicy::ChapterOpening => self.default_chapter_opening.clone(),
            PagePolicy::PartOpening => self.default_part_opening.clone(),
            PagePolicy::TitlePage => self.default_title_page.clone(),
            PagePolicy::ForcedBreak => {
                // For forced breaks, default to body_full inheritance
                PolicyConfig::forced_break(&self.default_body_full)
            }
        }
    }
}

impl Default for PolicyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_full_policy() {
        let config = PolicyConfig::body_full();
        assert_eq!(config.policy, PagePolicy::BodyFull);
        assert_eq!(config.target_delta, Mpt::ZERO);
        assert!(config.balance_applicable);
        assert!(!config.empty_column_allowed);
    }

    #[test]
    fn test_last_page_policy() {
        let config = PolicyConfig::last_page();
        assert_eq!(config.policy, PagePolicy::LastPage);
        assert!(!config.balance_applicable);
        assert!(config.empty_column_allowed);
    }

    #[test]
    fn test_forced_break_inheritance() {
        let base = PolicyConfig::body_full();
        let forced = PolicyConfig::forced_break(&base);

        assert_eq!(forced.policy, PagePolicy::ForcedBreak);
        assert_eq!(forced.target_delta, base.target_delta);
        assert_eq!(forced.balance_applicable, base.balance_applicable);
    }

    #[test]
    fn test_policy_satisfaction() {
        let config = PolicyConfig::body_full();

        // Exact match
        assert!(config.is_satisfied(Mpt::ZERO));

        // Within threshold (0.5pt = 500mpt)
        assert!(config.is_satisfied(Mpt::from_raw(400)));
        assert!(config.is_satisfied(Mpt::from_raw(-400)));

        // Outside threshold
        assert!(!config.is_satisfied(Mpt::from_raw(600)));
        assert!(!config.is_satisfied(Mpt::from_raw(-600)));
    }

    #[test]
    fn test_policy_not_applicable() {
        let config = PolicyConfig::last_page();
        assert!(!config.balance_applicable);

        // Any delta is acceptable when not applicable
        assert!(config.is_satisfied(Mpt::from_pt(100.0)));
        assert!(config.is_satisfied(Mpt::from_pt(-100.0)));
    }

    #[test]
    fn test_policy_resolver() {
        let resolver = PolicyResolver::new();

        let body_full = resolver.resolve(PagePolicy::BodyFull);
        assert_eq!(body_full.policy, PagePolicy::BodyFull);
        assert!(body_full.balance_applicable);

        let last_page = resolver.resolve(PagePolicy::LastPage);
        assert_eq!(last_page.policy, PagePolicy::LastPage);
        assert!(!last_page.balance_applicable);
    }

    #[test]
    fn test_all_seven_policies_exist() {
        let policies = [
            PagePolicy::BodyFull,
            PagePolicy::BodyPartial,
            PagePolicy::LastPage,
            PagePolicy::ChapterOpening,
            PagePolicy::PartOpening,
            PagePolicy::TitlePage,
            PagePolicy::ForcedBreak,
        ];

        assert_eq!(policies.len(), 7, "Expected 7 page policy types");
    }
}
