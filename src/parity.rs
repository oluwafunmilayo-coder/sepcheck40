use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityCheckResult {
    pub base_pair: String,
    pub quote_pair: String,
    pub cross_pair: String,
    pub expected_cross_rate: f64,
    pub actual_cross_rate: f64,
    pub discrepancy_bps: u32,
    pub is_consistent: bool,
}

pub struct TriangularParityChecker;

impl TriangularParityChecker {
    /// Checks consistency: pair_ab * pair_bc should equal pair_ac within max_discrepancy_bps (1 bps = 0.01%).
    pub fn verify(
        pair_ab: f64,
        pair_bc: f64,
        pair_ac: f64,
        max_discrepancy_bps: u32,
    ) -> ParityCheckResult {
        let expected = pair_ab * pair_bc;
        let diff = (expected - pair_ac).abs();
        let discrepancy_bps = if pair_ac > 0.0 {
            ((diff / pair_ac) * 10_000.0).round() as u32
        } else {
            10_000
        };

        ParityCheckResult {
            base_pair: "A/B".to_string(),
            quote_pair: "B/C".to_string(),
            cross_pair: "A/C".to_string(),
            expected_cross_rate: expected,
            actual_cross_rate: pair_ac,
            discrepancy_bps,
            is_consistent: discrepancy_bps <= max_discrepancy_bps,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadCheckResult {
    pub bid: f64,
    pub ask: f64,
    pub spread_bps: u32,
    pub is_acceptable: bool,
}

pub struct SpreadValidator;

impl SpreadValidator {
    pub fn verify(bid: f64, ask: f64, max_spread_bps: u32) -> SpreadCheckResult {
        if bid <= 0.0 || ask <= 0.0 || ask < bid {
            return SpreadCheckResult {
                bid,
                ask,
                spread_bps: 10_000,
                is_acceptable: false,
            };
        }
        let spread = ask - bid;
        let mid = (ask + bid) / 2.0;
        let spread_bps = ((spread / mid) * 10_000.0).round() as u32;

        SpreadCheckResult {
            bid,
            ask,
            spread_bps,
            is_acceptable: spread_bps <= max_spread_bps,
        }
    }
}

