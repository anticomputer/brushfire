# Brushfire Profile Syntax

## Basic Syntax

Profile files use line-based directives. Comments start with `#`.

```bash
# This is a comment
whitelist /home/user/workspace
blacklist /etc/shadow
```

**Note**: Brushfire's profile syntax is based on firejail's but has been extended. Profiles are not guaranteed to be 1:1 compatible with firejail.

## Filesystem Rules

### whitelist

Allow access to a path. First `whitelist` directive triggers default-deny mode for all filesystem access.

```bash
# Allow access to workspace directory
whitelist /home/user/workspace

# Allow specific file
whitelist /tmp/file.txt

# Allow system config directory
whitelist /etc
```

### blacklist

Deny access to a path. Takes precedence over whitelist.

```bash
# Block sensitive files
blacklist /etc/shadow
blacklist /root/.ssh

# Block directory
blacklist /var/log
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

Prevent execution of files in a path.

```bash
# Prevent execution from tmp
noexec /tmp
noexec /var/tmp

# Prevent execution from user downloads
noexec /home/user/Downloads
```

## Command Rules

### whitelist_exec

Allow execution of specific commands. First `whitelist_exec` directive triggers default-deny mode for all command execution.

```bash
# Only allow specific utilities
whitelist_exec /usr/bin/ls
whitelist_exec /usr/bin/cat
whitelist_exec /usr/bin/grep

# Allow all binaries in a directory (glob pattern)
whitelist_exec /usr/local/bin/*
```

### blacklist (commands)

Deny execution of specific commands.

```bash
# Block network utilities
blacklist /usr/bin/curl
blacklist /usr/bin/wget
blacklist /usr/bin/nc

# Block shells
blacklist /bin/bash
blacklist /bin/sh
blacklist /bin/zsh
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
# Only allow access to workspace
whitelist /home/user/workspace

# Only allow execution of common utilities
whitelist_exec /usr/bin/ls
whitelist_exec /usr/bin/cat
whitelist_exec /usr/bin/grep
whitelist_exec /usr/bin/find

# Default deny all other access and execution
```

### CI Build Environment

```bash
# Allow project directory
whitelist /home/runner/project

# Allow build tools
whitelist_exec /usr/bin/cargo
whitelist_exec /usr/bin/rustc
whitelist_exec /usr/local/bin/*

# Block network access
blacklist /usr/bin/curl
blacklist /usr/bin/wget

# Block shells to prevent policy bypass
blacklist /bin/bash
blacklist /bin/sh
```

### Read-Only System Access

```bash
# System directories as read-only
read-only /usr
read-only /etc
read-only /bin

# Workspace as read-write
whitelist /home/user/workspace

# Prevent execution from tmp
noexec /tmp
```

### Strict Lockdown

```bash
# Disable safe defaults
no-safe-dev-defaults

# Only allow specific workspace
whitelist /home/user/safe-workspace

# Only allow specific utilities
whitelist_exec /usr/bin/ls
whitelist_exec /usr/bin/cat

# Manually whitelist required device files
whitelist /dev/null
whitelist /dev/stdout
whitelist /dev/stderr
```

## Running with Profiles

```bash
# Build with policy support
cargo build --features policy

# Build with webhook support (automatically enables policy)
cargo build --features policy-webhook

# Basic usage
brush --profile my.profile -c 'commands here'

# With coreutils wrappers
brush --profile my.profile --wrap-coreutils -c 'cat file.txt'

# With webhook observability (requires policy-webhook feature)
brush --profile my.profile --policy-webhook http://localhost:8080 -c 'commands'

# Combined
brush --profile my.profile \
      --wrap-coreutils \
      --policy-webhook http://localhost:8080 \
      -c 'commands here'
```

## Default Deny Behavior

### Filesystem

Adding any `whitelist` directive switches to default-deny mode for filesystem access. Only explicitly whitelisted paths are accessible (plus safe `/dev/` defaults unless disabled).

```bash
# This profile is in default-deny mode
whitelist /home/user/workspace
# All other paths denied by default
```

### Commands

Adding any `whitelist_exec` directive switches to default-deny mode for command execution. Only explicitly whitelisted commands can be spawned.

```bash
# This profile is in default-deny mode for commands
whitelist_exec /usr/bin/ls
# All other commands denied by default
```

### Mixed Mode

You can use default-deny for one and permissive for the other:

```bash
# Default-deny for filesystem, permissive for commands
whitelist /home/user/workspace

# Default-deny for commands, permissive for filesystem
whitelist_exec /usr/bin/*
```
