#!/bin/bash
# Comprehensive brushfire tests
# Tests what brushfire CAN enforce (userspace limitations acknowledged)

BRUSH="../target/release/brush"
PASSED=0
FAILED=0

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Brushfire Comprehensive Test Suite ===${NC}"
echo "Testing userspace policy enforcement capabilities"
echo

test_should_fail() {
    local name="$1"
    local profile="$2"
    local command="$3"

    echo -n "TEST: $name ... "

    if $BRUSH --profile "$profile" -c "$command" 2>&1 | grep -q "Policy violation"; then
        echo -e "${GREEN}PASS${NC}"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}FAIL${NC}"
        ((FAILED++))
        return 1
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
        return 1
    else
        echo -e "${GREEN}PASS${NC}"
        ((PASSED++))
        return 0
    fi
}

echo -e "${YELLOW}[1] Command Blocking (Process Spawn Interception)${NC}"
echo "    Brushfire intercepts ALL external command execution"
test_should_fail "Block curl" "block-commands.profile" "curl --version"
test_should_fail "Block sudo" "block-commands.profile" "sudo echo test"
test_should_succeed "Allow echo" "block-commands.profile" "echo hello"
test_should_succeed "Allow ls" "block-commands.profile" "ls /tmp"
echo

echo -e "${YELLOW}[2] Shell Redirection Enforcement${NC}"
echo "    Brushfire controls file operations done BY THE SHELL"
test_should_fail "Block output redirect" "redirection-test.profile" "echo test > /tmp/blocked-file"
test_should_fail "Block append redirect" "redirection-test.profile" "echo test >> /tmp/blocked-file"
test_should_succeed "Allow write to /tmp/allowed" "redirection-test.profile" "echo test > /tmp/allowed"
echo

echo -e "${YELLOW}[3] Builtin Command Enforcement${NC}"
echo "    Brushfire controls builtin commands like cd"
# Create test directories
mkdir -p /tmp/allowed-dir /tmp/blocked-dir
cat > /tmp/cd-test.profile << 'EOF'
blacklist /tmp/blocked-dir
EOF
test_should_fail "Block cd to blacklisted dir" "/tmp/cd-test.profile" "cd /tmp/blocked-dir && pwd"
test_should_succeed "Allow cd to allowed dir" "/tmp/cd-test.profile" "cd /tmp/allowed-dir && pwd"
rm /tmp/cd-test.profile
echo

echo -e "${YELLOW}[4] Include Files and Macros${NC}"
echo "    Brushfire supports include directives and \${HOME} expansion"
test_should_fail "Include blocks curl" "with-includes.profile" "curl --version"
test_should_succeed "Include allows echo" "with-includes.profile" "echo test"
echo

echo -e "${YELLOW}[5] NoExec Enforcement${NC}"
echo "    Brushfire blocks execution from noexec directories"
# Create a test script in /tmp
echo '#!/bin/sh' > /tmp/test-script.sh
echo 'echo "executed"' >> /tmp/test-script.sh
chmod +x /tmp/test-script.sh
test_should_fail "Block exec from /tmp" "with-includes.profile" "/tmp/test-script.sh"
rm /tmp/test-script.sh
echo

echo -e "${BLUE}=== Known Limitations (By Design) ===${NC}"
echo -e "${YELLOW}Brushfire CANNOT enforce:${NC}"
echo "  × File operations by spawned processes (e.g., 'cat /etc/hosts')"
echo "    → Once /bin/cat is spawned, it opens files directly via OS"
echo "  × Network operations by spawned processes"
echo "    → Spawned process connects to network via OS"
echo "  × File operations using FFI or direct syscalls"
echo "    → Bypasses our interception points"
echo
echo -e "${GREEN}Brushfire CAN enforce:${NC}"
echo "  ✓ Which external commands can be executed"
echo "  ✓ Shell redirections (>, <, >>, 2>, etc.)"
echo "  ✓ Builtin commands (cd, exec, source)"
echo "  ✓ Execution from specific directories (noexec)"
echo

echo -e "${BLUE}=== Test Summary ===${NC}"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo "Brushfire is working correctly within its design constraints"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
