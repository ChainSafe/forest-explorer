# Claim Token API

**Base URL:** `https://forest-explorer.chainsafe.dev`  
**Endpoint:** `/api/claim_token`  
**HTTP Method:** `GET`

## Description

The Claim Token API provides a simple way to request calibnet tokens from the
faucet. This is primarily intended for developers and testers who need tokens to
interact with the network. Users provide a valid wallet address and specify the
token type (`faucet_info`) they wish to receive. On success, the API returns a
transaction ID confirming the token transfer.

**Key points:**

- Each address may be subject to rate limiting to prevent abuse.
- This API only distributes Calibnet `tFIL` and `tUSDFC` tokens.

---

## Query Parameters

| Parameter     | Type   | Required | Description                                                               |
| ------------- | ------ | -------- | ------------------------------------------------------------------------- |
| `faucet_info` | string | Yes      | The type of token to claim. Valid values: `CalibnetFIL`, `CalibnetUSDFC`. |
| `address`     | string | Yes      | The wallet address to receive the token.                                  |

---

## Status Codes

| Status Code | Description                                                  |
| ----------- | ------------------------------------------------------------ |
| 200         | Token successfully claimed; response contains transaction ID |
| 400         | Bad request - invalid address                                |
| 429         | Too many requests - rate limited                             |
| 500         | Server error; response contains error message                |
| 501         | Not implemented - mainnet not supported                      |

---

## Responses

### Success

- **Status:** `200 OK`
- **Content:** Plain JSON string containing the transaction ID.

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=CalibnetFIL&address=t1pxxbe7he3c6vcw5as3gfvq33kprpmlufgtjgfdq"
```

**Response:**

```bash
bafy2bzaceam3ihtqa73ru2bdvwoyaouwjwktsonkvs3rwwrn3z43e3xh3y4fk
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

#### 501 Not Implemented

- **Status:** `501 Not Implemented`
- **Content:** Plain string describing the error.

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=MainnetFIL&address=f1rgci272nfk4k6cpyejepzv4xstpejjckldlzidy"
```

**Response:**

```bash
ServerError|Mainnet token claim is not implemented.
```
