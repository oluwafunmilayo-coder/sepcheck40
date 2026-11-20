pub mod conformance;
pub mod fault_injector;
pub mod metrics;
pub mod normalizer;
pub mod outlier;
pub mod parity;
pub mod twap;
pub mod types;

pub use conformance::{ConformanceChecker, ConformanceReport, ConformanceViolation};
pub use fault_injector::FaultInjector;
pub use metrics::{FeedJitterMetrics, JitterAnalyzer};
pub use normalizer::{DecimalNormalizer, NormalizationError};
pub use outlier::OutlierFilter;
pub use parity::{ParityCheckResult, TriangularParityChecker};
pub use twap::TwapAggregator;
pub use types::{Asset, OracleMetadata, PriceData};

pub fn export_report_json(report: &ConformanceReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}
