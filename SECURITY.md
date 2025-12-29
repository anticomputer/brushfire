# Brushfire Security Model

## Intent

Brushfire provides **first-layer policy enforcement** for shell operations. It intercepts and controls what the brush shell itself does, but **does not control spawned child processes**.

Think of brushfire as a gatekeeper at the shell level - it decides which external programs can run and controls the shell's own file operations, but once a program is allowed to run, that program operates with normal OS permissions.

## What Brushfire Controls

### 1. Process Spawning (First Layer Only)

Brushfire controls **which external commands the shell spawns**:

```bash
# Profile: blacklist /usr/bin/curl

$ brush --profile restricted.profile -c "curl example.com"
error: Policy violation: Command execution denied

$ brush --profile restricted.profile -c "/bin/sh -c 'curl example.com'"
error: Policy violation: Command execution denied (/bin/sh blocked)
```

**Limitation**: If you allow `/bin/sh`, it can spawn anything:
```bash
# Profile: blacklist /usr/bin/curl
# BUT /bin/sh is NOT blacklisted

$ brush --profile restricted.profile -c "/bin/sh -c 'curl example.com'"
# curl WILL execute - /bin/sh is allowed and spawns curl directly
```

### 2. Shell File Operations

Brushfire controls **file operations performed by the shell**:

```bash
# Profile: blacklist /tmp/secret

$ brush --profile restricted.profile -c "echo data > /tmp/secret"
error: Policy violation: File access denied

$ brush --profile restricted.profile -c "cat < /tmp/secret"
error: Policy violation: File access denied
```

**Limitation**: Commands that open files themselves are not controlled:
```bash
# Profile: blacklist /tmp/secret

$ brush --profile restricted.profile -c "cat /tmp/secret"
# /bin/cat is spawned (allowed), then cat opens /tmp/secret directly via OS
# File is read successfully - brushfire only controlled the spawn, not cat's file ops
```

### 3. Shell Builtin Commands

Brushfire controls **builtin commands executed by the shell**:

```bash
# Profile: blacklist /restricted/dir

$ brush --profile restricted.profile -c "cd /restricted/dir && pwd"
error: Policy violation: File access denied
```

## What Brushfire Does NOT Control

### Child Process Operations

Once the shell spawns a child process, that process runs with **normal OS permissions**. Brushfire has no visibility into or control over what that process does.

```
┌─────────────────────────────────────────┐
│ brush (brushfire enforced)              │
│                                         │
│  ✓ Can spawn /bin/cat? → Check policy  │
│  ✓ Redirect > file? → Check policy     │
│  ✓ cd directory? → Check policy         │
│                                         │
│  Spawns: /bin/cat /etc/passwd           │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│ /bin/cat (NOT brushfire enforced)       │
│                                         │
│  Opens /etc/passwd directly via OS      │
│  Reads file with normal permissions     │
│  Brushfire cannot see or control this   │
└─────────────────────────────────────────┘
```

### Nested Shell Invocations

If you allow shell execution, nested shells can bypass policies:

```bash
# Profile: blacklist /usr/bin/curl

# This is blocked:
$ brush --profile restricted.profile -c "curl example.com"
error: Policy violation

# But if /bin/bash is allowed:
$ brush --profile restricted.profile -c "/bin/bash -c 'curl example.com'"
# curl runs - /bin/bash was allowed, and bash spawns curl directly
```

**Solution**: Block all shells in your profile:
```
blacklist /bin/bash
blacklist /bin/sh
blacklist /bin/zsh
blacklist /usr/bin/bash
```

### System Calls and Library Functions

Programs can make direct system calls or use library functions that brushfire cannot intercept:

- `open()`, `read()`, `write()` syscalls
- `fopen()`, `fread()`, `fwrite()` libc functions
- Network operations (`socket()`, `connect()`)
- Any FFI or native code operations

## Security Boundaries

### What Brushfire IS

- ✓ **Defense-in-depth** - Extra security layer
- ✓ **Policy enforcement** - Controls shell behavior
- ✓ **Accident prevention** - Stops mistakes
- ✓ **Script sandboxing** - Limits script actions at shell level

### What Brushfire IS NOT

- ✗ **Primary security boundary** - Not a replacement for OS security
- ✗ **Process sandbox** - Cannot isolate spawned processes
- ✗ **Kernel enforcement** - No mandatory access control
- ✗ **Exploit mitigation** - Malicious code can bypass

## Threat Model

### Protects Against

1. **Accidental operations**
   ```bash
   # Prevents: rm -rf / by mistake
   blacklist /bin/rm
   ```

2. **Script mistakes**
   ```bash
   # Script tries to write to wrong location
   echo "data" > /etc/important-config  # Blocked by read-only
   ```

3. **First-layer restrictions**
   ```bash
   # Prevents obvious network access
   blacklist /usr/bin/curl
   blacklist /usr/bin/wget
   ```

