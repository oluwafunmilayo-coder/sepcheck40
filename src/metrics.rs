use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeedJitterMetrics {
    pub sample_count: usize,
    pub mean_interval_seconds: f64,
    pub max_interval_seconds: u64,
    pub min_interval_seconds: u64,
    pub jitter_variance: f64,
}

pub struct JitterAnalyzer;

impl JitterAnalyzer {
    pub fn calculate(timestamps: &[u64]) -> Option<FeedJitterMetrics> {
        if timestamps.len() < 2 {
            return None;
        }

        let mut intervals = Vec::with_capacity(timestamps.len() - 1);
        for window in timestamps.windows(2) {
            if window[1] >= window[0] {
                intervals.push(window[1] - window[0]);
            }
        }

        if intervals.is_empty() {
            return None;
        }

        let count = intervals.len();
        let sum: u64 = intervals.iter().sum();
        let mean = sum as f64 / count as f64;

        let max = *intervals.iter().max().unwrap();
        let min = *intervals.iter().min().unwrap();

        let variance = intervals
            .iter()
            .map(|&i| {
                let diff = i as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / count as f64;

        Some(FeedJitterMetrics {
            sample_count: count,
            mean_interval_seconds: mean,
            max_interval_seconds: max,
            min_interval_seconds: min,
            jitter_variance: variance,
        })
    }
}
