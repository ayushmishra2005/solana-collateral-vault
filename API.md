# API Documentation

## Base URL

```
http://localhost:3000
```

## Endpoints

### POST /vault/initialize
Create new vault for user.

**Request:**
```json
{"user": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"}
```

**Response:**
```json
{"success": true, "signature": "5j7s8K9..."}
```

### POST /vault/deposit
Deposit collateral into vault.

**Request:**
```json
{"user": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU", "amount": 1000000}
```

### POST /vault/withdraw
Withdraw collateral from vault.

**Request:**
```json
{"user": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU", "amount": 500000}
```

### GET /vault/balance/:user
Get vault balance information.

**Response:**
```json
{
  "owner": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "vault": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
  "total_balance": 1000000,
  "locked_balance": 200000,
  "available_balance": 800000
}
```

### GET /vault/transactions/:user
Get transaction history. Query: `?limit=100`

### GET /vault/tvl
Get total value locked.

**Response:**
```json
{"total_value_locked": 50000000, "total_vaults": 1250, "timestamp": 1699123456}
```

## WebSocket

Connect to `ws://localhost:3000/ws` for real-time updates:
- Balance updates
- Deposit/withdrawal notifications
- TVL updates

