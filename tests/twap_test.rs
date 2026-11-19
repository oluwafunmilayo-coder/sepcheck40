use sepcheck40::{PriceData, TwapAggregator};

#[test]
fn test_twap_computation_variable_intervals() {
    let prices = vec![
        PriceData { price: 100_000, timestamp: 1000 },
        PriceData { price: 200_000, timestamp: 1100 }, // 100s, avg 150_000 -> weight 15_000_000
        PriceData { price: 200_000, timestamp: 1300 }, // 200s, avg 200_000 -> weight 40_000_000
    ];

    let twap = TwapAggregator::compute_twap(&prices).unwrap();
    // total time: 300s, total price-time: 55_000_000 / 300 = 183_333
    assert_eq!(twap, 183_333);
}

#[test]
fn test_twap_single_record() {
    let prices = vec![PriceData { price: 42_000, timestamp: 1000 }];
    assert_eq!(TwapAggregator::compute_twap(&prices), Some(42_000));
}
