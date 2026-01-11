#!/bin/bash

# Complete API Testing Script
# Replace with your wallet address (get it with: solana address)
# For local testing, this must match the wallet running the backend server
ADMIN_PUBKEY="${ADMIN_PUBKEY:-9WUUr2WNUiKMzwxJgbb4oxS81oYAyhrBFkv3NSg2mjbj}"

echo "=========================================="
echo "Solana Collateral Vault - All API Endpoints"
echo "=========================================="
echo ""

# 1. Initialize Vault
echo "=== 1. Initialize Vault ==="
echo "POST /vault/initialize"
curl -X POST http://localhost:3000/vault/initialize \
  -H "Content-Type: application/json" \
  -d "{\"user\": \"$ADMIN_PUBKEY\"}"
echo -e "\n\n"

# 2. Get Vault Balance
echo "=== 2. Get Vault Balance ==="
echo "GET /vault/balance/{user}"
curl http://localhost:3000/vault/balance/$ADMIN_PUBKEY
echo -e "\n\n"

# 3. Get TVL
echo "=== 3. Get Total Value Locked (TVL) ==="
echo "GET /vault/tvl"
curl http://localhost:3000/vault/tvl
echo -e "\n\n"

# 4. Get Transaction History
echo "=== 4. Get Transaction History ==="
echo "GET /vault/transactions/{user}"
curl http://localhost:3000/vault/transactions/$ADMIN_PUBKEY
echo -e "\n\n"

# 5. Deposit Tokens (1000 tokens = 1000000000 with 6 decimals)
echo "=== 5. Deposit Tokens ==="
echo "POST /vault/deposit"
echo "Depositing 1000 tokens..."
curl -X POST http://localhost:3000/vault/deposit \
  -H "Content-Type: application/json" \
  -d "{\"user\": \"$ADMIN_PUBKEY\", \"amount\": 1000000000}"
echo -e "\n\n"

# 6. Get Balance After Deposit
echo "=== 6. Get Balance After Deposit ==="
curl http://localhost:3000/vault/balance/$ADMIN_PUBKEY
echo -e "\n\n"

# 7. Withdraw Tokens (500 tokens = 500000000 with 6 decimals)
echo "=== 7. Withdraw Tokens ==="
echo "POST /vault/withdraw"
echo "Withdrawing 500 tokens..."
curl -X POST http://localhost:3000/vault/withdraw \
  -H "Content-Type: application/json" \
  -d "{\"user\": \"$ADMIN_PUBKEY\", \"amount\": 500000000}"
echo -e "\n\n"

# 8. Get Balance After Withdraw
echo "=== 8. Get Balance After Withdraw ==="
curl http://localhost:3000/vault/balance/$ADMIN_PUBKEY
echo -e "\n\n"

# 9. Get All Transactions
echo "=== 9. Get All Transactions ==="
curl http://localhost:3000/vault/transactions/$ADMIN_PUBKEY
echo -e "\n\n"

# 10. Get TVL (Updated)
echo "=== 10. Get TVL (Updated) ==="
curl http://localhost:3000/vault/tvl
echo -e "\n\n"

echo "=========================================="
echo "All API Tests Complete!"
echo "=========================================="

