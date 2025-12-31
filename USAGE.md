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

## Special Directives

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
