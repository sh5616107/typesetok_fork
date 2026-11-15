//! Quality scoring system for balance candidates.
//!
//! Provides functions to evaluate soft quality metrics like word spacing,
//! line quality, widow/orphan violations, and paragraph spacing.

use crate::candidate::{BalanceCandidate, WeightConfig};
use crate::mpt::Mpt;

/// Select the best candidate from a pool based on soft score.
///
/// Filters candidates by balance threshold, then selects the one with
/// the lowest soft score (best quality).
pub fn select_best_candidate(
    candidates: &[BalanceCandidate],
    threshold: Mpt,
    weights: &WeightConfig,
) -> Option<BalanceCandidate> {
    // Filter by balance threshold
    let mut acceptable: Vec<&BalanceCandidate> = candidates
        .iter()
        .filter(|c| c.bottom_delta.raw().abs() <= threshold.raw().abs())
        .collect();

    if acceptable.is_empty() {
        return None;
    }

    // Sort by soft score (lower is better)
    acceptable.sort_by(|a, b| {
        let score_a = a.soft_score(weights);
        let score_b = b.soft_score(weights);
        score_a
            .partial_cmp(&score_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    acceptable.first().map(|c| (*c).clone())
}

/// Calculate word spacing quality (0.0 = worst, 1.0 = best).
///
/// Evaluates how close the word spacing is to the ideal (typically 1.0).
pub fn calculate_word_spacing_quality(spacing_factor: f32) -> f32 {
    const IDEAL_SPACING: f32 = 1.0;
    const MAX_DEVIATION: f32 = 0.5; // Beyond this, quality is 0

    let deviation = (spacing_factor - IDEAL_SPACING).abs();
    if deviation >= MAX_DEVIATION {
        0.0
    } else {
        1.0 - (deviation / MAX_DEVIATION)
    }
}

/// Calculate line quality score based on badness.
///
/// Higher badness = lower quality. Normalizes badness to 0-1 range.
pub fn calculate_line_quality(avg_badness: f32) -> f32 {
    const MAX_ACCEPTABLE_BADNESS: f32 = 10000.0;

    if avg_badness >= MAX_ACCEPTABLE_BADNESS {
        0.0
    } else {
        1.0 - (avg_badness / MAX_ACCEPTABLE_BADNESS)
    }
}

/// Count widow and orphan violations in a candidate.
///
/// For Phase 3, this is a placeholder returning 0.
/// Full implementation requires paragraph-level information.
pub fn count_widow_orphan_violations(_candidate: &BalanceCandidate) -> usize {
    // TODO Phase 4: Implement with paragraph boundaries
    0
}

/// Calculate paragraph spacing deviation.
///
/// Measures how much the actual paragraph spacing deviates from ideal.
pub fn calculate_para_spacing_deviation(actual_spacing: Mpt, ideal_spacing: Mpt) -> f32 {
    (actual_spacing.to_pt() - ideal_spacing.to_pt()).abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::{BalanceCandidate, Stage};

    #[test]
    fn test_select_best_with_threshold() {
        let candidates = vec![
            {
                let mut c = BalanceCandidate::new(Stage::ColumnDivision, "a".to_string());
                c.bottom_delta = Mpt::from_pt(1.0);
                c.word_spacing_quality = 0.9;
                c
            },
            {
                let mut c = BalanceCandidate::new(Stage::ColumnDivision, "b".to_string());
                c.bottom_delta = Mpt::from_pt(0.5);
                c.word_spacing_quality = 0.8;
                c
            },
            {
                let mut c = BalanceCandidate::new(Stage::ColumnDivision, "c".to_string());
                c.bottom_delta = Mpt::from_pt(5.0); // Too large
                c.word_spacing_quality = 1.0;
                c
            },
        ];

        let threshold = Mpt::from_pt(2.0);
        let weights = WeightConfig::default();

        let best = select_best_candidate(&candidates, threshold, &weights).unwrap();

        // Should select "a" (better quality with lower soft score)
        assert_eq!(best.strategy, "a");
    }

    #[test]
    fn test_select_best_none_acceptable() {
        let candidates = vec![{
            let mut c = BalanceCandidate::new(Stage::ColumnDivision, "a".to_string());
            c.bottom_delta = Mpt::from_pt(10.0);
            c
        }];

        let threshold = Mpt::from_pt(1.0);
        let weights = WeightConfig::default();

        let best = select_best_candidate(&candidates, threshold, &weights);
        assert!(best.is_none());
    }

    #[test]
    fn test_word_spacing_quality_ideal() {
        let quality = calculate_word_spacing_quality(1.0);
        assert_eq!(quality, 1.0);
    }

    #[test]
    fn test_word_spacing_quality_deviation() {
        let quality = calculate_word_spacing_quality(1.25);
        assert!(quality > 0.4 && quality < 0.6);
    }

    #[test]
    fn test_word_spacing_quality_extreme() {
        let quality = calculate_word_spacing_quality(2.0);
        assert_eq!(quality, 0.0);
    }

    #[test]
    fn test_line_quality_perfect() {
        let quality = calculate_line_quality(0.0);
        assert_eq!(quality, 1.0);
    }

    #[test]
    fn test_line_quality_bad() {
        let quality = calculate_line_quality(5000.0);
        assert_eq!(quality, 0.5);
    }

    #[test]
    fn test_line_quality_worst() {
        let quality = calculate_line_quality(10000.0);
        assert_eq!(quality, 0.0);
    }

    #[test]
    fn test_para_spacing_deviation() {
        let deviation = calculate_para_spacing_deviation(Mpt::from_pt(12.0), Mpt::from_pt(10.0));
        assert_eq!(deviation, 2.0);
    }

    #[test]
    fn test_widow_orphan_placeholder() {
        let candidate = BalanceCandidate::new(Stage::ColumnDivision, "test".to_string());
        let count = count_widow_orphan_violations(&candidate);
        assert_eq!(count, 0); // Placeholder implementation
    }
}
