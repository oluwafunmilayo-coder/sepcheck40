# SEP-0040 Oracle Conformance Checker Guide

`sepcheck40` validates decentralized price feed oracle contracts on Stellar Soroban against the [SEP-0040: Standard Oracle Interface](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0040.md) specification.

---

## 1. Conformance Rules Verified

| Rule Category | Check Details |
|---|---|
| **Price Sanity** | Rejects zero, negative, or overflow prices across all base quote pairs. |
| **Monotonic Time** | Asserts timestamps increase strictly monotonically across feed tick updates. |
| **Freshness & Heartbeat** | Flags stale prices when feed update interval exceeds configured maximum heartbeat. |
| **Spread & Parity** | Tests cross-pair triangular parity ($A/B \times B/C = A/C$) and bid-ask spreads. |
| **Outlier Rejection** | Evaluates price spike dampening and TWAP (Time-Weighted Average Price) validity. |

---

## 2. CLI Usage & Reporting Options

```bash
# Verify oracle metadata and price feed conformance
sepcheck40 verify --oracle <CONTRACT_ID> --format markdown

# Export audit scorecard to CSV for automated CI reporting
sepcheck40 verify --oracle <CONTRACT_ID> --format csv > audit.csv
```
