use crate::host_oracle::{
    LendingConsumerContract, LendingConsumerContractClient, Sep40OracleContract,
    Sep40OracleContractClient,
};
use serde::{Deserialize, Serialize};
use soroban_sdk::{symbol_short, testutils::Ledger, Env};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleResilienceOutcome {
    pub scenario: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleResilienceReport {
    pub outcomes: Vec<OracleResilienceOutcome>,
}

impl OracleResilienceReport {
    pub fn all_passed(&self) -> bool {
        !self.outcomes.is_empty() && self.outcomes.iter().all(|o| o.passed)
    }
}

/// Real Soroban host resilience harness that deploys a SEP-40 oracle contract and a
/// consuming DeFi lending contract, then injects adversarial oracle faults.
pub struct SorobanOracleRunner;

impl SorobanOracleRunner {
    pub fn run_resilience_suite() -> OracleResilienceReport {
        let env = Env::default();
        env.ledger().with_mut(|l| l.timestamp = 1_000_000);

        let oracle_id = env.register(Sep40OracleContract, ());
        let oracle = Sep40OracleContractClient::new(&env, &oracle_id);
        oracle.init(&14, &300);

        let consumer_id = env.register(LendingConsumerContract, ());
        let consumer = LendingConsumerContractClient::new(&env, &consumer_id);

        let xlm = symbol_short!("XLM");
        let mut outcomes = Vec::new();

        // 1. Healthy quote -> borrow succeeds
        oracle.set_price(&xlm, &120_000_000_000_000, &999_950);
        let healthy = consumer.safe_borrow_limit(&oracle_id, &xlm, &100, &300);
        outcomes.push(OracleResilienceOutcome {
            scenario: "healthy_price_quote".to_string(),
            passed: healthy == Some(6_000_000_000_000_000),
            detail: "lending consumer computes 50% LTV borrow limit on fresh quote".to_string(),
        });

        // 2. Stale price fault (> 300s old) -> consumer must safely reject
        oracle.set_price(&xlm, &120_000_000_000_000, &999_000);
        let stale = consumer.safe_borrow_limit(&oracle_id, &xlm, &100, &300);
        outcomes.push(OracleResilienceOutcome {
            scenario: "stale_price_fault_rejected".to_string(),
            passed: stale.is_none(),
            detail: "lending consumer rejected borrow when oracle timestamp exceeded max_age".to_string(),
        });

        // 3. Zero / negative price fault -> consumer must safely reject
        oracle.set_price(&xlm, &0, &999_990);
        let zero = consumer.safe_borrow_limit(&oracle_id, &xlm, &100, &300);
        outcomes.push(OracleResilienceOutcome {
            scenario: "zero_price_fault_rejected".to_string(),
            passed: zero.is_none(),
            detail: "lending consumer rejected borrow on zero price fault".to_string(),
        });

        // 4. Missing asset quote -> consumer must safely reject
        let missing = consumer.safe_borrow_limit(&oracle_id, &symbol_short!("NOASSET"), &100, &300);
        outcomes.push(OracleResilienceOutcome {
            scenario: "missing_quote_rejected".to_string(),
            passed: missing.is_none(),
            detail: "lending consumer rejected borrow when asset has no oracle price".to_string(),
        });

        OracleResilienceReport { outcomes }
    }
}
