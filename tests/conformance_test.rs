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

#[test]
fn test_stale_price_freshness_rejection() {
    let now = 1700001000;
    let fresh_feed = PriceData { price: 100_000, timestamp: 1700000950 }; // 50s old
    let stale_feed = PriceData { price: 100_000, timestamp: 1700000500 }; // 500s old

    let fresh_report = ConformanceChecker::verify_freshness(&fresh_feed, now, 300);
    assert!(fresh_report.is_conformant());

    let stale_report = ConformanceChecker::verify_freshness(&stale_feed, now, 300);
    assert!(!stale_report.is_conformant());
    assert!(matches!(
        stale_report.violations[0],
        ConformanceViolation::StalePrice { age: 500, max_allowed: 300 }
    ));
}

#[test]
fn test_cross_feed_divergence_detection() {
    let asset = Asset::Other("ETH".to_string());
    // 3 oracle feeds: 2000, 2010, 2015 -> max divergence is (2015-2000)/2000 = 15/2000 = 0.75% = 75 bps
    let feeds_ok = vec![
        PriceData { price: 2000_000000, timestamp: 1700000000 },
        PriceData { price: 2010_000000, timestamp: 1700000000 },
        PriceData { price: 2015_000000, timestamp: 1700000000 },
    ];
    let rep_ok = ConformanceChecker::verify_cross_feed_tolerance(&asset, &feeds_ok, 100); // 100 bps tolerance
    assert!(rep_ok.is_conformant());

    // Divergent feed: 2000 vs 2100 -> 500 bps divergence
    let feeds_divergent = vec![
        PriceData { price: 2000_000000, timestamp: 1700000000 },
        PriceData { price: 2100_000000, timestamp: 1700000000 },
    ];
    let rep_err = ConformanceChecker::verify_cross_feed_tolerance(&asset, &feeds_divergent, 100);
    assert!(!rep_err.is_conformant());
    assert!(matches!(
        rep_err.violations[0],
        ConformanceViolation::PriceDivergenceExceeded { divergence_bps: 500, max_bps: 100 }
    ));
}

#[test]
fn test_decimal_normalizer_scaling() {
    use sepcheck40::DecimalNormalizer;

    // Scale 7 decimals (Stellar) to 14 decimals
    let stellar_price = 12_5000000; // 12.5 XLM
    let scaled = DecimalNormalizer::normalize(stellar_price, 7, 14).unwrap();
    assert_eq!(scaled, 12_50000000000000);

    // Downscale 14 to 7
    let downscaled = DecimalNormalizer::normalize(scaled, 14, 7).unwrap();
    assert_eq!(downscaled, stellar_price);

    // Excess decimals error
    let err = DecimalNormalizer::normalize(100, 19, 7).unwrap_err();
    assert!(err.to_string().contains("exceeds maximum"));
}

#[test]
fn test_median_and_divergence_math() {
    use sepcheck40::DecimalNormalizer;

    // Odd number of elements
    let odd_prices = vec![100, 300, 200];
    assert_eq!(DecimalNormalizer::compute_median(&odd_prices), Some(200));

    // Even number of elements
    let even_prices = vec![100, 200, 300, 400];
    assert_eq!(DecimalNormalizer::compute_median(&even_prices), Some(250));

    // Divergence basis points
    let prices = vec![1000, 1050]; // (1050 - 1000) / 1000 = 50 / 1000 = 5% = 500 bps
    assert_eq!(DecimalNormalizer::compute_divergence_bps(&prices), Some(500));
}

#[test]
fn test_heartbeat_violation_detection() {
    let regular_feed = vec![
        PriceData { price: 100_000, timestamp: 1000 },
        PriceData { price: 100_500, timestamp: 1300 }, // 300s gap
    ];
    let rep_ok = ConformanceChecker::verify_heartbeat(&regular_feed, 300);
    assert!(rep_ok.is_conformant());

    let delayed_feed = vec![
        PriceData { price: 100_000, timestamp: 1000 },
        PriceData { price: 100_500, timestamp: 1500 }, // 500s gap > 300s
    ];
    let rep_err = ConformanceChecker::verify_heartbeat(&delayed_feed, 300);
    assert!(!rep_err.is_conformant());
    assert!(matches!(
        rep_err.violations[0],
        ConformanceViolation::HeartbeatExceeded { interval_secs: 500, max_heartbeat: 300 }
    ));
}

#[test]
fn test_conformance_report_json_export() {
    let mut report = sepcheck40::ConformanceReport::default();
    report.total_checks = 5;
    report.passed_checks = 5;
    let json_str = sepcheck40::export_report_json(&report).unwrap();
    assert!(json_str.contains("\"total_checks\": 5"));
    assert!(json_str.contains("\"passed_checks\": 5"));
}
