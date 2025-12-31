# Brushfire

Policy enforcement layer for the [brush shell](https://github.com/reubeno/brush).

## What is Brushfire?

Brushfire is a **guardrails shell for AI agents and development environments** - providing first-layer policy enforcement to prevent accidental damage and provide observability, not a hard security boundary.

For security isolation of untrusted code, use OS-level enforcement. See [SECURITY.md](SECURITY.md) for the complete security model.

Brushfire adds shell-level policies controlling:
- **Filesystem access** - whitelist/blacklist paths, read-only directories, noexec regions
- **Command execution** - whitelist/blacklist which commands can spawn
- **Command argument checking** - heuristic detection and policy checking of file path arguments

## Documentation

- [SECURITY.md](SECURITY.md) - Security model, threat model, and limitations
- [USAGE.md](USAGE.md) - Profile syntax and examples

## Building

```bash
# Development build (recommended)
./build.sh dev

# Release build
./build.sh release

# With webhook support
./build.sh dev --webhook
```

Run `./build.sh help` for more build options.

## Quick Start

```bash
# Run with a policy profile
brush --profile my.profile -c 'commands here'

# All command arguments that resolve to paths are automatically checked
brush --profile my.profile -c 'cat /etc/passwd'
```

### Example Profile

```bash
# Only allow execution from /tmp/safe-bin
whitelist /tmp/safe-bin/*

# Only allow file access to workspace (but not execution)
whitelist /home/user/workspace
noexec /home/user/workspace

# Default deny all other access
```

## Policy Grammar

Brushfire uses a unified ACL model where directives apply to both filesystem and command operations:

### Core Directives

**`whitelist <path>`** - Allow filesystem access (read/write/execute) AND command execution
- Triggers default-deny mode for both filesystem and commands
- Filesystem rules are recursive (includes subdirectories)
- Command matching uses glob patterns (`/bin/*` to match all commands)

**`blacklist <path>`** - Deny filesystem access AND command execution
- Takes precedence over whitelist
- Blocks both file operations and process spawning

**`noexec <path>`** - Allow filesystem access but block execution
- Enables read/write operations while preventing command spawning
- Useful for data directories that should not contain executables

**`read-only <path>`** - Allow read/execute but block writes
- Protects paths from modification

### Examples

```bash
# Default-deny: only allow specific paths and commands
whitelist /workspace
whitelist /bin/ls
whitelist /bin/cat

# Allow file access but prevent execution
whitelist /data
noexec /data

# Block specific dangerous commands
blacklist /usr/bin/curl
blacklist /usr/bin/wget
```

See [USAGE.md](USAGE.md) for complete syntax and examples.

## Observability

Brushfire includes optional webhook-based observability for monitoring all policy checks:

### Webhook Events

When built with `--webhook`, Brushfire can send policy events to an HTTP endpoint:

```bash
# Build with webhook support
./build.sh dev --webhook

# Run with webhook reporting
./target/debug/brush --profile my.profile \
  --policy-webhook http://localhost:8080 \
  -c 'commands here'
```

**Event structure:**
- File access checks from command arguments
- Command spawn attempts
- Policy decisions (allowed/denied)
- Violation reasons

**Use cases:**
- Audit trails for compliance
- Real-time monitoring of agent behavior
- Security analytics and alerting
- Development debugging

See the webhook reporter implementation in `brushfire-policy/src/reporter.rs` for event schema details.

## About Brush

For details about the brush shell itself, see the [upstream README](https://github.com/reubeno/brush/blob/main/README.md).

Brush is a POSIX/bash-compatible shell implemented in Rust by [@reubeno](https://github.com/reubeno).

## License

MIT (same as upstream brush)
