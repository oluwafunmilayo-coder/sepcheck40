use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostPriceData {
    pub price: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OracleKey {
    Decimals,
    Resolution,
    Price(Symbol),
}

/// Real Soroban SEP-40 compatible oracle contract with deterministic fault injection
/// for resilience testing of consuming DeFi protocols.
#[contract]
pub struct Sep40OracleContract;

#[contractimpl]
impl Sep40OracleContract {
    pub fn init(env: Env, decimals: u32, resolution: u32) {
        env.storage().instance().set(&OracleKey::Decimals, &decimals);
        env.storage().instance().set(&OracleKey::Resolution, &resolution);
    }

    pub fn decimals(env: Env) -> u32 {
        env.storage().instance().get(&OracleKey::Decimals).unwrap_or(14)
    }

    pub fn resolution(env: Env) -> u32 {
        env.storage().instance().get(&OracleKey::Resolution).unwrap_or(300)
    }

    pub fn set_price(env: Env, asset: Symbol, price: i128, timestamp: u64) {
        let data = HostPriceData { price, timestamp };
        env.storage().persistent().set(&OracleKey::Price(asset), &data);
    }

    pub fn lastprice(env: Env, asset: Symbol) -> Option<HostPriceData> {
        env.storage().persistent().get(&OracleKey::Price(asset))
    }
}

/// Real Soroban DeFi lending consumer contract that queries a SEP-40 oracle before
/// approving a collateralized borrow. Enforces staleness and non-positive price guards.
#[contract]
pub struct LendingConsumerContract;

#[contractimpl]
impl LendingConsumerContract {
    /// Returns maximum borrowable USD value (scaled by oracle decimals) if the oracle quote
    /// is present, positive, and fresher than `max_age_seconds`. Returns `None` if unsafe.
    pub fn safe_borrow_limit(
        env: Env,
        oracle_id: Address,
        asset: Symbol,
        collateral_units: i128,
        max_age_seconds: u64,
    ) -> Option<i128> {
        let oracle = Sep40OracleContractClient::new(&env, &oracle_id);
        let quote = oracle.lastprice(&asset)?;
        if quote.price <= 0 {
            return None;
        }
        let now = env.ledger().timestamp();
        if now.saturating_sub(quote.timestamp) > max_age_seconds {
            return None;
        }
        // 50% LTV safety factor
        Some((collateral_units * quote.price) / 2)
    }
}
