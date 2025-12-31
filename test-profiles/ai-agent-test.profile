# AI Agent Sandbox Test Profile
# Tests the prompt directive for informing AI agents about policy

# Display instructions at interactive shell startup
prompt === AI Agent Sandbox ===
prompt
prompt CRITICAL: Use 'brush script.sh' instead of 'bash script.sh'
prompt Bash and other shells are blocked to maintain ACL coverage.
prompt
prompt Available tools: ls, cat, grep
prompt Workspace: /tmp (read/write)
prompt
prompt For help: see USAGE.md

# Default-deny mode
whitelist /usr/bin/ls
whitelist /bin/ls
whitelist /usr/bin/cat
whitelist /bin/cat
whitelist /usr/bin/grep
whitelist /bin/grep

# Block all shells except brush
blacklist /bin/bash
blacklist /bin/sh
blacklist /usr/bin/bash
blacklist /usr/bin/sh
blacklist /bin/zsh
blacklist /usr/bin/python*

# Allow /tmp for workspace
whitelist /tmp
