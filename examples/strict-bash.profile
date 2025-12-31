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
# Blacklist is unnecessary - anything not whitelisted is denied by default

# Safe file viewing commands
whitelist /bin/ls
whitelist /bin/cat
whitelist /usr/bin/head
whitelist /usr/bin/tail
whitelist /usr/bin/less
whitelist /usr/bin/more
whitelist /usr/bin/file

# Safe text processing
whitelist /usr/bin/grep
whitelist /usr/bin/cut
whitelist /usr/bin/tr
whitelist /usr/bin/wc
whitelist /usr/bin/sort
whitelist /usr/bin/uniq

# Safe file operations (read-only or non-destructive)
whitelist /bin/mkdir
whitelist /bin/cp
whitelist /usr/bin/touch
whitelist /usr/bin/find
whitelist /usr/bin/diff

# Basic utilities
whitelist /bin/echo
whitelist /bin/pwd
whitelist /usr/bin/which
whitelist /usr/bin/basename
whitelist /usr/bin/dirname
whitelist /usr/bin/date
whitelist /usr/bin/printf

# Note: Commands like sh, bash, env, perl, curl, rm, ssh, sudo are NOT
# whitelisted and will be denied by default. No explicit blacklist needed.

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
