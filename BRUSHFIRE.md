# Brushfire

Portable, cross-platform policy enforcement for brush shell. Firejail-style ACL configuration without OS dependencies.

## What is Brushfire?

Brushfire enforces security policies at the **shell level**. It controls:
- Which external commands the shell can spawn
- File operations performed by shell redirections
- Builtin command operations (cd, exec, source)
- Execution from specific directories (noexec)

**Important**: Brushfire controls what the shell does, not what spawned child processes do. See [SECURITY.md](SECURITY.md) for detailed explanation of capabilities and limitations.

## Quick Start

### Build
```bash
cargo build --release --features policy
```

### Run with Policy
```bash
./target/release/brush --profile sandbox.profile

# Or run a command
./target/release/brush --profile sandbox.profile -c "echo hello"
```

### Example Profile
```bash
# sandbox.profile
# Block dangerous commands
blacklist /usr/bin/curl
blacklist /usr/bin/wget
blacklist /bin/rm

# Prevent execution from temp directories
noexec /tmp

# Block access to sensitive locations
blacklist ${HOME}/.ssh
```

## Supported Directives

### Filesystem Controls
- `whitelist PATH` - Allow access (switches to restrictive mode)
- `blacklist PATH` - Deny access
- `noblacklist PATH` - Exception to blacklist
- `read-only PATH` - Allow read-only access
- `noexec PATH` - Prevent execution from directory

### Command Blocking
```bash
blacklist /usr/bin/curl      # Block specific command
blacklist /usr/bin/*          # Block with wildcards
```

### Composition
```bash
include common-rules.inc      # Include other profiles
# Comments start with #
```

### Macros
- `${HOME}` - User home directory
- `${DOWNLOADS}` - Downloads folder
- `${DOCUMENTS}` - Documents folder
- `${PATH}` - PATH environment variable

## What Gets Enforced

### ✓ Enforced (Shell Operations)

```bash
# Command spawning
$ brush --profile policy.profile -c "curl example.com"
→ Blocked if curl is blacklisted

# Shell redirections
$ brush --profile policy.profile -c "echo data > /tmp/secret"
→ Blocked if /tmp/secret is blacklisted

# Builtin commands
$ brush --profile policy.profile -c "cd /restricted && pwd"
→ Blocked if /restricted is blacklisted

# Execution location
$ brush --profile policy.profile -c "/tmp/script.sh"
→ Blocked if /tmp is noexec
```

### ✗ NOT Enforced (Child Process Operations)

```bash
# File ops by spawned commands
$ brush --profile policy.profile -c "cat /tmp/secret"
→ NOT blocked - /bin/cat opens file directly via OS

# Nested shells (if shells allowed)
$ brush --profile policy.profile -c "/bin/sh -c 'curl example.com'"
→ NOT blocked - /bin/sh spawns curl, bypassing policy

# Network ops by programs
$ brush --profile policy.profile -c "python3 -c 'import urllib; ...'"
→ NOT blocked - python makes network calls directly
```

**Solution**: To maintain strong first-layer control, blacklist shells:
```bash
blacklist /bin/bash
blacklist /bin/sh
blacklist /bin/zsh
```

See [SECURITY.md](SECURITY.md) for detailed security model.

## Use Cases

### Development Safety
```bash
# Prevent accidental damage
blacklist /bin/rm
blacklist /usr/bin/sudo
noexec ${HOME}/Downloads
```

### CI/CD Sandboxing
```bash
# Restrict build environment
blacklist /usr/bin/curl
blacklist /usr/bin/wget
blacklist /bin/bash    # Prevent policy bypass
blacklist /bin/sh
noexec /tmp
```

### Script Testing
```bash
# Test scripts in controlled environment
read-only ${HOME}/.config
noexec /tmp
blacklist /usr/bin/ssh
```

## Testing

```bash
# Run test suite
cd test-profiles
./comprehensive-tests.sh

# Run interactive demo
./demo.sh
```

## Architecture

```
brushfire/
├── brushfire-policy/      # Policy parser and engine
│   ├── parser.rs         # Firejail profile parser
│   ├── engine.rs         # Enforcement logic
│   ├── rules.rs          # Policy data structures
│   └── macros.rs         # Variable expansion
├── brush-core/           # Modified for policy checks
├── brush-builtins/       # Instrumented builtins
└── brush-shell/          # CLI with --profile flag
```

## Feature Flag

Brushfire is optional and feature-gated:

```toml
# Enable policy enforcement
cargo build --features policy

# Without policy feature
cargo build  # Standard brush, no policy code included
```

## Platform Support

Works on any platform that supports brush:
- ✓ Linux
- ✓ macOS
- ✓ Windows
- ✓ WASM (if applicable)

No OS-specific features required.

## Limitations

Brushfire is **defense-in-depth**, not a security sandbox:

1. **Child processes are not controlled**
   - Once spawned, programs operate with normal OS permissions
   - Cannot intercept syscalls from child processes

2. **Userspace only**
   - No kernel enforcement
   - No mandatory access control

3. **Policy bypass via shells**
   - If you allow `/bin/sh`, it can spawn anything
   - Solution: blacklist all shells

4. **Not for untrusted code**
   - Malicious programs will bypass userspace enforcement
   - Use OS-level tools for untrusted code (Docker, firejail, AppArmor)

Read [SECURITY.md](SECURITY.md) for complete security model and threat analysis.

## Contributing

Follow existing patterns:
- Policy checks in shell operations, not spawned processes
- Clear error messages citing policy violations
- Tests for new enforcement points
- Documentation of limitations

## License

Same as brush (MIT).

## Credits

- Inspired by firejail (https://firejail.wordpress.com/)
- Built on brush shell (https://github.com/reubeno/brush)
- Profile syntax compatible with firejail subset
