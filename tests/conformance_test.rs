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
fn test_mutant_oracle_with_invalid_decimal_precision_is_rejected() {
    let mutant_metadata = OracleMetadata {
        base: Asset::Other("USD".to_string()),
        decimals: 19,
        resolution: 300,
        assets: vec![Asset::Other("XLM".to_string())],
    };

    let report = ConformanceChecker::verify_metadata(&mutant_metadata);
    assert!(!report.is_conformant());
    assert!(report.violations.iter().any(|violation| matches!(
        violation,
        ConformanceViolation::InvalidDecimals { decimals: 19 }
    )));
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

#[test]
fn test_triangular_cross_rate_parity() {
    use sepcheck40::TriangularParityChecker;

    let res_ok = TriangularParityChecker::verify(0.10, 0.90, 0.09, 50);
    assert!(res_ok.is_consistent);
    assert_eq!(res_ok.discrepancy_bps, 0);

    let res_bad = TriangularParityChecker::verify(0.10, 0.90, 0.10, 50);
    assert!(!res_bad.is_consistent);
    assert!(res_bad.discrepancy_bps > 50);
}

#[test]
fn test_feed_jitter_calculations() {
    use sepcheck40::JitterAnalyzer;

    let timestamps = vec![1000, 1010, 1025, 1030];
    let metrics = JitterAnalyzer::calculate(&timestamps).expect("Sufficient sample size");
    assert_eq!(metrics.sample_count, 3);
    assert_eq!(metrics.mean_interval_seconds, 10.0);
    assert_eq!(metrics.max_interval_seconds, 15);
    assert_eq!(metrics.min_interval_seconds, 5);
    assert!(metrics.jitter_variance > 0.0);

    assert!(JitterAnalyzer::calculate(&[1000]).is_none());
}

#[test]
fn test_asset_symbol_normalization() {
    use sepcheck40::Asset;

    let stellar_asset = Asset::Stellar("  xlm  ".into());
    assert_eq!(stellar_asset.normalized_symbol(), "XLM");
    assert!(stellar_asset.matches_symbol("XLM"));
    assert!(stellar_asset.matches_symbol("xlm"));
    assert!(!stellar_asset.matches_symbol("USDC"));

    let other_asset = Asset::Other("eurc".into());
    assert_eq!(other_asset.normalized_symbol(), "EURC");
    assert!(other_asset.matches_symbol("EURC"));
}

#[test]
fn test_conformance_markdown_scorecard() {
    let ok_report = sepcheck40::ConformanceReport::default();
    let md_ok = ok_report.to_markdown_scorecard();
    assert!(md_ok.contains("Status: **COMPLIANT**"));

    let mut bad_report = sepcheck40::ConformanceReport::default();
    bad_report.violations.push(sepcheck40::ConformanceViolation::ZeroResolution);
    let md_bad = bad_report.to_markdown_scorecard();
    assert!(md_bad.contains("Status: **NON-COMPLIANT**"));
    assert!(md_bad.contains("Zero resolution"));
}

#[test]
fn test_example_fixtures_oracle_feed_parsing() {
    let feed_json = include_str!("../examples/fixtures/mock_oracle_feed.json");
    let val: serde_json::Value = serde_json::from_str(feed_json).expect("valid feed JSON");
    assert_eq!(val["base"], "XLM");
    assert_eq!(val["decimals"], 7);

    let ticks_json = include_str!("../examples/fixtures/price_ticks.json");
    let ticks: Vec<serde_json::Value> = serde_json::from_str(ticks_json).expect("valid ticks JSON");
    assert_eq!(ticks.len(), 3);
    assert_eq!(ticks[0]["asset"], "USDC");
}

#[test]
fn test_spread_validator_threshold_rules() {
    use sepcheck40::SpreadValidator;

    let res_ok = SpreadValidator::verify(10.0, 10.05, 100);
    assert!(res_ok.is_acceptable);
    assert!(res_ok.spread_bps <= 100);

    let res_wide = SpreadValidator::verify(10.0, 10.50, 100);
    assert!(!res_wide.is_acceptable);
    assert!(res_wide.spread_bps > 100);

    let res_inverted = SpreadValidator::verify(10.50, 10.0, 100);
    assert!(!res_inverted.is_acceptable);
}

#[test]
fn test_conformance_csv_export_format() {
    let mut report = sepcheck40::ConformanceReport::default();
    report.passed_checks = 10;
    let csv = report.to_csv();
    assert!(csv.starts_with("check_category,passed,failed,is_conformant\n"));
    assert!(csv.contains("oracle_conformance,10,0,true"));

    report.violations.push(sepcheck40::ConformanceViolation::ZeroResolution);
    let csv_bad = report.to_csv();
    assert!(csv_bad.contains("oracle_conformance,10,1,false"));
}

#[test]
fn test_sep40_cli_guide_exists_and_covers_checks() {
    let guide_path = std::path::Path::new("docs/CLI_GUIDE.md");
    assert!(guide_path.exists(), "docs/CLI_GUIDE.md should exist");
    let content = std::fs::read_to_string(guide_path).expect("read CLI_GUIDE.md");
    assert!(content.contains("# SEP-0040 Oracle Conformance Checker Guide"));
    assert!(content.contains("--format csv"));
    assert!(content.contains("Conformance Rules Verified"));
}




