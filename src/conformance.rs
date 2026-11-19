use crate::types::{Asset, OracleMetadata, PriceData};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConformanceViolation {
    #[error("Non-monotonic timestamp sequence: previous {prev}, current {curr}")]
    NonMonotonicTimestamp { prev: u64, curr: u64 },

    #[error("Zero or negative price detected for asset '{asset}': price={price}")]
    InvalidPriceValue { asset: String, price: i128 },

    #[error("Zero resolution configured; resolution must be positive number of seconds")]
    ZeroResolution,

    #[error("Unsupported decimal precision: {decimals} (expected 0 < decimals <= 18)")]
    InvalidDecimals { decimals: u32 },

    #[error("Requested asset '{0}' not supported by oracle")]
    AssetNotFound(String),

    #[error("Stale oracle price detected: age={age}s exceeds threshold {max_allowed}s")]
    StalePrice { age: u64, max_allowed: u64 },

    #[error("Oracle cross-feed price divergence {divergence_bps} bps exceeds max tolerance {max_bps} bps")]
    PriceDivergenceExceeded { divergence_bps: u32, max_bps: u32 },

    #[error("Oracle heartbeat exceeded: update gap of {interval_secs}s exceeds max allowed heartbeat of {max_heartbeat}s")]
    HeartbeatExceeded { interval_secs: u64, max_heartbeat: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConformanceReport {
    pub total_checks: usize,
    pub passed_checks: usize,
    pub violations: Vec<ConformanceViolation>,
}

impl ConformanceReport {
    pub fn is_conformant(&self) -> bool {
        self.violations.is_empty()
    }
}

pub struct ConformanceChecker;

impl ConformanceChecker {
    pub fn verify_metadata(meta: &OracleMetadata) -> ConformanceReport {
        let mut report = ConformanceReport::default();
        report.total_checks += 2;

        if meta.resolution == 0 {
            report.violations.push(ConformanceViolation::ZeroResolution);
        } else {
            report.passed_checks += 1;
        }

        if meta.decimals == 0 || meta.decimals > 18 {
            report
                .violations
                .push(ConformanceViolation::InvalidDecimals { decimals: meta.decimals });
        } else {
            report.passed_checks += 1;
        }

        report
    }

    pub fn verify_price_series(asset: &Asset, records: &[PriceData]) -> ConformanceReport {
        let mut report = ConformanceReport::default();

        let mut last_timestamp = None;
        for record in records {
            report.total_checks += 2;

            if record.price <= 0 {
                report.violations.push(ConformanceViolation::InvalidPriceValue {
                    asset: asset.to_string(),
                    price: record.price,
                });
            } else {
                report.passed_checks += 1;
            }

            if let Some(prev) = last_timestamp {
                // Must be strictly descending or ascending
                if record.timestamp <= prev {
                    report.violations.push(ConformanceViolation::NonMonotonicTimestamp {
                        prev,
                        curr: record.timestamp,
                    });
                } else {
                    report.passed_checks += 1;
                }
            } else {
                report.passed_checks += 1;
            }

            last_timestamp = Some(record.timestamp);
        }

        report
    }

    pub fn verify_freshness(
        record: &PriceData,
        current_time: u64,
        max_staleness_secs: u64,
    ) -> ConformanceReport {
        let mut report = ConformanceReport::default();
        report.total_checks += 1;

        let age = current_time.saturating_sub(record.timestamp);
        if age > max_staleness_secs {
            report.violations.push(ConformanceViolation::StalePrice {
                age,
                max_allowed: max_staleness_secs,
            });
        } else {
            report.passed_checks += 1;
        }

        report
    }

    pub fn verify_cross_feed_tolerance(
        _asset: &Asset,
        feeds: &[PriceData],
        max_divergence_bps: u32,
    ) -> ConformanceReport {
        let mut report = ConformanceReport::default();
        report.total_checks += 1;

        let prices: Vec<i128> = feeds.iter().map(|f| f.price).collect();
        if let Some(div_bps) = crate::normalizer::DecimalNormalizer::compute_divergence_bps(&prices) {
            if div_bps > max_divergence_bps {
                report.violations.push(ConformanceViolation::PriceDivergenceExceeded {
                    divergence_bps: div_bps,
                    max_bps: max_divergence_bps,
                });
            } else {
                report.passed_checks += 1;
            }
        } else {
            report.passed_checks += 1;
        }

        report
    }

    pub fn verify_heartbeat(
        records: &[PriceData],
        max_heartbeat_secs: u64,
    ) -> ConformanceReport {
        let mut report = ConformanceReport::default();
        if records.len() < 2 {
            return report;
        }

        for window in records.windows(2) {
            report.total_checks += 1;
            let gap = window[1].timestamp.saturating_sub(window[0].timestamp);
            if gap > max_heartbeat_secs {
                report.violations.push(ConformanceViolation::HeartbeatExceeded {
                    interval_secs: gap,
                    max_heartbeat: max_heartbeat_secs,
                });
            } else {
                report.passed_checks += 1;
            }
        }

        report
    }
}
