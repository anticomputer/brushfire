#!/bin/bash
# Comprehensive test script for all 30 brushfire wrappers

set -e

PROFILE="test-profiles/wrapper-simple-test.profile"
BRUSH="cargo run -p brush-shell --features policy -- --profile $PROFILE --wrap-coreutils"

echo "=========================================="
echo "Testing All 30 Brushfire Wrappers"
echo "=========================================="
echo ""

# Setup test files
mkdir -p /tmp/brushfire-test
echo "test content" > /tmp/brushfire-test/allowed.txt
echo "secret data" > /tmp/test-secret.txt

TEST_COUNT=0
PASS_COUNT=0
FAIL_COUNT=0

test_wrapper() {
    local name=$1
    local cmd=$2
    local expect_fail=${3:-false}

    TEST_COUNT=$((TEST_COUNT + 1))
    echo -n "Test $TEST_COUNT: $name ... "

    if $BRUSH -c "$cmd" > /tmp/brushfire-test/output.txt 2>&1; then
        if [ "$expect_fail" = "true" ]; then
            echo "FAIL (expected failure but succeeded)"
            FAIL_COUNT=$((FAIL_COUNT + 1))
            cat /tmp/brushfire-test/output.txt
        else
            echo "PASS"
            PASS_COUNT=$((PASS_COUNT + 1))
        fi
    else
        if [ "$expect_fail" = "true" ]; then
            if grep -q "Policy violation\|brushfire:" /tmp/brushfire-test/output.txt; then
                echo "PASS (correctly blocked)"
                PASS_COUNT=$((PASS_COUNT + 1))
            else
                echo "FAIL (wrong error)"
                FAIL_COUNT=$((FAIL_COUNT + 1))
                cat /tmp/brushfire-test/output.txt
            fi
        else
            echo "FAIL (unexpected failure)"
            FAIL_COUNT=$((FAIL_COUNT + 1))
            cat /tmp/brushfire-test/output.txt
        fi
    fi
}

echo "=== File Reading Operations ==="
test_wrapper "cat allowed file" "cat /tmp/brushfire-test/allowed.txt"
test_wrapper "cat blocked file" "cat /tmp/test-secret.txt" true
test_wrapper "head allowed file" "head /tmp/brushfire-test/allowed.txt"
test_wrapper "tail allowed file" "tail /tmp/brushfire-test/allowed.txt"
test_wrapper "grep allowed file" "grep test /tmp/brushfire-test/allowed.txt"
test_wrapper "wc allowed file" "wc /tmp/brushfire-test/allowed.txt"
test_wrapper "cut allowed file" "cut -c1-5 /tmp/brushfire-test/allowed.txt"
test_wrapper "sort allowed file" "sort /tmp/brushfire-test/allowed.txt"
test_wrapper "uniq allowed file" "uniq /tmp/brushfire-test/allowed.txt"
test_wrapper "diff allowed files" "diff /tmp/brushfire-test/allowed.txt /tmp/brushfire-test/allowed.txt"

echo ""
echo "=== File Metadata Operations ==="
test_wrapper "ls allowed dir" "ls /tmp/brushfire-test"
test_wrapper "file allowed file" "file /tmp/brushfire-test/allowed.txt"
test_wrapper "stat allowed file" "stat /tmp/brushfire-test/allowed.txt"

echo ""
echo "=== File Writing Operations ==="
test_wrapper "touch new file" "touch /tmp/brushfire-test/newfile.txt"
test_wrapper "rm allowed file" "rm /tmp/brushfire-test/newfile.txt"
test_wrapper "mkdir new dir" "mkdir /tmp/brushfire-test/newdir"
test_wrapper "rmdir new dir" "rmdir /tmp/brushfire-test/newdir"
test_wrapper "cp allowed file" "cp /tmp/brushfire-test/allowed.txt /tmp/brushfire-test/copy.txt"
test_wrapper "mv allowed file" "mv /tmp/brushfire-test/copy.txt /tmp/brushfire-test/moved.txt"
test_wrapper "rm moved file" "rm /tmp/brushfire-test/moved.txt"

echo ""
echo "=== Permission Operations ==="
test_wrapper "chmod allowed file" "chmod 644 /tmp/brushfire-test/allowed.txt"

echo ""
echo "=== Text Processing ==="
echo "line1" > /tmp/brushfire-test/test1.txt
echo "line2" > /tmp/brushfire-test/test2.txt
test_wrapper "paste files" "paste /tmp/brushfire-test/test1.txt /tmp/brushfire-test/test2.txt"
test_wrapper "sed read file" "sed -n '1p' /tmp/brushfire-test/allowed.txt"
test_wrapper "awk read file" "awk '{print}' /tmp/brushfire-test/allowed.txt"

echo ""
echo "=== Symlink Operations ==="
test_wrapper "ln create symlink" "ln -s /tmp/brushfire-test/allowed.txt /tmp/brushfire-test/link.txt"
test_wrapper "rm symlink" "rm /tmp/brushfire-test/link.txt"

echo ""
echo "=== Search Operations ==="
test_wrapper "find allowed dir" "find /tmp/brushfire-test -name allowed.txt"

echo ""
echo "=== Stdin/Stdout Operations ==="
test_wrapper "tr translate" "echo test | tr a-z A-Z"
test_wrapper "tee write file" "echo test | tee /tmp/brushfire-test/tee-output.txt"

echo ""
echo "=== Policy Enforcement Tests ==="
test_wrapper "cat blocked via absolute path" "/bin/cat /tmp/test-secret.txt" true
test_wrapper "grep blocked file" "grep secret /tmp/test-secret.txt" true
test_wrapper "wc blocked file" "wc /tmp/test-secret.txt" true
test_wrapper "head blocked file" "head /tmp/test-secret.txt" true
test_wrapper "tail blocked file" "tail /tmp/test-secret.txt" true

echo ""
echo "=========================================="
echo "Test Results"
echo "=========================================="
echo "Total Tests: $TEST_COUNT"
echo "Passed: $PASS_COUNT"
echo "Failed: $FAIL_COUNT"
echo ""

# Cleanup
rm -rf /tmp/brushfire-test
rm -f /tmp/test-secret.txt

if [ $FAIL_COUNT -eq 0 ]; then
    echo "✅ All tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
