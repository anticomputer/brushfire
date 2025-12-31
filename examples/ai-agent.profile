# AI Coding Agent Sandbox Profile
#
# This profile creates a restricted environment suitable for AI coding assistants.
# The AI can work with files in a designated workspace but cannot:
# - Execute arbitrary code via interpreters (bash, python, node, etc.)
# - Access system files outside the workspace
# - Modify the policy enforcement mechanism
#
# Usage:
#   mkdir -p /tmp/ai-workspace
#   brush --profile examples/ai-agent.profile
#
# The AI can spawn child shells with 'brush' which will inherit these restrictions.

prompt === AI Coding Agent Sandbox ===
prompt
prompt You are an AI coding assistant with restricted filesystem access.
prompt
prompt ALLOWED:
prompt - Read/write files in /tmp/ai-workspace only
prompt - Commands: ls, cat, echo, grep, find, mkdir, touch, rm, mv, cp
prompt - Spawn child shells with 'brush' (inherits restrictions)
prompt
prompt BLOCKED:
prompt - All other commands (default-deny via whitelist)
prompt - System files outside /tmp/ai-workspace
prompt
prompt Task: Help the user with code in /tmp/ai-workspace
prompt ====================================

# Workspace directory (automatically canonicalized)
whitelist /tmp/ai-workspace

# File operations
whitelist /bin/ls
whitelist /usr/bin/ls
whitelist /bin/cat
whitelist /usr/bin/cat
whitelist /bin/echo
whitelist /usr/bin/echo
whitelist /bin/grep
whitelist /usr/bin/grep
whitelist /usr/bin/find
whitelist /bin/mkdir
whitelist /usr/bin/mkdir
whitelist /bin/touch
whitelist /usr/bin/touch
whitelist /bin/rm
whitelist /usr/bin/rm
whitelist /bin/mv
whitelist /usr/bin/mv
whitelist /bin/cp
whitelist /usr/bin/cp

# Check CWD access for commands that operate on current directory by default
check-cwd /bin/ls
check-cwd /usr/bin/ls
check-cwd /usr/bin/find

# Note: All other commands (bash, python, node, etc.) are blocked by default-deny.
# The whitelist directive automatically enables default-deny mode.
