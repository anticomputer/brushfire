# Brushfire Profile Syntax

## Basic Syntax

Profile files use line-based directives. Comments start with `#`.

```bash
# This is a comment
whitelist /home/user/workspace
blacklist /etc/shadow
```

**Note**: Brushfire's profile syntax is based on firejail's but has been extended. Profiles are not guaranteed to be 1:1 compatible with firejail.

## Filesystem and Command Rules

### whitelist

Allow access to a path for both filesystem operations (read/write/execute) AND command execution. First `whitelist` directive triggers default-deny mode for both filesystem and commands.

```bash
# Allow file access AND command execution from workspace
whitelist /home/user/workspace

# Allow specific file access
whitelist /tmp/file.txt

# Allow execution of specific commands
whitelist /usr/bin/ls
whitelist /usr/bin/cat
whitelist /usr/bin/grep

# Allow all binaries in a directory (glob pattern)
whitelist /usr/local/bin/*
```

**Important**: Command matching uses glob patterns. To match all commands in a directory, use `/*`:
- `whitelist /bin/*` - allows executing any command in `/bin` (e.g., `/bin/ls`, `/bin/cat`)
- `whitelist /bin` - only allows executing exactly `/bin` (not subdirectories)
- `whitelist /bin/ls` - only allows executing `/bin/ls`

### blacklist

Deny access to a path for both filesystem operations AND command execution. Takes precedence over whitelist.

```bash
# Block sensitive files and prevent them from being executed
blacklist /etc/shadow
blacklist /root/.ssh

# Block directory access
blacklist /var/log

# Block command execution
blacklist /usr/bin/curl
blacklist /usr/bin/wget
blacklist /bin/bash
```

### read-only

Allow reads but deny writes to a path.

```bash
# System directories as read-only
read-only /usr
read-only /etc

# Specific file as read-only
read-only /etc/hosts
```

### noexec

Prevent execution from a path. This blocks both file execution (script sourcing, shared library loading) AND command spawning from that location. Often used with `whitelist` to allow file access but prevent execution.

```bash
# Allow file access to /tmp but prevent execution
whitelist /tmp
noexec /tmp

# Allow workspace for file operations but prevent execution
whitelist /home/user/workspace
noexec /home/user/workspace

# Prevent execution from downloads
noexec /home/user/Downloads
```

### check-cwd

Specify commands that require CWD (current working directory) access checking when invoked without explicit path arguments.

Many POSIX utilities default to operating on the current directory when run without arguments (e.g., `ls`, `find`, `pwd`). Use `check-cwd` to enforce that the CWD is accessible according to policy before allowing these commands to run.

```bash
# Commands that default to CWD behavior
check-cwd /bin/ls          # ls without args lists CWD
check-cwd /usr/bin/find    # find without args searches CWD
check-cwd /bin/pwd         # pwd operates on CWD

# Version control systems
check-cwd /usr/bin/git     # git commands operate on repo in CWD
check-cwd /usr/bin/hg
check-cwd /usr/bin/svn

# Build tools and package managers
check-cwd /usr/local/bin/cargo    # cargo build reads Cargo.toml from CWD
check-cwd /usr/local/bin/npm      # npm reads package.json from CWD
check-cwd /usr/local/bin/make     # make reads Makefile from CWD

# Linters and formatters
check-cwd /usr/local/bin/eslint   # eslint lints files in CWD
check-cwd /usr/local/bin/clippy   # clippy checks project in CWD
```

**How it works:**
- When a `check-cwd` command is invoked with only flags (no non-flag arguments)
- Brushfire checks if the CWD is accessible according to filesystem policy
- Blocks execution if CWD access would violate policy

**Example:**
```bash
# Profile
check-cwd /bin/ls
whitelist /tmp/allowed-workspace

# From /tmp/allowed-workspace:
$ ls              # ✓ Allowed - CWD is whitelisted

# From /tmp/blocked-workspace:
$ ls              # ✗ Blocked - CWD not whitelisted
$ ls /tmp/allowed-workspace  # ✓ Allowed - explicit path is whitelisted
```

## Special Directives

### prompt

