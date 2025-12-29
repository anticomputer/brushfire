#!/bin/bash
# Brushfire test suite
# Tests various policy enforcement scenarios

BRUSH="../target/release/brush"
PASSED=0
FAILED=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== Brushfire Policy Enforcement Tests ==="
echo

# Test helper function
test_should_fail() {
    local name="$1"
    local profile="$2"
    local command="$3"

    echo -n "TEST: $name ... "

    if $BRUSH --profile "$profile" -c "$command" 2>&1 | grep -q "Policy violation"; then
        echo -e "${GREEN}PASS${NC}"
        ((PASSED++))
    else
        echo -e "${RED}FAIL${NC} (expected policy violation)"
        ((FAILED++))
    fi
}

test_should_succeed() {
    local name="$1"
    local profile="$2"
    local command="$3"

    echo -n "TEST: $name ... "

    if $BRUSH --profile "$profile" -c "$command" 2>&1 | grep -q "Policy violation"; then
        echo -e "${RED}FAIL${NC} (unexpected policy violation)"
        ((FAILED++))
    else
        echo -e "${GREEN}PASS${NC}"
        ((PASSED++))
    fi
}

# Test 1: Basic blacklist
echo "--- Basic Blacklist Tests ---"
test_should_fail "Block /etc/shadow" "basic-blacklist.profile" "cat /etc/shadow"
test_should_fail "Block /etc/passwd" "basic-blacklist.profile" "cat /etc/passwd"
test_should_succeed "Allow /etc/hosts" "basic-blacklist.profile" "cat /etc/hosts"
echo

# Test 2: Command blocking
echo "--- Command Blocking Tests ---"
test_should_fail "Block curl command" "block-commands.profile" "curl --version"
test_should_fail "Block wget command" "block-commands.profile" "wget --version"
test_should_succeed "Allow echo command" "block-commands.profile" "echo hello"
echo

# Test 3: Read-only enforcement
echo "--- Read-Only Tests ---"
# Create a test file in home directory for testing
TEST_FILE="$HOME/.brushfire-test-$$"
touch "$TEST_FILE"
test_should_fail "Block write to home" "readonly.profile" "echo test > $TEST_FILE"
test_should_succeed "Allow read from home" "readonly.profile" "cat $TEST_FILE"
rm -f "$TEST_FILE"
echo

# Test 4: Include files and macros
echo "--- Include and Macro Tests ---"
test_should_fail "Include blocks shadow" "with-includes.profile" "cat /etc/shadow"
test_should_fail "Include blocks .ssh" "with-includes.profile" "ls \$HOME/.ssh"
test_should_fail "Noexec blocks /tmp" "with-includes.profile" "/tmp/test.sh"
echo

# Test 5: Builtin cd enforcement
echo "--- Builtin cd Tests ---"
# Note: cd itself doesn't produce output, so we test directory access
test_should_fail "Block cd to blacklisted dir" "basic-blacklist.profile" "cd /etc && cat shadow"
echo

# Summary
echo "=== Test Summary ==="
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed${NC}"
    exit 1
fi
