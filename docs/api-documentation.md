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

| Status Code | Description                                                            |
| ----------- | ---------------------------------------------------------------------- |
| 200         | Token successfully claimed; response contains transaction ID           |
| 500         | Server error, including rate limiting; response contains error message |

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

### Error

- **Status:** `500 Internal Server Error`
- **Content:** Plain string describing the error.

**Example:**

```bash
curl "https://forest-explorer.chainsafe.dev/api/claim_token?faucet_info=CalibnetFIL&address=invalidaddress"
```

**Response:**

```bash
ServerError|Not a valid Testnet address
```