4. **Shell-level constraints**
   ```bash
   # Scripts cannot cd to sensitive dirs
   cd /root/.ssh  # Blocked
   ```

### Does NOT Protect Against

1. **Malicious programs**
   - Once a program is allowed to spawn, it can do anything its OS permissions allow

2. **Privilege escalation**
   - Brushfire runs as the user, cannot provide root-level enforcement

3. **Direct syscalls**
   - Programs can bypass userspace by making syscalls directly

4. **Multi-layer indirection**
   ```bash
   # If /bin/sh is allowed:
   /bin/sh -c '/bin/sh -c "/bin/sh -c curl example.com"'
   # Eventually reaches curl
   ```

## Recommended Usage

### Good Use Cases

1. **Development environments**
   - Prevent accidental damage during development
   - Enforce workspace boundaries

2. **CI/CD pipelines**
   - Control what build scripts can spawn
   - Prevent network access during builds (if no shells allowed)

3. **Script testing**
   - Test scripts in restricted environment
   - Verify script behavior without full system access

4. **Educational purposes**
   - Learn about security policies
   - Understand sandboxing concepts

5. **Defense-in-depth**
   - Additional layer alongside OS security (AppArmor, SELinux, etc.)
   - Catch common mistakes before they cause damage

### Poor Use Cases

1. **Primary malware defense**
   - Malicious code will bypass userspace enforcement

2. **Untrusted binary execution**
   - Binaries can make direct syscalls, use FFI, etc.

3. **Production security isolation**
   - Use OS-level tools: Docker, firejail, systemd, AppArmor, SELinux

4. **Privilege boundaries**
   - Cannot enforce across user/root boundary

## Best Practices

### 1. Layer Your Defenses

Use brushfire WITH other security measures:
```bash
# System level: firejail with namespaces
# Shell level: brushfire policy
# Application level: least privilege
```

### 2. Block Shell Spawning

If you want strong first-layer control, block nested shells:
```
blacklist /bin/bash
blacklist /bin/sh
blacklist /bin/zsh
blacklist /usr/bin/bash
blacklist /usr/local/bin/bash
```

### 3. Understand Your Trust Model

Ask: "Do I trust the programs I'm allowing to spawn?"
- If YES: Brushfire provides accident prevention
- If NO: You need OS-level isolation (containers, namespaces)

### 4. Document Your Policies

```
# Profile purpose: CI build environment
# Trust model: Build scripts are trusted, prevent network access
#
# Block network tools to prevent data exfiltration
blacklist /usr/bin/curl
blacklist /usr/bin/wget
blacklist /usr/bin/nc
#
# Block shells to prevent policy bypass
blacklist /bin/bash
blacklist /bin/sh
```

## Comparison to Other Tools

| Feature | firejail | Docker | AppArmor | Brushfire |
|---------|----------|--------|----------|-----------|
| Kernel enforcement | ✓ | ✓ | ✓ | ✗ |
| Controls child procs | ✓ | ✓ | ✓ | ✗ |
| Cross-platform | ✗ | ✓ | ✗ | ✓ |
| No root required | ✗ | ✗ | ✗ | ✓ |
| Shell-level control | ✗ | ✗ | ✗ | ✓ |
| First-layer only | ✗ | ✗ | ✗ | ✓ |

## Example: What Actually Gets Enforced

```bash
# Profile
blacklist /usr/bin/curl
blacklist /tmp/secret
noexec /tmp

# Test cases
$ brush --profile test.profile -c "curl example.com"
→ ✗ BLOCKED (process spawn check)

$ brush --profile test.profile -c "echo data > /tmp/secret"
→ ✗ BLOCKED (shell file operation)

$ brush --profile test.profile -c "/tmp/script.sh"
→ ✗ BLOCKED (noexec directory)

$ brush --profile test.profile -c "cat /tmp/secret"
→ ✓ ALLOWED (/bin/cat spawns, then cat opens /tmp/secret via OS)

$ brush --profile test.profile -c "python3 -c 'import urllib; urllib.request.urlopen(...)'"
→ ✓ ALLOWED (python3 spawns, then python makes network calls directly)

$ brush --profile test.profile -c "/bin/sh -c 'curl example.com'"
→ ✓ ALLOWED if /bin/sh not blacklisted (sh spawns, then sh spawns curl)
```

## Summary

Brushfire is a **first-layer policy enforcement tool** that controls what the shell does. It provides meaningful defense-in-depth for trusted environments but is not a security sandbox for untrusted code.

**Key principle**: Brushfire controls the shell's actions. Once a child process spawns, it operates independently with normal OS permissions.

Use brushfire to:
- Prevent shell-level mistakes
- Control script execution at entry point
- Add an extra security layer
- Learn sandboxing concepts portably

Do not use brushfire as:
- Primary malware defense
- Process isolation tool
- Replacement for OS security
- Trusted computing base
