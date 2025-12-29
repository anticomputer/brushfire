//! Error types for brushfire policy engine.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during policy parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    /// Unknown directive in profile.
    #[error("Unknown directive: {0}")]
    UnknownDirective(String),

    /// Invalid directive format or arguments.
    #[error("Invalid directive format: {0}")]
    InvalidFormat(String),

    /// Include file not found.
    #[error("Include file not found: {0}")]
    IncludeNotFound(PathBuf),

    /// Maximum include depth exceeded.
    #[error("Maximum include depth ({0}) exceeded")]
    MaxIncludeDepth(usize),

    /// I/O error while reading profile.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur during policy enforcement.
#[derive(Error, Debug)]
pub enum PolicyViolation {
    /// File is blacklisted.
    #[error("File access denied (blacklisted): {0}")]
    FileBlacklisted(String),

    /// File is not whitelisted in restrictive mode.
    #[error("File access denied (not whitelisted): {0}")]
    FileNotWhitelisted(String),

    /// File is read-only.
    #[error("File is read-only: {0}")]
    FileReadOnly(String),

    /// Execution is not allowed from this directory.
    #[error("Execution denied from directory (noexec): {0}")]
    NoExec(String),

    /// Command is blocked.
    #[error("Command execution denied (blacklisted): {0}")]
    CommandBlocked(String),

    /// Path canonicalization failed.
    #[error("Failed to canonicalize path {0}: {1}")]
    CanonicalizationFailed(String, std::io::Error),
}
