use sepcheck40::{Sep40OracleContract, Sep40OracleContractClient};
use soroban_sdk::{symbol_short, Env};

#[test]
fn test_real_sep40_oracle_contract_metadata_and_lastprice() {
    let env = Env::default();
    let oracle_id = env.register(Sep40OracleContract, ());
    let client = Sep40OracleContractClient::new(&env, &oracle_id);

    client.init(&14, &300);
    assert_eq!(client.decimals(), 14);
    assert_eq!(client.resolution(), 300);

    let usdc = symbol_short!("USDC");
    assert!(client.lastprice(&usdc).is_none());

    client.set_price(&usdc, &100_000_000_000_000, &1_700_000_000);
    let quote = client.lastprice(&usdc).expect("quote should exist");
    assert_eq!(quote.price, 100_000_000_000_000);
    assert_eq!(quote.timestamp, 1_700_000_000);
}
