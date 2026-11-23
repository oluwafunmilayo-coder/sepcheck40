use sepcheck40::SorobanOracleRunner;

#[test]
fn test_real_soroban_oracle_and_consumer_resilience_suite() {
    let report = SorobanOracleRunner::run_resilience_suite();
    assert!(report.all_passed(), "all oracle resilience checks must pass");
    assert_eq!(report.outcomes.len(), 4);
}
