use sepcheck40::{
    LendingConsumerContract, LendingConsumerContractClient, Sep40OracleContract,
    Sep40OracleContractClient, VulnerableLendingConsumer, VulnerableLendingConsumerClient,
};
use soroban_sdk::{symbol_short, testutils::Ledger, Env};

#[test]
fn test_stale_oracle_caught_by_safe_consumer_but_exploits_vulnerable_consumer() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 2_000_000);

    let oracle_id = env.register(Sep40OracleContract, ());
    let oracle = Sep40OracleContractClient::new(&env, &oracle_id);
    oracle.init(&14, &300);

    let xlm = symbol_short!("XLM");
    // Stale price (2 hours old)
    oracle.set_price(&xlm, &500_000_000_000_000, &1_992_800);

    let safe_id = env.register(LendingConsumerContract, ());
    let safe = LendingConsumerContractClient::new(&env, &safe_id);
    assert!(
        safe.safe_borrow_limit(&oracle_id, &xlm, &100, &300).is_none(),
        "safe consumer must block borrow on stale quote"
    );

    let vuln_id = env.register(VulnerableLendingConsumer, ());
    let vuln = VulnerableLendingConsumerClient::new(&env, &vuln_id);
    assert!(
        vuln.unsafe_borrow_limit(&oracle_id, &xlm, &100).is_some(),
        "vulnerable consumer demonstrates unsafe borrow on stale price"
    );
}
