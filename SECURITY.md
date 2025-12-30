# Brushfire Security Model

## What Brushfire Does

Brushfire provides **first-layer policy enforcement** for shell operations. It controls what the brush shell itself does, but **does not control spawned child processes**.

### Process Spawning

Brushfire controls which external commands the shell spawns:

```bash
# Profile: blacklist /usr/bin/curl

$ brush --profile restricted.profile -c "curl example.com"
error: Policy violation: Command execution denied
```

Using `whitelist` triggers default-deny mode for all commands and filesystem access:

```bash
# Profile: whitelist /usr/bin/ls

$ brush --profile restricted.profile -c "ls"
# Allowed - matches whitelist

$ brush --profile restricted.profile -c "curl example.com"
error: Policy violation: Command execution denied
# Denied - not in whitelist, default deny
```

**Limitation**: Once a command is allowed to spawn, it can spawn other programs:

```bash
# Profile: blacklist /usr/bin/curl
# BUT /bin/sh is NOT blacklisted

$ brush --profile restricted.profile -c "/bin/sh -c 'curl example.com'"
# curl WILL execute - /bin/sh is allowed and spawns curl directly
```

### Shell File Operations

Brushfire controls file operations performed by the shell:

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
# File is read successfully - brushfire only controlled the spawn
```

### Shell Builtin Commands

Brushfire controls builtin commands executed by the shell:

```bash
# Profile: blacklist /restricted/dir

$ brush --profile restricted.profile -c "cd /restricted/dir && pwd"
error: Policy violation: File access denied
```

## What Brushfire Does NOT Control

Once the shell spawns a child process, that process runs with normal OS permissions. Brushfire has no visibility into or control over what spawned processes do.

Programs can make direct system calls and library functions that brushfire cannot intercept:
- `open()`, `read()`, `write()` syscalls
- `fopen()`, `fread()`, `fwrite()` libc functions
- Network operations (`socket()`, `connect()`)
- Any FFI or native code operations

## Enforcement Examples

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

The `--wrap-coreutils` feature wraps common utilities with policy-checking proxies:

```bash
brush --profile policy.profile --wrap-coreutils -c "cat /etc/passwd"
# cat's file access is checked against policy before execution
```

### How It Works

1. Creates policy-aware wrappers for ~30 POSIX utilities
2. Prepends wrapper directory to PATH
3. Auto-blacklists real utilities to prevent bypass via absolute paths
4. Wrappers check arguments against policy before execing real utility

### Use Case

Coreutils wrappers are designed for **AI agent observability and accountability**, not adversarial security. They provide guardrails and audit trail for AI agents performing file operations.

### Wrapped Utilities

File operations: `cat`, `cp`, `mv`, `rm`, `ls`, `head`, `tail`, `touch`, `mkdir`, `ln`, `chmod`, `chown`, `chgrp`, `rmdir`, `dd`, `file`, `stat`

Text processing: `grep`, `sed`, `awk`, `cut`, `paste`, `sort`, `uniq`, `tr`, `wc`, `tee`, `diff`

Archive/search: `tar`, `find`

### Example

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

**Time-of-Check-Time-of-Use (TOCTOU)**: Race condition between policy check and file access:

```
Wrapper checks /tmp/file.txt → ALLOWED
                [race window]
Attacker: mv /tmp/file.txt /tmp/old && ln -s /etc/passwd /tmp/file.txt
Real utility opens /tmp/file.txt → reads /etc/passwd
```

This is acceptable for the intended use case (AI agent guardrails) but not for adversarial security.

**Non-Wrapped Utilities**: Only listed POSIX utilities are wrapped:

```bash
# Wrapped - policy enforced
cat /etc/shadow → checked

# Not wrapped - policy bypassed
perl -e 'open(F, "/etc/shadow"); print <F>' → not checked
python3 -c 'open("/etc/shadow").read()' → not checked
```

### Webhook Observability

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
