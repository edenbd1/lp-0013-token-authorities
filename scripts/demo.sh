#!/usr/bin/env bash
# LP-0013 end-to-end demo: Token Authority Lifecycle
# Runs against a local LEZ sequencer (docker-compose) with RISC0_DEV_MODE=0 (real STARK proofs).
#
# Lifecycle: create token with authority -> mint -> rotate -> mint with new admin
#            -> revoke -> verify mint rejected
set -euo pipefail

export RISC0_DEV_MODE=${RISC0_DEV_MODE:-0}

# Time to wait between transactions for block confirmation (seconds).
BLOCK_WAIT=${BLOCK_WAIT:-8}

echo "============================================"
echo "  LP-0013: Token Authority Lifecycle Demo"
echo "  RISC0_DEV_MODE=$RISC0_DEV_MODE"
echo "============================================"
echo ""

# --- Build ---
echo "[build] Building wallet in release mode..."
cargo build --release -p wallet 2>&1 | tail -3
WALLET=./target/release/wallet

# --- Wallet home setup ---
# The wallet needs a config directory with wallet_config.json and will create storage.json
# on first use (prompting for a password via stdin).
DEMO_WALLET_HOME="$(mktemp -d)/lp0013-demo-wallet"
mkdir -p "$DEMO_WALLET_HOME"
export NSSA_WALLET_HOME_DIR="$DEMO_WALLET_HOME"

# Copy the debug config (sequencer at 127.0.0.1:3040) into our ephemeral wallet home.
cp wallet/configs/debug/wallet_config.json "$DEMO_WALLET_HOME/wallet_config.json"

echo "[wallet] Wallet home: $DEMO_WALLET_HOME"

# --- Start sequencer ---
echo ""
echo "[infra] Starting LEZ sequencer via docker-compose..."
docker compose up -d --wait 2>&1 | tail -5
sleep 5

cleanup() {
  echo ""
  echo "[cleanup] Stopping sequencer..."
  docker compose down 2>/dev/null || true
  echo "[cleanup] Removing ephemeral wallet home..."
  rm -rf "$DEMO_WALLET_HOME"
}
trap cleanup EXIT

# --- Helper: run wallet (auto-init with piped password on first use) ---
# The wallet reads a password from stdin on first run (when storage.json is absent).
# After init, subsequent commands do not prompt.
WALLET_INITIALIZED=false
run_wallet() {
  if [ "$WALLET_INITIALIZED" = false ]; then
    echo "demo-password" | $WALLET "$@"
    WALLET_INITIALIZED=true
  else
    $WALLET "$@"
  fi
}

# --- Health check ---
echo ""
echo "[health] Checking wallet can connect to sequencer..."
run_wallet check-health 2>&1 | tail -3
echo "  Wallet initialized and connected."

# --- Generate accounts ---
# We need 4 public accounts:
#   1) definition  - token definition account
#   2) supply      - token initial supply holder
#   3) holder      - receives minted tokens
#   4) new_admin   - will become the new authority after rotation
echo ""
echo "[accounts] Generating 4 public accounts..."

extract_account_id() {
  # Parse: "Generated new account with account_id Public/<ID> at path ..."
  # Return the raw base58 ID (without Public/ prefix) for display,
  # but we'll use the Public/<ID> form in commands.
  sed -n 's/.*Public\/\([A-Za-z0-9]*\).*/\1/p' | head -1
}

DEF_ACCOUNT=$(run_wallet account new public 2>&1 | extract_account_id)
sleep 1
SUPPLY_ACCOUNT=$(run_wallet account new public 2>&1 | extract_account_id)
sleep 1
HOLDER_ACCOUNT=$(run_wallet account new public 2>&1 | extract_account_id)
sleep 1
NEW_ADMIN_ACCOUNT=$(run_wallet account new public 2>&1 | extract_account_id)

echo "  Definition:   Public/$DEF_ACCOUNT"
echo "  Supply:       Public/$SUPPLY_ACCOUNT"
echo "  Holder:       Public/$HOLDER_ACCOUNT"
echo "  New Admin:    Public/$NEW_ADMIN_ACCOUNT"

# --- Step 1: Create token with authority ---
echo ""
echo "[1/6] Creating token 'DEMO' with initial supply 1000, authority = Public/$DEF_ACCOUNT..."
run_wallet token new-with-authority \
  --definition-account-id "Public/$DEF_ACCOUNT" \
  --supply-account-id "Public/$SUPPLY_ACCOUNT" \
  --name "DEMO" \
  --initial-supply 1000 \
  --authority "Public/$DEF_ACCOUNT" 2>&1 | tail -3
echo "  Done."
sleep "$BLOCK_WAIT"

# --- Step 2: Mint with authority ---
echo ""
echo "[2/6] Minting 500 tokens to holder..."
run_wallet token mint-with-authority \
  --definition "Public/$DEF_ACCOUNT" \
  --holder "Public/$HOLDER_ACCOUNT" \
  --amount 500 \
  --authority-signer "Public/$DEF_ACCOUNT" 2>&1 | tail -3
echo "  Done."
sleep "$BLOCK_WAIT"

# --- Step 3: Rotate authority ---
echo ""
echo "[3/6] Rotating authority from Public/$DEF_ACCOUNT to Public/$NEW_ADMIN_ACCOUNT..."
run_wallet token rotate-authority \
  --definition "Public/$DEF_ACCOUNT" \
  --new-authority "Public/$NEW_ADMIN_ACCOUNT" \
  --authority-signer "Public/$DEF_ACCOUNT" 2>&1 | tail -3
echo "  Done."
sleep "$BLOCK_WAIT"

# --- Step 4: New admin mints ---
echo ""
echo "[4/6] New admin mints 300 tokens..."
run_wallet token mint-with-authority \
  --definition "Public/$DEF_ACCOUNT" \
  --holder "Public/$HOLDER_ACCOUNT" \
  --amount 300 \
  --authority-signer "Public/$NEW_ADMIN_ACCOUNT" 2>&1 | tail -3
echo "  Done."
sleep "$BLOCK_WAIT"

# --- Step 5: Revoke authority ---
echo ""
echo "[5/6] Revoking authority (supply becomes permanently fixed)..."
run_wallet token revoke-authority \
  --definition "Public/$DEF_ACCOUNT" \
  --authority-signer "Public/$NEW_ADMIN_ACCOUNT" 2>&1 | tail -3
echo "  Done."
sleep "$BLOCK_WAIT"

# --- Step 6: Verify mint fails after revocation ---
echo ""
echo "[6/6] Attempting mint after revocation (should fail)..."
if run_wallet token mint-with-authority \
  --definition "Public/$DEF_ACCOUNT" \
  --holder "Public/$HOLDER_ACCOUNT" \
  --amount 100 \
  --authority-signer "Public/$NEW_ADMIN_ACCOUNT" 2>/dev/null; then
  echo "  ERROR: Mint should have been rejected!"
  exit 1
else
  echo "  OK: Mint correctly rejected -- authority permanently revoked."
fi

echo ""
echo "============================================"
echo "  Demo Complete"
echo "  Final state: supply = 1800 (1000 + 500 + 300)"
echo "  Authority: REVOKED (permanent)"
echo "  RISC0_DEV_MODE=$RISC0_DEV_MODE"
echo "============================================"
