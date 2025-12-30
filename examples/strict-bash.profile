# Strict Bash Profile
# Example policy for sandboxed bash execution
#
# This demonstrates brushfire's enforcement model:
# - First-layer policy for shell operations
# - Use with --wrap-coreutils for better file access control
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
# Without --wrap-coreutils:
#   - Shell file operations are controlled (redirections, builtins)
#   - Command spawning is controlled (can't run curl, rm, etc.)
#   - BUT: "cat /etc/shadow" would succeed (cat opens file, not shell)
#
# With --wrap-coreutils:
#   - All the above PLUS
#   - Coreutils wrappers check file arguments against policy
#   - "cat /etc/shadow" would be blocked by the wrapper
#   - Better enforcement for file operations via common utilities
#
# This is first-layer enforcement - once a command spawns, it can make
# direct syscalls. See SECURITY.md for the complete security model.
