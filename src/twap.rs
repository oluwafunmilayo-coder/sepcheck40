use crate::types::PriceData;

pub struct TwapAggregator;

impl TwapAggregator {
    /// Computes Time-Weighted Average Price (TWAP) across chronologically ordered price records.
    pub fn compute_twap(records: &[PriceData]) -> Option<i128> {
        if records.is_empty() {
            return None;
        }
        if records.len() == 1 {
            return Some(records[0].price);
        }

        let mut cumulative_price_time: i128 = 0;
        let mut total_time_span: u64 = 0;

        for window in records.windows(2) {
            let p0 = &window[0];
            let p1 = &window[1];
            if p1.timestamp > p0.timestamp {
                let dt = p1.timestamp - p0.timestamp;
                // Trapezoidal average price over interval dt
                let avg_price = (p0.price + p1.price) / 2;
                cumulative_price_time += avg_price * (dt as i128);
                total_time_span += dt;
            }
        }

        if total_time_span == 0 {
            Some(records.last()?.price)
        } else {
            Some(cumulative_price_time / (total_time_span as i128))
        }
    }
}
