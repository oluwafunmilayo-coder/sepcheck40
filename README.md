# sepcheck40

SEP-40 Decentralized Oracle Conformance and Fault-Injection Testing Suite for Stellar and Soroban.

## Overview

Stellar Ecosystem Proposal 40 (SEP-0040) standardizes oracle contract interfaces for price feeds and financial data across Soroban protocols. Ensuring lending markets, AMMs, and derivative platforms handle edge-case oracle conditions gracefully is critical to on-chain financial safety.

`sepcheck40` provides automated conformance validation and fault-injection testing for SEP-40 compliant price oracle implementations.

### Features

- **Protocol Interface Conformance**: Validates contract interfaces against the SEP-40 specification (`base`, `assets`, `decimals`, `resolution`, `price`, `prices`, `lastprice`).
- **Data Validation & Sanity Verification**: Asserts monotonic timestamp sequencing, decimal precision scaling, and price record sorting requirements.
- **Fault-Injection Engine**: Simulates adverse oracle conditions against dependent smart contracts:
  - Stale price feeds and timestamp halts
  - Extreme price volatility shocks and deviation thresholds
  - Missing asset quotes and unexpected zero/negative prices
  - Round-trip decimal mismatch traps
- **Automated Reporting**: Emits structured conformance matrices and violation diagnostics.

## Getting Started

### Prerequisites

- Rust 1.79+
- Soroban SDK

### Installation

```bash
cargo install --path .
```

### Running Conformance Tests

```bash
# Verify a compiled oracle contract WASM against SEP-40
sepcheck40 verify --wasm path/to/oracle.wasm

# Run fault-injection simulation suite
sepcheck40 fault-inject --wasm path/to/oracle.wasm --scenario stale-prices
```

## Roadmap

- [ ] SEP-40 core interface inspection & validation
- [ ] Timestamp monotonicity and decimal precision verification
- [ ] Fault-injection simulator (stale data, price jumps, missing feeds)
- [ ] Conformance report generator (CLI table & JSON outputs)

## License

Apache-2.0 or MIT
