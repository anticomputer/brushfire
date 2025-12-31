//! Policy event reporting for observability.
//!
//! This module provides a callback/webhook system for reporting policy actions
//! and their outcomes. Designed for observability in MVP, with support for
//! future interactive allow/deny mechanisms.

use crate::FileAccessMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// A policy event that can be reported to external systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvent {
    /// Timestamp of the event in ISO 8601 format
    pub timestamp: String,

    /// Session identifier for correlating events
    pub session_id: String,

    /// Type of policy check
    pub event_type: EventType,

    /// Details of the action being checked
    pub action: ActionContext,

    /// Result of the policy check
    pub result: CheckResult,

    /// Reason for the result (e.g., "file_blacklisted")
    pub reason: String,

    /// Path to the policy profile being used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_profile: Option<String>,
}

/// Type of policy event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// File read/write access check
    FileAccess,

    /// Command or file execution check
    Execution,

    /// Network access check (future)
    NetworkCheck,
}

/// Context about the action being checked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContext {
    /// Resource being accessed (file path, command path, etc.)
    pub resource: String,

    /// Operation type: "read", "write", "execute"
    pub operation: String,

    /// Resource type: "file" or "command"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,

    /// Command that triggered the check (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Command line arguments (for execution checks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    /// Current working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

/// Result of a policy check
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckResult {
    /// Operation was allowed
    Allowed,

    /// Operation was denied
    Denied,
}

/// Trait for reporting policy events to external systems.
pub trait PolicyReporter: Send + Sync {
    /// Report a policy event.
    ///
    /// This should be non-blocking and should not fail even if the
    /// reporting mechanism is unavailable.
    fn report(&self, event: &PolicyEvent);
}

/// Webhook-based policy reporter that sends events via HTTP POST.
#[derive(Debug, Clone)]
pub struct WebhookReporter {
    url: String,
    client: Arc<reqwest::Client>,
    session_id: String,
}

impl WebhookReporter {
    /// Create a new webhook reporter.
    ///
    /// # Arguments
    ///
    /// * `url` - The webhook URL to POST events to
    /// * `session_id` - Unique session identifier for this shell instance
    pub fn new(url: String, session_id: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            url,
            client: Arc::new(client),
            session_id,
        }
    }

    /// Create a new webhook reporter with authentication.
    ///
    /// # Arguments
    ///
    /// * `url` - The webhook URL to POST events to
    /// * `session_id` - Unique session identifier for this shell instance
    /// * `auth_token` - Bearer token for authentication
    pub fn with_auth(url: String, session_id: String, auth_token: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth_header = format!("Bearer {}", auth_token);
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&auth_header).unwrap(),
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            url,
            client: Arc::new(client),
            session_id,
        }
    }

    fn send_async(&self, event: PolicyEvent) {
        let url = self.url.clone();
        let client = Arc::clone(&self.client);

        // Try to use tokio runtime if available, otherwise spawn a thread
        match tokio::runtime::Handle::try_current() {
            Ok(_handle) => {
                // We're in a Tokio runtime, use tokio::spawn (fire-and-forget)
                tokio::spawn(async move {
                    match client.post(&url).json(&event).send().await {
                        Ok(response) => {
                            if !response.status().is_success() {
                                eprintln!(
                                    "brushfire webhook: HTTP {} from {}",
                                    response.status(),
                                    url
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("brushfire webhook error: {}", e);
                        }
                    }
                });
            }
            Err(_) => {
                // No Tokio runtime available, create one and WAIT for completion
                // This is important for short-lived processes that exit immediately
                let handle = std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("Failed to create Tokio runtime");

                    rt.block_on(async move {
                        match client.post(&url).json(&event).send().await {
                            Ok(response) => {
                                if !response.status().is_success() {
                                    eprintln!(
                                        "brushfire webhook: HTTP {} from {}",
                                        response.status(),
                                        url
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!("brushfire webhook error: {}", e);
                            }
                        }
                    });
                });

                // Wait for the webhook to complete (blocking)
                // This ensures the process doesn't exit before the webhook is sent
                let _ = handle.join();
            }
        }
    }
}

impl PolicyReporter for WebhookReporter {
    fn report(&self, event: &PolicyEvent) {
        // Add session_id if not already set
        let mut event = event.clone();
        if event.session_id.is_empty() {
            event.session_id = self.session_id.clone();
        }

        self.send_async(event);
    }
}

/// Helper to create policy events
impl PolicyEvent {
    /// Create a new file access check event (read or write)
    pub fn file_access(
        resource: PathBuf,
        mode: FileAccessMode,
        result: CheckResult,
        reason: String,
    ) -> Self {
        let operation = match mode {
            FileAccessMode::Read => "read",
            FileAccessMode::Write => "write",
            FileAccessMode::Execute => "execute", // Should not be used for FileAccess events
        };

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: String::new(), // Will be filled by reporter
            event_type: EventType::FileAccess,
            action: ActionContext {
                resource: resource.display().to_string(),
                operation: operation.to_string(),
                resource_type: Some("file".to_string()),
                command: std::env::args().collect::<Vec<_>>().get(0).cloned(),
                args: None,
                cwd: std::env::current_dir()
                    .ok()
                    .map(|p| p.display().to_string()),
            },
            result,
            reason,
            policy_profile: None, // Can be set later
        }
    }

    /// Create a new file execution check event
    pub fn file_execution(
        resource: PathBuf,
        result: CheckResult,
        reason: String,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: String::new(),
            event_type: EventType::Execution,
            action: ActionContext {
                resource: resource.display().to_string(),
                operation: "execute".to_string(),
                resource_type: Some("file".to_string()),
                command: std::env::args().collect::<Vec<_>>().get(0).cloned(),
                args: None,
                cwd: std::env::current_dir()
                    .ok()
                    .map(|p| p.display().to_string()),
            },
            result,
            reason,
            policy_profile: None,
        }
    }

    /// Create a new command execution check event
    pub fn command_execution(
        command: PathBuf,
        args: Option<Vec<String>>,
        result: CheckResult,
        reason: String,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: String::new(),
            event_type: EventType::Execution,
            action: ActionContext {
                resource: command.display().to_string(),
                operation: "execute".to_string(),
                resource_type: Some("command".to_string()),
                command: Some(command.display().to_string()),
                args,
                cwd: std::env::current_dir()
                    .ok()
                    .map(|p| p.display().to_string()),
            },
            result,
            reason,
            policy_profile: None,
        }
    }
}
