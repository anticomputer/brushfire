#!/bin/bash
# Test script to verify args are captured in webhook events

# Clean up any existing servers
pkill -f "test-webhook-server.py" 2>/dev/null
sleep 1

# Start webhook server with unbuffered output
python3 -u test-webhook-server.py &
WEBHOOK_PID=$!
sleep 2

echo "Testing command spawn with arguments..."
cargo run -q -p brush-shell --features policy-webhook -- \
  --profile test-profiles/strict-whitelist.profile \
  --policy-webhook http://localhost:8765 \
  --wrap-coreutils \
  -c 'cat -n /tmp/test-args.txt' 2>&1 | head -5

# Give webhook time to flush
sleep 2

# Cleanup
kill $WEBHOOK_PID 2>/dev/null
wait $WEBHOOK_PID 2>/dev/null

echo ""
echo "Test complete"
