#!/bin/bash
# Brushfire Demo - Real World Usage

BRUSH="../target/release/brush"
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

clear
echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         BRUSHFIRE - Portable Policy Enforcement        ║${NC}"
echo -e "${BLUE}║    Firejail-style ACLs without OS dependencies         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo

echo -e "${YELLOW}[Demo 1: Sandboxed Script Execution]${NC}"
echo "Running untrusted script with sandbox.profile..."
echo

# Create a suspicious script
cat > /tmp/suspicious.sh << 'EOF'
#!/bin/bash
echo "Trying dangerous operations..."
curl https://malicious.example.com 2>&1 | head -1
rm -rf /tmp/important 2>&1 | head -1
cat ~/.ssh/id_rsa 2>&1 | head -1
echo "Done!"
EOF
chmod +x /tmp/suspicious.sh

echo -e "${BLUE}Script content:${NC}"
cat /tmp/suspicious.sh
echo

echo -e "${YELLOW}Without protection:${NC}"
echo "(Showing what WOULD happen without brushfire)"
echo "  ✗ curl would connect to internet"
echo "  ✗ rm would delete files"
echo "  ✗ SSH keys would be exposed"
echo

echo -e "${YELLOW}With brushfire protection:${NC}"
$BRUSH --profile sandbox.profile -c "/tmp/suspicious.sh" 2>&1 | head -20
echo

rm /tmp/suspicious.sh

echo -e "${YELLOW}[Demo 2: Safe File Operations]${NC}"
echo "Controlling shell redirections..."
echo

echo -e "${BLUE}Blocked: Writing to sensitive location${NC}"
$BRUSH --profile sandbox.profile -c "echo 'malicious' > ${HOME}/.ssh/test" 2>&1 | grep -A1 "error"
echo

echo -e "${GREEN}Allowed: Writing to /tmp${NC}"
$BRUSH --profile sandbox.profile -c "echo 'safe data' > /tmp/safe-file && cat /tmp/safe-file"
rm /tmp/safe-file 2>/dev/null
echo

echo -e "${YELLOW}[Demo 3: Command Whitelisting]${NC}"
echo "Only allowing specific tools..."
echo

echo -e "${GREEN}Allowed: Standard utilities${NC}"
$BRUSH --profile sandbox.profile -c "echo 'Hello from brushfire!' && ls /tmp | head -3"
echo

echo -e "${RED}Blocked: Network tools${NC}"
$BRUSH --profile sandbox.profile -c "curl --version" 2>&1 | grep "Policy violation"
echo

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    Demo Complete                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo
echo -e "${GREEN}Brushfire provides:${NC}"
echo "  ✓ Cross-platform policy enforcement"
echo "  ✓ Firejail-compatible configuration"
echo "  ✓ Defense-in-depth security layer"
echo "  ✓ No root or kernel dependencies"
echo
echo -e "${YELLOW}Use cases:${NC}"
echo "  • Running untrusted scripts safely"
echo "  • CI/CD pipeline sandboxing"
echo "  • Development environment isolation"
echo "  • Educational security demonstrations"
