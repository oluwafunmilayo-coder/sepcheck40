use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Asset {
    Stellar(String),
    Other(String),
}

impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stellar(s) => write!(f, "Stellar({})", s),
            Self::Other(s) => write!(f, "Other({})", s),
        }
    }
}

impl Asset {
    pub fn normalized_symbol(&self) -> String {
        match self {
            Self::Stellar(s) => s.trim().to_uppercase(),
            Self::Other(s) => s.trim().to_uppercase(),
        }
    }

    pub fn matches_symbol(&self, symbol: &str) -> bool {
        self.normalized_symbol() == symbol.trim().to_uppercase()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleMetadata {
    pub base: Asset,
    pub decimals: u32,
    pub resolution: u32,
    pub assets: Vec<Asset>,
}
