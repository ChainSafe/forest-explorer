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

---

## Query Parameters

| Parameter     | Type   | Required | Description                                                               |
| ------------- | ------ | -------- | ------------------------------------------------------------------------- |
| `faucet_info` | string | Yes      | The type of token to claim. Valid values: `CalibnetFIL`, `CalibnetUSDFC`. |
| `address`     | string | Yes      | The wallet address to receive the token.                                  |

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

#### Success claim for `CalibnetFIL`

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

#### Success claim for `CalibnetUSDFC`

- **Status:** `200 OK`
- **Content:** Plain text string containing the transaction hash.

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=CalibnetUSDFC&address=0xae9c4b9508c929966ef37209b336e5796d632cdc"
```

**Response:**

```bash
0x8d75e2394dcf829ab9353370069b6d6afb04c88ea38c765ab4443a1587e12922
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

# Claim Token All API

**Base URL:** `https://forest-explorer.chainsafe.dev`  
**Endpoint:** `/api/claim_token_all`  
**HTTP Method:** `GET`

## Description

Requests claims for both `CalibnetUSDFC` and `CalibnetFIL` in one call. Returns
a JSON array of per-claim results. Each item corresponds to one faucet claim.

---

## Query Parameters

| Parameter | Type   | Required | Description                                   |
| --------- | ------ | -------- | --------------------------------------------- |
| `address` | string | Yes      | The wallet address to receive all the tokens. |

---

## Status Codes

| Status Code | Description                      |
| ----------- | -------------------------------- |
| 200         | Tokens successfully claimed      |
| 400         | Bad request - invalid address    |
| 429         | Too many requests - rate limited |
| 500         | Server error                     |

---

### Claim Response Success & Failure

The API returns a **JSON array**, where each object corresponds to a faucet
claim attempt. Each object contains:

- `faucet_info`: A string identifying the faucet (e.g., `CalibnetFIL`,
  `CalibnetUSDFC`)

And either:

- `tx_hash`: A string containing the transaction hash **if the claim was
  successful**,  
  **or**
- `error`: An object containing the error details **if the claim failed**

---

## Examples

### Success

- **Status:** `200 OK`

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token_all?address=0xAe9C4b9508c929966ef37209b336E5796D632CDc"
```

**Response:**

```json
[
  {
    "faucet_info": "CalibnetUSDFC",
    "tx_hash": "0x8d75e2394dcf829ab9353370069b6d6afb04c88ea38c765ab4443a1587e12922"
  },
  {
    "faucet_info": "CalibnetFIL",
    "tx_hash": "0xf133c6aae45e40a48b71449229cb45f5ab5f2e7bd8ae488d1142319191ca8eb0"
  }
]
```

### Failure

#### 400 Bad Request

- **Status:** `400 Bad Request`
- **Content:** JSON array where each item represents a faucet claim result. Each
  item includes `faucet_info` and either a `tx_hash` (on success) or an `error`
  object (on failure).

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=CalibnetFIL&address=invalidaddress"
```

**Response:**

```bash
[
    {
        "faucet_info": "CalibnetUSDFC",
        "error": {
            "ServerError": "Invalid address: Not a valid Testnet address"
        }
    },
    {
        "faucet_info": "CalibnetFIL",
        "error": {
            "ServerError": "Invalid address: Not a valid Testnet address"
        }
    }
]
```

#### 429 Too Many Requests

- **Status:** `429 Too Many Requests`
- **Content:** JSON array where each item represents a faucet claim result. Each
  item includes `faucet_info` and either a `tx_hash` (on success) or an `error`
  object (on failure).

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=CalibnetFIL&address=0xAe9C4b9508c929966ef37209b336E5796D632CDc"
```

**Response:**

```bash
[
    {
        "faucet_info": "CalibnetUSDFC",
        "error": {
            "ServerError": "Too many requests: Rate limited. Try again in 46 seconds."
        }
    },
    {
        "faucet_info": "CalibnetFIL",
        "error": {
            "ServerError": "Too many requests: Rate limited. Try again in 12 seconds."
        }
    }
]
```

---

**Key points:**

- Each address is subject to rate limiting to prevent abuse.
- This API only distributes Calibnet `tFIL` and `tUSDFC` tokens.

## Rate Limits

| Faucet Type     | Cooldown Period | Drip Amount | Wallet Cap | Global Cap   |
| --------------- | --------------- | ----------- | ---------- | ------------ |
| `CalibnetFIL`   | 60 seconds      | 1 tFIL      | 2 tFIL     | 200 tFIL     |
| `CalibnetUSDFC` | 60 seconds      | 5 tUSDFC    | 10 tUSDFC  | 1,000 tUSDFC |

**Note:** All limits reset every 24 hours. Abuse, farming, or automated requests
are prohibited and may result in stricter limits or bans.

---

## Faucet Top-Up Requests

If you encounter a server error indicating that faucet is exhausted.

**Example:**

```bash
ServerError|Faucet is empty, Request top-up
```

You can request for faucet top-up
[Here](https://github.com/ChainSafe/forest-explorer/discussions/134). This
discussion thread is monitored for top-up requests.
