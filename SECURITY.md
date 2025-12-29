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

## Coreutils Wrappers (`--wrap-coreutils`)

### Overview

The `--wrap-coreutils` feature extends policy enforcement to common utilities by wrapping them with policy-checking proxies. This addresses the child process limitation for frequently-used commands.

```bash
brush --profile policy.profile --wrap-coreutils -c "cat /etc/passwd"
# Now cat's file access is checked against policy
```

### How It Works

1. **Wrapper Generation**: Creates policy-aware wrappers for ~30 POSIX utilities
2. **PATH Manipulation**: Prepends wrapper directory to PATH
3. **Auto-Blacklisting**: Real utilities are automatically blacklisted to prevent bypass
4. **Policy Checking**: Wrappers check arguments against policy before execing real utility

### Use Case: AI Agent Accountability

Coreutils wrappers are **primarily designed for AI agent observability and accountability**, not adversarial security:

- **Goal**: Provide guardrails and audit trail for AI agents performing file operations
- **Threat Model**: Prevent unintended access, not adversarial exploitation
- **Design**: ACL accounting and webhook observability, not hard security boundary

Think of it as "bumpers in a bowling lane" - keeping AI agents on track, not preventing malicious actors from escaping.

### Wrapped Utilities

File operations: `cat`, `cp`, `mv`, `rm`, `ls`, `head`, `tail`, `touch`, `mkdir`, `ln`, `chmod`, `chown`, `chgrp`, `rmdir`, `dd`, `file`, `stat`

Text processing: `grep`, `sed`, `awk`, `cut`, `paste`, `sort`, `uniq`, `tr`, `wc`, `tee`, `diff`

Archive/search: `tar`, `find`

### What Gets Enforced

```bash
# Profile: blacklist /etc/shadow

# WITHOUT --wrap-coreutils:
$ brush --profile policy.profile -c "cat /etc/shadow"
✓ ALLOWED - /bin/cat spawns, opens file directly

# WITH --wrap-coreutils:
$ brush --profile policy.profile --wrap-coreutils -c "cat /etc/shadow"
✗ BLOCKED - wrapper checks /etc/shadow against policy before calling real cat
```

### Known Limitations

#### 1. Time-of-Check-Time-of-Use (TOCTOU)

**Issue**: Race condition between policy check and file access

```
Wrapper checks /tmp/file.txt → ALLOWED
                [race window]
Attacker: mv /tmp/file.txt /tmp/old && ln -s /etc/passwd /tmp/file.txt
Real utility opens /tmp/file.txt → reads /etc/passwd
```

**Impact**: In adversarial scenarios, precise timing could bypass policy

**Mitigation Status**: Known limitation, documented for transparency

**Why Acceptable**:
- Primary use case is AI agent accountability, not adversarial defense
- AI agents don't actively exploit TOCTOU races
- Provides observability and guardrails, not security isolation
- Similar to firejail's userspace limitations

#### 2. Non-Wrapped Utilities

Only listed POSIX utilities are wrapped. Other commands bypass wrapper checks:

```bash
# Wrapped - policy enforced
cat /etc/shadow → checked

# Not wrapped - policy bypassed
perl -e 'open(F, "/etc/shadow"); print <F>' → not checked
python3 -c 'open("/etc/shadow").read()' → not checked
```

**Solution**: Blacklist interpreters if needed, or accept this as intended behavior for scripting languages.

#### 3. Wrapper Directory Whitelist

Wrappers must be whitelisted for execution (typically `/var/folders` on macOS, `/tmp` on Linux). This directory is user-readable.

**Impact**: Low - OS permissions prevent arbitrary writes to wrapper directory

**Recommendation**: Use most specific path pattern your profile supports

### Best Practices with Wrappers

#### 1. Understand Your Threat Model

```bash
# For AI agent guardrails (intended use):
✓ Use --wrap-coreutils for observability and accident prevention
✓ Rely on webhook accounting for audit trail
✓ Accept TOCTOU as documented limitation

# For adversarial scenarios (NOT intended use):
✗ Do not rely on wrappers as security boundary
✗ Use OS-level isolation (containers, namespaces, firejail)
✗ Implement kernel-level enforcement (AppArmor, SELinux)
```

#### 2. Combine with Shell Blacklisting

```bash
# Block nested shells to prevent wrapper bypass
blacklist /bin/bash
blacklist /bin/sh
blacklist /bin/zsh
```

#### 3. Use Webhook Observability

```bash
brush --profile policy.profile \
      --wrap-coreutils \
      --policy-webhook http://localhost:8080 \
      -c "commands here"

# Webhook receives JSON events for all policy checks:
# - Allowed operations (for audit trail)
# - Denied operations (for alerting)
# - File paths, access modes, timestamps
```

## Summary

Brushfire is a **first-layer policy enforcement tool** that controls what the shell does. It provides meaningful defense-in-depth for trusted environments but is not a security sandbox for untrusted code.

**Key principle**: Brushfire controls the shell's actions. Once a child process spawns, it operates independently with normal OS permissions.

**Coreutils wrappers**: Extend policy enforcement to common utilities, primarily for **AI agent accountability and observability**. TOCTOU races are a known limitation, acceptable for the intended use case of guardrails and audit trails rather than adversarial security.

Use brushfire to:
- Prevent shell-level mistakes
- Control script execution at entry point
- Add an extra security layer
- Provide AI agent accountability and observability
- Learn sandboxing concepts portably

Do not use brushfire as:
- Primary malware defense
- Process isolation tool
- Replacement for OS security
- Trusted computing base
- Defense against adversarial exploitation

## Safe `/dev/` Defaults

When `whitelist` directives enable default-deny filesystem mode, brushfire automatically whitelists these device files:

- `/dev/null`
- `/dev/zero`
- `/dev/urandom`
- `/dev/random`
- `/dev/stdin`
- `/dev/stdout`
- `/dev/stderr`
- `/dev/tty`

To disable automatic whitelisting:

```bash
no-safe-dev-defaults

whitelist /your/allowed/path
```

Without safe defaults, shell redirections to `/dev/null` and similar operations will fail with policy violations.

## Glob Expansion Information Disclosure

Shell glob patterns reveal directory structure in default-deny mode:

```bash
$ echo /etc/*
/etc/hosts /etc/passwd /etc/group ...
```

File contents remain protected:

```bash
$ cat /etc/passwd
Policy violation: File access denied (not whitelisted)
```

This occurs because shells expand globs before brushfire intercepts file operations. Cannot be prevented without breaking shell functionality.
