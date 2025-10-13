# Claim Token API

**Base URL:** `https://forest-explorer.chainsafe.dev`  
**Endpoint:** `/api/claim_token`  
**HTTP Method:** `GET`

## Description

The Claim Token API provides a simple way to request calibnet tokens from the
faucet. This is primarily intended for developers and testers who need tokens to
interact with the network. Users provide a valid wallet address and specify the
token type (`faucet_info`) they wish to receive. On success, the API returns a
transaction hash confirming the token transfer.

**Key points:**

- Each address is subject to rate limiting to prevent abuse.
- This API only distributes Calibnet `tFIL` and `tUSDFC` tokens.

---

## Query Parameters

| Parameter     | Type   | Required | Description                                                               |
| ------------- | ------ | -------- | ------------------------------------------------------------------------- |
| `faucet_info` | string | Yes      | The type of token to claim. Valid values: `CalibnetFIL`, `CalibnetUSDFC`. |
| `address`     | string | Yes      | The wallet address to receive the token.                                  |

---

## Rate Limits

| Faucet Type     | Cooldown Period | Drip Amount | Wallet Cap | Global Cap   |
| --------------- | --------------- | ----------- | ---------- | ------------ |
| `CalibnetFIL`   | 60 seconds      | 1 tFIL      | 2 tFIL     | 200 tFIL     |
| `CalibnetUSDFC` | 60 seconds      | 5 tUSDFC    | 10 tUSDFC  | 1,000 tUSDFC |

**Note:** All limits reset every 24 hours. Abuse, farming, or automated requests
are prohibited and may result in stricter limits or bans.

---

## Status Codes

| Status Code | Description                                                    |
| ----------- | -------------------------------------------------------------- |
| 200         | Token successfully claimed; response contains transaction hash |
| 400         | Bad request - invalid address                                  |
| 429         | Too many requests - rate limited                               |
| 500         | Server error; response contains error message                  |
| 418         | I'm a teapot - mainnet not supported                           |

---

## Examples

### Success

- **Status:** `200 OK`
- **Content:** Plain text string containing the transaction hash.

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=CalibnetFIL&address=t1pxxbe7he3c6vcw5as3gfvq33kprpmlufgtjgfdq"
```

**Response:**

```bash
0x06784dd239f7f0e01baa19a82877e17b7fcd6e1dd725913fd6f741a2a6c56ce5
```

### Failure

#### 400 Bad Request

- **Status:** `400 Bad Request`
- **Content:** Plain string describing the error.

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=CalibnetFIL&address=invalidaddress"
```

**Response:**

```bash
ServerError|Invalid address: Not a valid Testnet address
```

#### 429 Too Many Requests

- **Status:** `429 Too Many Requests`
- **Content:** Plain string describing the rate limit error.

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=CalibnetFIL&address=t1pxxbe7he3c6vcw5as3gfvq33kprpmlufgtjgfdq"
```

**Response:**

```bash
ServerError|Too many requests: Rate limited. Try again in 60 seconds.
```

#### 500 Internal Server Error

- **Status:** `500 Internal Server Error`
- **Content:** Plain string describing the server error.

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=Calibnet&address=t1pxxbe7he3c6vcw5as3gfvq33kprpmlufgtjgfdq"
```

**Response:**

```bash
Args|unknown variant `Calibnet`, expected one of `MainnetFIL`, `CalibnetFIL`, `CalibnetUSDFC`
```

#### 418 I'm a Teapot

- **Status:** `418 I'm a Teapot`
- **Content:** Plain string describing the error.

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=MainnetFIL&address=f1rgci272nfk4k6cpyejepzv4xstpejjckldlzidy"
```

**Response:**

```bash
ServerError|I'm a teapot - mainnet tokens are not available.
```

---

## Faucet Top-Up Requests

If you encounter a server error indicating that faucet is exhausted (e.g.,
"ServerError|Faucet is empty, Request top-up"), you can request a refill here:

- [Request Faucet Top-Up](https://github.com/ChainSafe/forest-explorer/discussions/134)

This discussion thread is monitored for top-up requests.
