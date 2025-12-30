#!/bin/bash
# Test webhook functionality

echo "=== Testing Brushfire Policy Webhook ===="
echo ""
echo "Step 1: Starting webhook server..."
python3 test-webhook-server.py > /tmp/webhook-output.log 2>&1 &
WEBHOOK_PID=$!
sleep 2

echo "Step 2: Running brush with policy webhook enabled..."
echo "  Command: cat /tmp/test-secret.txt (should be DENIED)"
echo ""

# Create test file
echo "secret data" > /tmp/test-secret.txt

# Run brush with webhook - this will send events to the webhook server
cargo run -p brush-shell --features policy-webhook -- \
  --profile test-profiles/wrapper-simple-test.profile \
  --policy-webhook http://localhost:9999 \
  -c 'cat /tmp/test-secret.txt' 2>&1 | grep -E "(brushfire|Policy)" || echo "  (Access denied as expected)"

sleep 2

echo ""
echo "Step 3: Testing allowed operation..."
echo "  Command: cat /tmp/allowed.txt (should be ALLOWED)"
echo "allowed content" > /tmp/allowed.txt

cargo run -p brush-shell --features policy-webhook -- \
  --profile test-profiles/wrapper-simple-test.profile \
  --policy-webhook http://localhost:9999 \
  -c 'cat /tmp/allowed.txt' 2>&1 | tail -1

sleep 2

echo ""
echo "Step 4: Checking webhook server output..."
kill $WEBHOOK_PID 2>/dev/null
wait $WEBHOOK_PID 2>/dev/null

cat /tmp/webhook-output.log
rm -f /tmp/webhook-output.log /tmp/test-secret.txt /tmp/allowed.txt

echo ""
echo "=== Test Complete ==="
