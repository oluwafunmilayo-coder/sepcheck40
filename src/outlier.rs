use crate::types::PriceData;

pub struct OutlierFilter;

impl OutlierFilter {
    /// Filters out price records that deviate beyond max_deviation_ratio (e.g. 0.35 = 35%) from median.
    pub fn filter_outliers(records: &[PriceData], max_deviation_ratio: f64) -> Vec<PriceData> {
        if records.len() < 3 {
            return records.to_vec();
        }

        let prices: Vec<i128> = records.iter().map(|r| r.price).collect();
        let median = match crate::normalizer::DecimalNormalizer::compute_median(&prices) {
            Some(m) => m as f64,
            None => return records.to_vec(),
        };

        records
            .iter()
            .filter(|r| {
                let p = r.price as f64;
                let deviation = (p - median).abs() / median;
                deviation <= max_deviation_ratio
            })
            .cloned()
            .collect()
    }
}
