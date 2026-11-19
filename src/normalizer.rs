use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum NormalizationError {
    #[error("Decimals {0} exceeds maximum supported 18")]
    ExcessiveDecimals(u32),
    #[error("Arithmetic overflow during price normalization")]
    Overflow,
}

pub struct DecimalNormalizer;

impl DecimalNormalizer {
    pub const MAX_DECIMALS: u32 = 18;

    /// Normalizes a price from `from_decimals` to `to_decimals` using checked arithmetic.
    pub fn normalize(
        price: i128,
        from_decimals: u32,
        to_decimals: u32,
    ) -> Result<i128, NormalizationError> {
        if from_decimals > Self::MAX_DECIMALS {
            return Err(NormalizationError::ExcessiveDecimals(from_decimals));
        }
        if to_decimals > Self::MAX_DECIMALS {
            return Err(NormalizationError::ExcessiveDecimals(to_decimals));
        }

        if from_decimals == to_decimals {
            return Ok(price);
        }

        if to_decimals > from_decimals {
            let diff = to_decimals - from_decimals;
            let factor = 10_i128
                .checked_pow(diff)
                .ok_or(NormalizationError::Overflow)?;
            price.checked_mul(factor).ok_or(NormalizationError::Overflow)
        } else {
            let diff = from_decimals - to_decimals;
            let factor = 10_i128
                .checked_pow(diff)
                .ok_or(NormalizationError::Overflow)?;
            Ok(price / factor)
        }
    }

    /// Computes the median price among a slice of prices.
    pub fn compute_median(prices: &[i128]) -> Option<i128> {
        if prices.is_empty() {
            return None;
        }
        let mut sorted = prices.to_vec();
        sorted.sort_unstable();
        let len = sorted.len();
        if len % 2 == 1 {
            Some(sorted[len / 2])
        } else {
            let mid1 = sorted[(len / 2) - 1];
            let mid2 = sorted[len / 2];
            Some((mid1 + mid2) / 2)
        }
    }

    /// Computes maximum pairwise divergence in basis points (1 bp = 0.01%).
    /// (max - min) / min * 10,000
    pub fn compute_divergence_bps(prices: &[i128]) -> Option<u32> {
        if prices.len() < 2 {
            return Some(0);
        }
        let min_price = *prices.iter().min()?;
        let max_price = *prices.iter().max()?;
        if min_price <= 0 {
            return None;
        }

        let diff = (max_price - min_price) as u128;
        let bps = (diff.saturating_mul(10_000)) / (min_price as u128);
        Some(bps as u32)
    }
}
