# CORS Testing Guide

## Key Terms

**CORS (Cross-Origin Resource Sharing)**: Browser security mechanism that
controls whether web pages from one domain can access resources from another
domain.

**Preflight Request**: An OPTIONS request browsers send before the actual
request to check if the cross-origin request is permitted.

**Origin Header**: Identifies the domain making the request (e.g.,
`https://app.example.com`). Automatically set by browsers, cannot be overridden
in JavaScript.

**Access-Control-Allow-\* Headers**: Server response headers that tell browsers
which cross-origin requests are permitted.

---

## What We're Testing

### CORS Configuration for Public API Access

Testing that the `/api/claim_token` endpoint properly supports cross-origin
requests from any domain:

- **CalibnetFIL, CalibnetUSDFC, MainnetFIL** faucets accessible from external
  websites
- **Browser-based DApps** can integrate with the faucet API
- **Third-party integrations** work without CORS errors

### Current Configuration

```rust
// src/lib.rs
let cors = CorsLayer::new()
    .allow_origin(Any)           // Accept requests from ANY origin
    .allow_methods(Method::GET); // Only GET requests allowed
```

**Result**: `Access-Control-Allow-Origin: *` header on all responses

**Note**: We don't set `allow_headers()` because GET requests only use "simple
headers" (like `Accept`, `User-Agent`) which don't require CORS preflight
permission.

---

## How We Test in CI

### Matrix Strategy with Isolated State

```yaml
- name: "CORS Tests"
  script: "e2e/test_cors.js"
  port: 8788 # Separate port from Browser/API tests
  k6_browser_enabled: false # No browser support needed
  state_dir: "cors" # Isolated Durable Object state
```

Each test suite runs independently:

```bash
yarn wrangler dev --port 8788 --persist-to .wrangler-state-cors-${github.run_id}
```

This prevents CORS test state from interfering with Browser or API tests.

---

## Test Flow

### 1. Preflight OPTIONS Request (5 checks)

Validates that browsers can check permissions before sending actual requests:

```javascript
// Request
OPTIONS /api/claim_token
Origin: https://external-example.com
Access-Control-Request-Method: GET

// Validates
✓ Status is 200 or 204
✓ Has Access-Control-Allow-Origin header
✓ Allows all origins (*)
✓ Has Access-Control-Allow-Methods header
✓ Allows GET method
```

### 2. Actual Cross-Origin GET Request (3 checks)

Confirms the API properly responds to cross-origin requests:

```javascript
// Request
GET /api/claim_token?faucet_info=CalibnetFIL&address=0x...
Origin: https://external-example.com

// Validates
✓ Has Access-Control-Allow-Origin in response
✓ CORS header allows all origins (*)
✓ Response received (not blocked by CORS)
```

### 3. Same-Origin Request (2 checks)

Verifies CORS headers are present even without explicit origin:

```javascript
// Request (no Origin header)
GET /api/claim_token?faucet_info=CalibnetFIL&address=0x...

// Validates
✓ Request succeeds (200 or 429)
✓ Has Access-Control-Allow-Origin header
```

### 4. Multiple Origins (9 checks)

Tests that different domains can all access the API:

```javascript
// Tests three different origins:
- https://app.example.com
- http://localhost:3000
- https://wallet.filecoin.io

// For each origin:
✓ Request succeeds
✓ CORS allows origin
✓ Response received
```

### 5. Security & Edge Cases (3 checks)

Validates security headers and error handling:

```javascript
// Security
✓ Access-Control-Allow-Credentials is NOT set (safer for public APIs)

// Error responses
✓ CORS headers present even on 500/400 errors (critical for browser error handling)
```

**Total: 22 checks covering full CORS compliance**

---

## Running Tests Locally

```bash
# Start server
yarn wrangler dev --port 8787

# Run CORS tests
API_URL="http://127.0.0.1:8787" k6 run e2e/test_cors.js
```

### Expected Output (All Passing)

```
checks_succeeded...: 100.00% 22 out of 22

✓ Preflight: Status is 200 or 204
✓ Preflight: Has Access-Control-Allow-Origin header
✓ Preflight: Allows all origins (*)
✓ Preflight: Has Access-Control-Allow-Methods
✓ Preflight: Allows GET method
✓ Actual Request: Has Access-Control-Allow-Origin in response
✓ Actual Request: CORS header allows all origins
✓ Actual Request: Response received (not blocked by CORS)
✓ Same-Origin: Request succeeds
✓ Same-Origin: Has Access-Control-Allow-Origin (even for same-origin)
✓ Multiple Origins: https://app.example.com - Request succeeds
✓ Multiple Origins: https://app.example.com - CORS allows origin
✓ Multiple Origins: https://app.example.com - Response received
✓ Multiple Origins: http://localhost:3000 - Request succeeds
✓ Multiple Origins: http://localhost:3000 - CORS allows origin
✓ Multiple Origins: http://localhost:3000 - Response received
✓ Multiple Origins: https://wallet.filecoin.io - Request succeeds
✓ Multiple Origins: https://wallet.filecoin.io - CORS allows origin
✓ Multiple Origins: https://wallet.filecoin.io - Response received
✓ Security: Has Access-Control-Allow-Origin
✓ Security: Access-Control-Allow-Credentials is not set
✓ Error Response: CORS headers present even on errors
```

**Note**: Total of 22 checks (5 preflight + 3 actual + 2 same-origin + 9
multiple-origins + 3 security/edge cases)

### Common Test Results

**All CORS checks pass, but some "Request succeeds" fail:**

- ✅ CORS is working correctly
- ⚠️ Server returning 500 errors (unrelated to CORS)
- Check server logs for the actual API error

**CORS header checks failing:**

- ❌ Configuration issue in `src/lib.rs`
- Verify `allow_origin(Any)` and `allow_headers(Any)` are set
- Confirm `CorsLayer` is applied to the router

---

## Quick Manual Test

### Browser DevTools (Simplest)

1. Open browser console (F12) on any page
2. Run:

```javascript
fetch(
  "http://127.0.0.1:8787/api/claim_token?faucet_info=CalibnetFIL&address=f1pxxbe7he3c6vcw5as3gfvq33kprpmlufgtjgfdq",
)
  .then((r) => console.log("✅ CORS works! Status:", r.status))
  .catch((err) => console.error("❌ CORS blocked:", err));
```

### curl (For CI/Scripts)

```bash
# Test preflight
curl -i -X OPTIONS \
  -H "Origin: https://example.com" \
  -H "Access-Control-Request-Method: GET" \
  http://127.0.0.1:8787/api/claim_token

# Look for: Access-Control-Allow-Origin: *
```

---

## Why CORS Matters

### Before CORS (Blocked)

```javascript
// From https://my-dapp.com
fetch("https://faucet.chainsafe.io/api/claim_token?...");
// ❌ Error: CORS policy: No 'Access-Control-Allow-Origin' header
```

### After CORS (Allowed)

```javascript
// From https://my-dapp.com
fetch("https://faucet.chainsafe.io/api/claim_token?...");
// ✅ Success: Response includes "Access-Control-Allow-Origin: *"
```

**Security**: Protected by rate limiting (60s cooldown, 2-drip wallet cap), not
CORS restrictions.
