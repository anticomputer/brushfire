# Strict Bash Profile
# Example policy for sandboxed bash execution
#
# This demonstrates brushfire's enforcement model:
# - First-layer policy for shell operations
# - Heuristic path checking for command arguments
# - Commands can spawn freely once allowed (see SECURITY.md)

# ============================================================================
# COMMAND EXECUTION - Default Deny Mode
# ============================================================================
# Using 'whitelist' triggers default-deny for BOTH filesystem AND commands
# Only explicitly whitelisted commands can be spawned

# Allow common safe utilities (glob pattern to match all in directory)
whitelist /usr/bin/*
whitelist /bin/*

# Block dangerous commands explicitly (takes precedence over whitelist)
blacklist /usr/bin/curl
blacklist /usr/bin/wget
blacklist /usr/bin/nc
blacklist /bin/nc
blacklist /usr/bin/ssh
blacklist /usr/bin/scp
blacklist /bin/rm
blacklist /usr/bin/sudo
blacklist /usr/bin/su

# ============================================================================
# FILESYSTEM ACCESS
# ============================================================================

# Allow shell initialization files for interactive shell
whitelist ${HOME}/.bashrc
whitelist ${HOME}/.bash_profile
whitelist ${HOME}/.bash_login
whitelist ${HOME}/.profile

# Allow system-wide shell initialization
whitelist /etc/profile
whitelist /etc/bash.bashrc
whitelist /etc/bashrc

# Allow workspace for file operations (read/write)
whitelist /tmp/workspace
# But prevent execution from workspace (data only, not binaries)
noexec /tmp/workspace

# Allow temp directory
whitelist /tmp
noexec /tmp

# Block sensitive directories
blacklist ${HOME}/.ssh
blacklist ${HOME}/.gnupg
blacklist ${HOME}/.aws
blacklist /root

# ============================================================================
# NOTES
# ============================================================================
#
# Policy enforcement:
#   - Shell file operations are controlled (redirections, builtins)
#   - Command spawning is controlled (can't run curl, rm, etc.)
#   - Command arguments are checked for file paths automatically
#   - "cat /etc/shadow" would be blocked (heuristic detects /etc/shadow)
#
# This is first-layer enforcement - once a command spawns, it can make
# direct syscalls. Paths embedded in strings won't be detected.
# See SECURITY.md for the complete security model.
