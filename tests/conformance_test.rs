use sepcheck40::{
    Asset, ConformanceChecker, ConformanceViolation, FaultInjector, OracleMetadata, PriceData,
};

#[test]
fn test_valid_oracle_metadata() {
    let meta = OracleMetadata {
        base: Asset::Other("USD".to_string()),
        decimals: 14,
        resolution: 300,
        assets: vec![
            Asset::Stellar("CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC".to_string()),
        ],
    };

    let report = ConformanceChecker::verify_metadata(&meta);
    assert!(report.is_conformant());
    assert_eq!(report.passed_checks, 2);
}

#[test]
fn test_compliant_price_feed() {
    let asset = Asset::Other("XLM".to_string());
    let prices = vec![
        PriceData { price: 125_000, timestamp: 1700000000 },
        PriceData { price: 125_500, timestamp: 1700000300 },
        PriceData { price: 126_100, timestamp: 1700000600 },
    ];

    let report = ConformanceChecker::verify_price_series(&asset, &prices);
    assert!(report.is_conformant());
}

#[test]
fn test_non_monotonic_timestamp_detection() {
    let asset = Asset::Other("XLM".to_string());
    let valid_prices = vec![
        PriceData { price: 125_000, timestamp: 1700000000 },
        PriceData { price: 125_500, timestamp: 1700000300 },
        PriceData { price: 126_100, timestamp: 1700000600 },
    ];

    let corrupted = FaultInjector::inject_inverted_timestamps(&valid_prices);
    let report = ConformanceChecker::verify_price_series(&asset, &corrupted);

    assert!(!report.is_conformant());
    assert!(report.violations.iter().any(|v| matches!(v, ConformanceViolation::NonMonotonicTimestamp { .. })));
}

#[test]
fn test_zero_or_negative_price_detection() {
    let asset = Asset::Other("BTC".to_string());
    let bad_prices = vec![
        PriceData { price: 0, timestamp: 1700000000 },
        PriceData { price: -500, timestamp: 1700000300 },
    ];

    let report = ConformanceChecker::verify_price_series(&asset, &bad_prices);
    assert!(!report.is_conformant());
    assert_eq!(report.violations.len(), 2);
}

#[test]
fn test_price_shock_fault_injection() {
    let prices = vec![
        PriceData { price: 100_000, timestamp: 1700000000 },
        PriceData { price: 101_000, timestamp: 1700000300 },
    ];

    let shocked = FaultInjector::inject_price_shock(&prices, 0.20); // 80% flash crash
    assert_eq!(shocked[0].price, 20_000);
    assert_eq!(shocked[1].price, 20_200);
}