Display informational messages at shell startup for interactive sessions. This directive can be repeated to build up multi-line banners or separate sections of documentation.

```bash
# Inform users/agents about shell constraints
prompt WARNING: This shell is restricted by policy
prompt Only 'brush script.sh' is allowed - bash/sh are blocked
prompt
prompt Available commands: ls, cat, grep, find
prompt Allowed directories: /tmp/workspace, /home/user/data
```

**Behavior:**
- Prompts are displayed only when the shell starts in interactive mode
- Not shown for `-c` command execution or script files (prevents interference with expected output)
- Multiple `prompt` directives are displayed in order, useful for organizing information by topic
- Empty prompt lines can be used to add spacing in the banner

**Use cases:**
- Documenting policy constraints for AI agents
- Providing usage instructions for sandboxed environments
- Listing available tools and allowed paths
- Warning users about restricted capabilities

**Example for AI agents:**
```bash
# ai-agent.profile
prompt === AI Agent Sandbox ===
prompt
prompt CRITICAL: Always use 'brush script.sh', NEVER use 'bash' or 'sh'
prompt Those interpreters are blocked to maintain ACL coverage.
prompt
prompt Tools: ls, cat, grep, head, tail, wc, sort, uniq
prompt Workspace: /tmp/agent-workspace (read/write)
prompt Data: /data (read-only)
prompt
prompt All other shells and interpreters (python, perl, ruby) are blocked.
```

### no-safe-dev-defaults

Disable automatic whitelisting of `/dev/null`, `/dev/stdin`, etc. in default-deny mode.

```bash
# Strict mode without safe defaults
no-safe-dev-defaults

whitelist /allowed/path
```

## Complete Examples

### Workspace Sandbox

```bash
# Allow file access to workspace
whitelist /home/user/workspace
# Prevent execution from workspace (allow file ops only)
noexec /home/user/workspace

# Allow execution of common utilities
whitelist /usr/bin/ls
whitelist /usr/bin/cat
whitelist /usr/bin/grep
whitelist /usr/bin/find

# Default deny all other access and execution
```

### CI Build Environment

```bash
# Allow project directory for file operations but not execution
whitelist /home/runner/project
noexec /home/runner/project

# Allow build tools (default deny all other commands)
whitelist /usr/bin/cargo
whitelist /usr/bin/rustc
whitelist /usr/local/bin/*
```

### Read-Only System Access

```bash
# System directories as read-only
read-only /usr
read-only /etc
read-only /bin

# Workspace as read-write but no execution
whitelist /home/user/workspace
noexec /home/user/workspace

# Prevent execution from tmp
whitelist /tmp
noexec /tmp
```

### Strict Lockdown

```bash
# Disable safe defaults
no-safe-dev-defaults

# Only allow specific workspace for file operations
whitelist /home/user/safe-workspace
noexec /home/user/safe-workspace

# Only allow specific utilities
whitelist /usr/bin/ls
whitelist /usr/bin/cat

# Manually whitelist required device files
whitelist /dev/null
whitelist /dev/stdout
whitelist /dev/stderr
```

## Running with Profiles

```bash
# Build the project (see README.md for build options)
./build.sh dev

# Basic usage
./target/debug/brush --profile my.profile -c 'commands here'

# With webhook observability (requires building with --webhook flag)
./build.sh dev --webhook
./target/debug/brush --profile my.profile \
      --policy-webhook http://localhost:8080 \
      -c 'commands here'
```

## Default Deny Behavior

Adding any `whitelist` directive switches to default-deny mode for both filesystem access AND command execution. Only explicitly whitelisted paths/commands are accessible (plus safe `/dev/` defaults unless disabled).

```bash
# This profile is in default-deny mode for both filesystem and commands
whitelist /home/user/workspace
whitelist /usr/bin/ls
whitelist /usr/bin/cat
# All other paths and commands denied by default
```

### Using noexec with whitelist

The `noexec` directive exempts paths from command execution while maintaining file access:

```bash
# Allow file operations in /tmp but prevent execution
whitelist /tmp
noexec /tmp

# Allow execution of system utilities
whitelist /usr/bin/*
```

This allows you to have different policies for file access versus command execution within the same default-deny profile.
