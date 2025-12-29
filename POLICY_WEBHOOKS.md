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

```json
{
  "timestamp": "2025-01-15T10:30:45Z",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "event_type": "file_access_check",
  "action": {
    "resource": "/etc/passwd",
    "mode": "read",
    "command": "cat /etc/passwd",
    "cwd": "/home/user"
  },
  "result": "denied",
  "reason": "file_blacklisted",
  "policy_profile": "/path/to/profile.policy"
}
```

### 3. Event Types

- `file_access_check`: Check file/directory access
- `command_spawn_check`: Check command execution
- `network_check`: Check network access (future)

### 4. Results

- `allowed`: Operation permitted
- `denied`: Operation blocked by policy

### 5. Webhook Backend

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
// Request
{
  "timestamp": "...",
  "event_type": "file_access_check",
  "action": { "resource": "/etc/passwd", "mode": "read" },
  "awaiting_decision": true
}

// Response
{
  "decision": "allow",
  "cache_duration": 3600,
  "reason": "approved by admin"
}
```

## Security Considerations

1. Webhook URL should use HTTPS
2. Support for API key authentication
3. No sensitive data in payloads (configurable)
4. Rate limiting to prevent DoS on webhook endpoint
