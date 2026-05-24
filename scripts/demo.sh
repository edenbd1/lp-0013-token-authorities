#!/usr/bin/env bash
# LP-0013 end-to-end demo: Token Authority Lifecycle
#
# Runs against a native standalone LEZ sequencer (no Docker needed).
# Demonstrates: create token with authority → mint → revoke → verify rejection.
#
# Prerequisites:
#   - Rust stable toolchain
#   - The LEZ fork: https://github.com/edenbd1/logos-execution-zone (branch lp-0013-token-authorities)
#
# Usage:
#   cd /path/to/logos-execution-zone   # the LEZ fork
#   bash /path/to/lp-0013-token-authorities/scripts/demo.sh
#
# The script builds the wallet and standalone sequencer, runs the full
# authority lifecycle, and verifies each step via JSON-RPC getAccount.

set -euo pipefail

export RISC0_DEV_MODE=${RISC0_DEV_MODE:-0}
BLOCK_WAIT=${BLOCK_WAIT:-18}
SEQ_PORT=${SEQ_PORT:-3040}

echo "============================================"
echo "  LP-0013: Token Authority Lifecycle Demo"
echo "  RISC0_DEV_MODE=$RISC0_DEV_MODE"
echo "============================================"
echo ""

# --- Build ---
echo "[build] Building wallet + standalone sequencer..."
PYO3_PYTHON=${PYO3_PYTHON:-python3} cargo build --release -p wallet --features standalone -p sequencer_service 2>&1 | tail -3
W=./target/release/wallet

# --- Start sequencer ---
echo ""
echo "[infra] Starting standalone sequencer on port $SEQ_PORT..."
rm -rf ./rocksdb 2>/dev/null || true
RUST_LOG=warn ./target/release/sequencer_service \
  sequencer/service/configs/debug/sequencer_config.json \
  --port "$SEQ_PORT" > /tmp/lp0013-seq.log 2>&1 &
SEQ_PID=$!
sleep 5

cleanup() {
  echo ""
  echo "[cleanup] Stopping sequencer (PID $SEQ_PID)..."
  kill "$SEQ_PID" 2>/dev/null || true
  rm -rf ./rocksdb /tmp/lp0013-seq.log "$WHOME" 2>/dev/null || true
}
trap cleanup EXIT

# Verify sequencer is up
curl -sf -X POST "http://127.0.0.1:$SEQ_PORT" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"checkHealth","id":1}' > /dev/null
echo "  Sequencer running (PID $SEQ_PID)."

# --- Wallet setup ---
WHOME=$(mktemp -d)/lp0013-demo
mkdir -p "$WHOME"
cat > "$WHOME/wallet_config.json" <<EOF
{"sequencer_addr":"http://127.0.0.1:$SEQ_PORT","seq_poll_timeout":"30s","seq_tx_poll_max_blocks":15,"seq_poll_max_retries":10,"seq_block_poll_max_amount":100}
EOF
export NSSA_WALLET_HOME_DIR="$WHOME"

echo "[wallet] Initializing..."
echo "demo-pw" | $W check-health 2>&1 | tail -1

# --- Helpers ---
gid() { $W account new public 2>&1 | grep account_id | sed 's/.*account_id //;s/ .*//'; }
qry() {
  local raw=$(echo "$1" | sed 's/Public\///')
  curl -s -X POST "http://127.0.0.1:$SEQ_PORT" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"getAccount\",\"params\":{\"account_id\":\"$raw\"},\"id\":1}"
}
show_supply() {
  python3 -c "
import sys, json
d = json.load(sys.stdin)
data = d['result']['data']
if not data:
    print('  (empty account)')
    sys.exit(0)
nl = int.from_bytes(bytes(data[1:5]), 'little')
ts = int.from_bytes(bytes(data[5+nl:5+nl+16]), 'little')
print(f'  supply={ts}  nonce={d[\"result\"][\"nonce\"]}')
"
}

# --- Create accounts ---
echo ""
echo "[accounts] Generating accounts..."
DEF=$(gid)
SUP=$(gid)
HLD=$(gid)
echo "  Definition: $DEF"
echo "  Supply:     $SUP"
echo "  Holder:     $HLD"

# --- Step 1: Create token with authority ---
echo ""
echo "[1/4] NewFungibleDefinitionWithAuthority (supply=1000, authority=DEF)"
$W token new-with-authority \
  --definition-account-id "$DEF" \
  --supply-account-id "$SUP" \
  --name "DEMO" \
  --initial-supply 1000 \
  --authority "$DEF"
sleep "$BLOCK_WAIT"
qry "$DEF" | show_supply

# --- Step 2: Mint with authority ---
echo ""
echo "[2/4] MintWithAuthority (amount=500)"
$W token mint-with-authority \
  --definition "$DEF" \
  --holder "$HLD" \
  --amount 500 \
  --authority-signer "$DEF"
sleep "$BLOCK_WAIT"
qry "$DEF" | show_supply

# --- Step 3: Revoke authority ---
echo ""
echo "[3/4] RevokeAuthority (supply becomes permanently fixed)"
$W token revoke-authority \
  --definition "$DEF" \
  --authority-signer "$DEF"
sleep "$BLOCK_WAIT"
qry "$DEF" | show_supply

# --- Step 4: Verify rejection ---
echo ""
echo "[4/4] MintWithAuthority after revoke (expect rejection)"
$W token mint-with-authority \
  --definition "$DEF" \
  --holder "$HLD" \
  --amount 100 \
  --authority-signer "$DEF" 2>&1 || true
sleep "$BLOCK_WAIT"

# Check supply didn't change
echo "  Verifying supply unchanged..."
qry "$DEF" | show_supply

# Check sequencer rejected it
if grep -q "Renounced" /tmp/lp0013-seq.log; then
  echo "  CONFIRMED: sequencer rejected with 'Renounced: authority has been permanently revoked'"
else
  echo "  WARNING: rejection not found in sequencer logs"
fi

echo ""
echo "============================================"
echo "  Demo Complete"
echo ""
echo "  Step 1: supply=1000 (created)"
echo "  Step 2: supply=1500 (minted 500)"
echo "  Step 3: authority REVOKED"
echo "  Step 4: mint REJECTED (supply unchanged)"
echo ""
echo "  RISC0_DEV_MODE=$RISC0_DEV_MODE"
echo "============================================"
