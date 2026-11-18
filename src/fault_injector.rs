use crate::types::PriceData;

pub struct FaultInjector;

impl FaultInjector {
    /// Simulates a stale price feed by locking timestamps to an old historical timestamp.
    pub fn inject_stale_timestamp(records: &[PriceData], stale_offset_secs: u64) -> Vec<PriceData> {
        records
            .iter()
            .map(|r| PriceData {
                price: r.price,
                timestamp: r.timestamp.saturating_sub(stale_offset_secs),
            })
            .collect()
    }

    /// Simulates sudden extreme market volatility shock (e.g., +50% or -80% flash crash).
    pub fn inject_price_shock(records: &[PriceData], percentage_multiplier: f64) -> Vec<PriceData> {
        records
            .iter()
            .map(|r| PriceData {
                price: ((r.price as f64) * percentage_multiplier) as i128,
                timestamp: r.timestamp,
            })
            .collect()
    }

    /// Simulates a corrupted oracle node publishing unordered or inverted ledger timestamps.
    pub fn inject_inverted_timestamps(records: &[PriceData]) -> Vec<PriceData> {
        let mut corrupted = records.to_vec();
        if corrupted.len() >= 2 {
            // Swap first two timestamps to trigger monotonicity failure
            let t0 = corrupted[0].timestamp;
            let t1 = corrupted[1].timestamp;
            corrupted[0].timestamp = t1;
            corrupted[1].timestamp = t0;
        }
        corrupted
    }
}
