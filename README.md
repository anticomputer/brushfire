# Brushfire

Policy enforcement layer for the [brush shell](https://github.com/reubeno/brush).

## What is Brushfire?

Brushfire is a **guardrails shell for AI agents and development environments** - providing first-layer policy enforcement to prevent accidental damage and provide observability, not a hard security boundary.

For security isolation of untrusted code, use OS-level enforcement. See [SECURITY.md](SECURITY.md) for the complete security model.

Brushfire adds shell-level policies controlling:
- **Filesystem access** - whitelist/blacklist paths, read-only directories, noexec regions
- **Command execution** - whitelist/blacklist which commands can spawn
- **Coreutils wrappers** - optional policy-aware wrappers for common utilities

## Documentation

- [SECURITY.md](SECURITY.md) - Security model, threat model, and limitations
- [USAGE.md](USAGE.md) - Profile syntax and examples

## Quick Start

```bash
# Build with policy support
cargo build --features policy

# Run with a policy profile
brush --profile my.profile -c 'commands here'

# With coreutils wrappers for extended enforcement
brush --profile my.profile --wrap-coreutils -c 'cat /etc/passwd'
```

### Example Profile

```bash
# Only allow execution from /tmp/safe-bin
whitelist_exec /tmp/safe-bin/*

# Only allow file access to workspace
whitelist /home/user/workspace

# Block sensitive files
blacklist /etc/shadow
blacklist /root/.ssh
```

## About Brush

For details about the brush shell itself, see the [upstream README](https://github.com/reubeno/brush/blob/main/README.md).

Brush is a POSIX/bash-compatible shell implemented in Rust by [@reubeno](https://github.com/reubeno).

## License

MIT (same as upstream brush)
