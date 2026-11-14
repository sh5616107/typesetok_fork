//! Configuration structures for the typesetting engine.
//!
//! Defines search limits, weight configurations, and versioning support.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Search limits for balance algorithm and pagination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchLimits {
    /// Maximum number of balance candidates to evaluate per page.
    pub max_candidates: usize,
    
    /// Maximum number of pages to look ahead for optimization.
    pub max_pages: usize,
    
    /// Maximum number of pages to look back for balance adjustments.
    pub lookback: usize,
    
    /// Maximum number of dynamic programming states.
    pub max_dp_states: usize,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            max_candidates: 1000,
            max_pages: 10,
            lookback: 3,
            max_dp_states: 10000,
        }
    }
}

/// Configuration for soft scoring weights.
///
/// These weights determine the relative importance of different layout quality
/// factors when evaluating balance candidates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeightConfig {
    /// Weight for badness of line breaks (word spacing deviation).
    pub line_badness_weight: f32,
    
    /// Weight for badness of page breaks.
    pub page_badness_weight: f32,
    
    /// Weight for paragraph spacing adjustments.
    pub paragraph_spacing_weight: f32,
    
    /// Weight for bottom delta (column balance deviation).
    pub bottom_delta_weight: f32,
    
    /// Weight for widow/orphan penalties.
    pub widow_orphan_weight: f32,
    
    /// Configuration version for tracking changes.
    pub version: String,
}

impl Default for WeightConfig {
    fn default() -> Self {
        Self {
            line_badness_weight: 1.0,
            page_badness_weight: 1.0,
            paragraph_spacing_weight: 0.5,
            bottom_delta_weight: 2.0,
            widow_orphan_weight: 1.5,
            version: "1.0.0".to_string(),
        }
    }
}

/// Complete typesetting configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypesetConfig {
    /// Search limits for algorithms.
    pub search_limits: SearchLimits,
    
    /// Soft scoring weights.
    pub weights: WeightConfig,
    
    /// Word spacing range (min, max) as fractions of ideal spacing.
    pub word_spacing_range: (f32, f32),
    
    /// Paragraph spacing range (min, max) as fractions of ideal spacing.
    pub paragraph_spacing_range: (f32, f32),
}

impl Default for TypesetConfig {
    fn default() -> Self {
        Self {
            search_limits: SearchLimits::default(),
            weights: WeightConfig::default(),
            word_spacing_range: (0.85, 1.40), // 85% to 140% of ideal
            paragraph_spacing_range: (0.80, 1.20), // 80% to 120% of ideal
        }
    }
}

impl TypesetConfig {
    /// Compute a deterministic hash of this configuration.
    ///
    /// Used to detect configuration changes that affect layout results.
    pub fn compute_hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        
        // Hash all relevant configuration fields
        let json = serde_json::to_string(self).unwrap_or_default();
        json.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
}

/// Engine version information.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the current engine version string.
pub fn engine_version() -> String {
    ENGINE_VERSION.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_search_limits() {
        let limits = SearchLimits::default();
        assert_eq!(limits.max_candidates, 1000);
        assert_eq!(limits.max_pages, 10);
        assert_eq!(limits.lookback, 3);
    }

    #[test]
    fn test_default_weight_config() {
        let weights = WeightConfig::default();
        assert_eq!(weights.line_badness_weight, 1.0);
        assert_eq!(weights.bottom_delta_weight, 2.0);
        assert_eq!(weights.version, "1.0.0");
    }

    #[test]
    fn test_config_hash_deterministic() {
        let config1 = TypesetConfig::default();
        let config2 = TypesetConfig::default();
        
        assert_eq!(config1.compute_hash(), config2.compute_hash());
    }

    #[test]
    fn test_config_hash_changes() {
        let config1 = TypesetConfig::default();
        
        let mut config2 = TypesetConfig::default();
        config2.word_spacing_range = (0.80, 1.50);
        
        assert_ne!(config1.compute_hash(), config2.compute_hash());
    }

    #[test]
    fn test_engine_version() {
        let version = engine_version();
        assert!(!version.is_empty());
        // Should be a valid semver string
        assert!(version.contains('.'));
    }

    #[test]
    fn test_config_serialization() {
        let config = TypesetConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TypesetConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config, deserialized);
    }
}
