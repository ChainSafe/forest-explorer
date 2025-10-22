# Forest Explorer E2E Testing Guide

## Key Terms

**Durable Object (DO)**: A Cloudflare Worker with persistent state, used here to
manage rate limits and wallet caps across requests.

**Filecoin Address Types**: Different formats representing the same underlying
wallet:

- `t1`/`f1` (secp256k1): Traditional Filecoin addresses
- `t410`/`f410` (EVM): Ethereum-compatible addresses within Filecoin
- `eth` (0x): Standard Ethereum addresses
- `t0`/`f0` (ID): Numeric actor IDs corresponding to other address formats

**Calibnet**: Filecoin's calibration testnet for development and testing.

## What We're Testing

### Browser vs API Split

- **Browser Tests** (`e2e/script.js`): Full user journey through the web
  interface.
- **API Tests** (`e2e/test_claim_token_api.js`): Direct validation of the
  `/api/claim_token` endpoint.

### Faucet API Core Functionality

Testing the token claim API for different Filecoin networks:

- **CalibnetFIL**: Calibration network FIL tokens.
- **CalibnetUSDFC**: Calibration network USDC tokens.
- **MainnetFIL**: Mainnet FIL tokens (read-only validation).

### Rate Limiting & Wallet Cap Enforcement

- **Per-faucet cooldown**: A 60-second minimum between requests to the same
  faucet type.
- **Per-wallet limits**: A maximum of two successful claims per wallet address.
- **Independent faucets**: CalibnetFIL and CalibnetUSDFC have separate rate
  limits.

## How We Test in Parallel Without Interference

### Matrix Strategy with Isolated State

```yaml
strategy:
  matrix:
    include:
      - name: "Browser Tests"
        state_dir: "browser"
      - name: "API Tests"
        state_dir: "api"
```

Each test suite runs with isolated Durable Object storage:

```bash
yarn wrangler dev --port ${matrix.port} --persist-to .wrangler-state-${state_dir}-${github.run_id}
```

This prevents the rate-limiting state from one test from affecting the other.

## API Test Flow

### 1. Connectivity Check

Before running tests, `validateServerConnectivity()` confirms the server is
responsive. If this check fails, all subsequent tests are aborted.

### 2. Input Validation Tests

This section tests invalid parameters and malformed addresses using the
`INVALID_REQUESTS` scenarios:

- Missing parameters (`faucet_info: null`, `address: null`).
- Invalid faucet types (`InvalidFaucet`).
- Malformed addresses (see the comprehensive edge cases in
  `TEST_ADDRESSES.INVALID` within the configuration file).

Each case asserts the specific HTTP status defined in `INVALID_REQUESTS`,
covering 500, 418, and 400 responses.

### 3. Rate Limit Cooldown Tests (`RATE_LIMIT_TEST_COOLDOWN_CASES`)

This sequence verifies that once a successful request is made, all subsequent
requests for any address type on that same faucet are rate-limited until the
60-second cooldown expires.

**CalibnetFIL Sequence:**

1. `CalibnetFIL (t1) - 1st SUCCESS` → 200 (starts 60s cooldown)
2. `CalibnetFIL (t410) - RATE LIMITED` → 429 (within cooldown)
3. `CalibnetFIL (eth) - RATE LIMITED` → 429 (within cooldown)
4. Additional address formats → All 429 (within cooldown)

**CalibnetUSDFC Sequence (independent cooldown):**

1. `CalibnetUSDFC (eth) - 1st SUCCESS` → 200 (starts separate 60s cooldown)
2. `CalibnetUSDFC (t410) - RATE LIMITED` → 429 (within USDFC cooldown)
3. Additional formats → All 429 (within USDFC cooldown)

### 4. Wallet Cap Tests (`RATE_LIMIT_TEST_WALLET_CAP_CASES`)

This sequence validates the 2-drip per-wallet limit by testing wallets that
reach their maximum claims, then verifying that all equivalent address formats
for the same wallet are also capped. The test sequences use a `waitBefore: 65`
second delay between claims to ensure the 60-second cooldown does not interfere
with wallet cap validation.

**CalibnetFIL t1 wallet (already has 1 drip from cooldown tests):**

1. Wait 65s → `2nd SUCCESS` (200, reaches cap)
2. Wait 65s → `3rd attempt WALLET CAPPED` (429, >1h retry time)

**CalibnetFIL eth wallet (fresh for this faucet):**

1. Wait 65s → `1st SUCCESS` (200)
2. Wait 65s → `2nd SUCCESS` (200, reaches cap)
3. Wait 65s → `3rd attempt WALLET CAPPED` (429, >1h retry time)
4. Test equivalent addresses (`t410`, `t0`, `ID`) → All 429 (same wallet)

**CalibnetUSDFC eth wallet (already has 1 drip from cooldown tests):**

1. Wait 65s → `2nd SUCCESS` (200, reaches cap)
2. Wait 65s → `3rd attempt WALLET CAPPED` (429, >1h retry time)
3. Test equivalent address (`t410`) → 429 (same wallet)

### 5. Common Helper Functions

**`makeClaimRequest(faucetInfo, address)`**: A centralized API call that handles
null parameters gracefully and provides consistent timeouts and headers.

**`validateTransactionHash(txHash)`**: Validates that successful responses
return a proper Ethereum-formatted transaction hash (0x + 64 hex chars).

**`runTestScenarios(scenarios, options)`**: Executes test arrays with optional
`waitBefore` delays and fail-fast logic that skips additional checks if the
primary status assertion fails.

### Test Configuration Reference

All test scenarios and edge cases are defined in
`test_claim_token_api_config.js`:

- `INVALID_REQUESTS`: Parameter validation cases.
- `RATE_LIMIT_TEST_COOLDOWN_CASES`: 60-second cooldown enforcement.
- `RATE_LIMIT_TEST_WALLET_CAP_CASES`: 2-drip wallet limit enforcement.
- `TEST_ADDRESSES.INVALID`: A comprehensive corpus of malformed addresses.
