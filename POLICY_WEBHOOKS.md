# Policy Webhook System Design

## Overview

Optional callback/webhook system that reports policy actions for observability. Designed to support future interactive allow/deny mechanism.

## MVP Requirements

- Report all policy checks (file access, command execution)
- JSON payload with full context
- HTTP POST to configurable webhook URL
- Non-blocking (async)
- Optional via CLI flag

## Future: Interactive Allow/Deny

Design supports future two-way communication:
- Webhook can return allow/deny decisions
- Timeout handling with fallback behavior
- Session-based decisions

## Architecture

### 1. PolicyReporter Trait

```rust
pub trait PolicyReporter: Send + Sync {
    fn report_check(&self, event: &PolicyEvent);
}
```

### 2. Event Structure

**File Access Event (read/write):**
```json
{
  "timestamp": "2025-01-15T10:30:45Z",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "event_type": "file_access",
  "action": {
    "resource": "/etc/passwd",
    "access_mode": "read",
    "resource_type": "file",
    "command": "cat /etc/passwd",
    "cwd": "/home/user"
  },
  "result": "denied",
  "reason": "file_blacklisted",
  "policy_profile": "/path/to/profile.policy"
}
```

**Execution Event (command spawn):**
```json
{
  "timestamp": "2025-01-15T10:30:45Z",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "event_type": "execution",
  "action": {
    "resource": "/usr/bin/curl",
    "access_mode": "execute",
    "resource_type": "command",
    "command": "/usr/bin/curl",
    "args": ["https://example.com"],
    "cwd": "/home/user"
  },
  "result": "denied",
  "reason": "command_blacklisted",
  "policy_profile": "/path/to/profile.policy"
}
```

**Execution Event (file execution):**
```json
{
  "timestamp": "2025-01-15T10:30:45Z",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "event_type": "execution",
  "action": {
    "resource": "/tmp/script.sh",
    "access_mode": "execute",
    "resource_type": "file",
    "cwd": "/home/user"
  },
  "result": "denied",
  "reason": "noexec_violation",
  "policy_profile": "/path/to/profile.policy"
}
```

### 3. Event Types

The event model reflects the two fundamental types of access control:

- `file_access`: File read/write operations (not execution)
  - Used for: file reads, writes, redirections
  - `action.access_mode`: "read" or "write"
  - `action.resource_type`: "file"

- `execution`: Command spawning or file execution
  - Used for: process spawning, script execution, noexec checks
  - `action.access_mode`: "execute"
  - `action.resource_type`: "command" (process spawn) or "file" (execution check)

- `network_check`: Network access check (future)

**Design Rationale:**

This simplified model accurately represents Brushfire's policy enforcement:
- **File access** controls data operations (reading/writing files)
- **Execution** controls code execution (spawning commands, executing scripts)

Both command spawning and file execution are unified under `execution` because they represent the same security boundary: running code. The `resource_type` field distinguishes whether it's a command being spawned or a file being checked for execution permission.

This design makes webhook consumers simpler—you only need to handle two event types to monitor all policy decisions.

### 4. Results

- `allowed`: Operation permitted
- `denied`: Operation blocked by policy

### 5. Reason Codes

Common reason codes for policy decisions:

**File Access:**
- `file_blacklisted`: Resource is explicitly blacklisted
- `file_not_whitelisted`: Resource not in whitelist (default-deny mode)
- `file_readonly`: Write operation denied by read-only rule
- `policy_check_passed`: Operation allowed by policy

**Execution:**
- `command_blacklisted`: Command is explicitly blacklisted
- `command_not_explicitly_allowed`: Command not whitelisted (default-deny mode)
- `command_explicitly_allowed`: Command explicitly whitelisted
- `noexec_violation`: Execution blocked by noexec rule
- `no_matching_rules`: Operation allowed (no blocking rules, default-allow mode)
- `policy_check_passed`: Operation allowed by policy

### 6. Webhook Backend

```rust
pub struct WebhookReporter {
    url: String,
    client: reqwest::Client,
    timeout: Duration,
}

impl WebhookReporter {
    pub fn new(url: String) -> Self {
        // Non-blocking HTTP client
        // Timeout: 2 seconds (don't block shell operations)
    }

    pub fn report(&self, event: &PolicyEvent) {
        // Spawn async task to POST event
        // Log errors but don't fail
    }
}
```

## Configuration

### CLI Flag

```bash
brush --policy-webhook https://example.com/policy-events [...]
```

### Environment Variable

```bash
BRUSHFIRE_WEBHOOK_URL=https://example.com/policy-events
```

## Implementation Notes

1. **Non-blocking**: Use `tokio::spawn` to send webhooks asynchronously
2. **Error handling**: Log webhook failures but don't block operations
3. **Session ID**: Generate once per shell session for correlation
4. **Timeout**: 2-second timeout for webhook calls
5. **Retries**: No retries in MVP (fire-and-forget)
6. **Authentication**: Support for Bearer tokens via header

## Future: Interactive Mode

```json
// Request (file access)
{
  "timestamp": "...",
  "event_type": "file_access",
  "action": {
    "resource": "/etc/passwd",
    "operation": "read",
    "resource_type": "file"
  },
  "awaiting_decision": true
}

// Request (execution)
{
  "timestamp": "...",
  "event_type": "execution",
  "action": {
    "resource": "/usr/bin/curl",
    "access_mode": "execute",
    "resource_type": "command",
    "args": ["https://example.com"]
  },
  "awaiting_decision": true
}

// Response
{
  "decision": "allow",
  "cache_duration": 3600,
  "reason": "approved by admin"
}
```

## Example: Processing Events

```python
from http.server import HTTPServer, BaseHTTPRequestHandler
import json

class PolicyEventHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        event = json.loads(self.rfile.read(content_length))

        # Handle different event types
        if event['event_type'] == 'file_access':
            self.handle_file_access(event)
        elif event['event_type'] == 'execution':
            self.handle_execution(event)

        self.send_response(200)
        self.end_headers()

    def handle_file_access(self, event):
        """Handle file read/write operations"""
        resource = event['action']['resource']
        access_mode = event['action']['access_mode']  # "read" or "write"
        result = event['result']

        if result == 'denied':
            print(f"BLOCKED: {access_mode} access to {resource}")
            self.alert_security_team(event)

    def handle_execution(self, event):
        """Handle command spawning and file execution"""
        resource = event['action']['resource']
        resource_type = event['action']['resource_type']  # "command" or "file"
        result = event['result']

        if result == 'denied':
            if resource_type == 'command':
                print(f"BLOCKED: Command execution: {resource}")
            else:
                print(f"BLOCKED: File execution: {resource}")
            self.alert_security_team(event)

    def alert_security_team(self, event):
        # Send alert to security monitoring system
        pass
```

## Security Considerations

1. Webhook URL should use HTTPS
2. Support for API key authentication
3. No sensitive data in payloads (configurable)
4. Rate limiting to prevent DoS on webhook endpoint
