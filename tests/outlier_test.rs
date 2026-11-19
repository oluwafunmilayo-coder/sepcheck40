use sepcheck40::{OutlierFilter, PriceData};

#[test]
fn test_outlier_filtering_on_spike() {
    let prices = vec![
        PriceData { price: 100_000, timestamp: 1000 },
        PriceData { price: 101_000, timestamp: 1010 },
        PriceData { price: 99_000, timestamp: 1020 },
        PriceData { price: 500_000, timestamp: 1030 }, // 500% flash spike
        PriceData { price: 100_500, timestamp: 1040 },
    ];

    let filtered = OutlierFilter::filter_outliers(&prices, 0.30); // 30% max deviation
    assert_eq!(filtered.len(), 4);
    assert!(!filtered.iter().any(|p| p.price == 500_000));
}
