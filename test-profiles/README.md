# Brushfire Test Suite

Comprehensive tests for brushfire policy enforcement.

## Test Files

- `comprehensive-tests.sh` - Full test suite (12 tests)
- `demo.sh` - Interactive demonstration
- `*.profile` - Various test profiles

## Running Tests

```bash
# Build with policy feature
cd ..
cargo build --release --features policy

# Run comprehensive tests
cd test-profiles
./comprehensive-tests.sh

# Run interactive demo
./demo.sh
```

## Test Results

```
✓ Command Blocking (4/4 tests)
  - Block curl, sudo: PASS
  - Allow echo, ls: PASS

✓ Shell Redirection Enforcement (3/3 tests)
  - Block redirects to blacklisted files: PASS
  - Allow writes to permitted locations: PASS

✓ Builtin Command Enforcement (2/2 tests)
  - Block cd to blacklisted dirs: PASS
  - Allow cd to permitted dirs: PASS

✓ Include Files and Macros (2/2 tests)
  - Include files work: PASS
  - Macro expansion works: PASS

✓ NoExec Enforcement (1/1 test)
  - Block execution from /tmp: PASS

Total: 12/12 tests PASS
```

## What Brushfire Enforces

### ✓ Can Control

1. **External Command Execution**
   - Which commands can run
   - Where commands can execute from (noexec)

2. **Shell File Operations**
   - Output redirections (>, >>)
   - Input redirections (<)
   - All shell-controlled file opens

3. **Builtin Commands**
   - cd directory access
   - exec command execution
   - source script loading

### ✗ Cannot Control (By Design)

1. **Spawned Process Operations**
   - File ops by `/bin/cat`, `/usr/bin/vim`, etc.
   - Network ops by spawned processes
   - Direct syscalls in spawned processes

This is a fundamental limitation of userspace enforcement - once we spawn
a process, that process interacts directly with the OS.

## Example Profiles

### Basic Blacklist
```bash
blacklist /etc/shadow
blacklist /usr/bin/curl
noexec /tmp
```

### With Includes
```bash
include common-blocks.inc
blacklist /usr/bin/wget
read-only ${HOME}/.config
```

### Sandbox Profile
```bash
# Block dangerous commands
blacklist /usr/bin/curl
blacklist /usr/bin/wget
blacklist /bin/rm

# Block sensitive dirs
blacklist ${HOME}/.ssh

# Prevent execution from temp
noexec /tmp
noexec ${HOME}/Downloads
```

## Use Cases

- **Development Safety**: Prevent accidental file deletion
- **Script Testing**: Run untrusted scripts with constraints
- **CI/CD**: Sandbox build scripts
- **Education**: Learn sandboxing concepts portably
- **Defense-in-Depth**: Extra layer alongside OS security

## Platform Compatibility

Tested on:
- ✓ macOS (Darwin 25.2.0)
- ✓ Linux (expected to work)
- ✓ Windows (expected to work)

Works anywhere brush runs - no OS-specific features required.
