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

**Heuristic argument checking**: Brushfire inspects command arguments for file paths:

```bash
# Profile: blacklist /tmp/secret

$ brush --profile restricted.profile -c "cat /tmp/secret"
# ✗ BLOCKED - "/tmp/secret" detected as file path argument and checked against policy
```

**Limitation**: Non-obvious file access patterns may bypass detection:

```bash
$ brush --profile restricted.profile -c "python3 -c 'open(\"/tmp/secret\").read()'"
# ✓ ALLOWED - no file path arguments detected, Python opens file via syscall
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
→ ✗ BLOCKED (heuristic detects "/tmp/secret" as file path argument)

$ brush --profile test.profile -c "python3 -c 'import urllib; urllib.request.urlopen(...)'"
→ ✓ ALLOWED (python3 spawns, then python makes network calls directly)

$ brush --profile test.profile -c "/bin/sh -c 'curl example.com'"
→ ✓ ALLOWED if /bin/sh not blacklisted (sh spawns, then sh spawns curl)
```

## Heuristic Path Argument Checking

Brushfire automatically detects and checks file path arguments to ANY command:

### How It Works

1. Before spawning a process, Brushfire inspects all command arguments
2. Arguments that don't start with `-` are tested to see if they resolve to paths
3. Paths are canonicalized (resolving symlinks, relative paths, non-existent files)
4. Each detected path is checked against the policy (conservatively as Write access)
5. If any path violates policy, the command is blocked before spawning

### Coverage

This approach provides **universal coverage** - works for ANY command, not just specific utilities:

```bash
cat /etc/passwd          # ✓ Checked
grep pattern file.txt    # ✓ "file.txt" checked
python script.py         # ✓ "script.py" checked
custom-tool data.json    # ✓ "data.json" checked
```

### What Gets Checked

- **Existing files**: Direct canonicalization
- **New files**: Parent directory + filename (e.g., `touch /tmp/newfile.txt`)
- **Relative paths**: Resolved to absolute paths
- **Symlinks**: Resolved to canonical targets

### What Gets Skipped

- **Flags**: Arguments starting with `-`
- **Non-paths**: Strings that don't resolve to valid paths
- **Invalid paths**: Paths whose parent directories don't exist

### Examples

```bash
# Detects file path arguments
$ brush --profile policy.profile -c "cat /etc/passwd"
# ✗ BLOCKED - "/etc/passwd" detected and checked

# Skips non-path arguments
$ brush --profile policy.profile -c "grep pattern file.txt"
# "pattern" - not a valid path, skipped
# "file.txt" - detected and checked

# Handles new files
$ brush --profile policy.profile -c "touch /tmp/newfile.txt"
# ✗ BLOCKED - "/tmp/newfile.txt" (parent "/tmp" exists, checked)
```

### Known Limitations

**Conservative Access Mode**: All paths are checked with Write access mode by default. This may be overly restrictive but prevents accidental modifications.

**Embedded file paths**: Paths embedded in strings or constructed dynamically are not detected:

```bash
# Not detected:
python3 -c 'open("/etc/passwd").read()'  # Path in string literal
node -e 'fs.readFileSync("/etc/passwd")' # Path in code
```

This is acceptable for the intended use case (AI agent guardrails) - agents typically pass file paths as direct arguments.

### Webhook Observability

```bash
brush --profile policy.profile \
      --policy-webhook http://localhost:8080 \
      -c "commands here"

# Webhook receives JSON events for all policy checks:
# - Allowed operations (for audit trail)
# - Denied operations (for alerting)
# - File paths detected from arguments
# - Policy violation reasons
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
