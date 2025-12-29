//! Brushfire policy engine - firejail-style ACL enforcement for brush shell.
//!
//! This crate provides portable, cross-platform security policy enforcement
//! without requiring OS-level features like namespaces or seccomp. It's
//! designed for defense-in-depth, not as a primary security boundary.
//!
//! # Example
//!
//! ```no_run
//! use brushfire_policy::{ProfileParser, PolicyEngine, FileAccessMode};
//! use std::path::Path;
//!
//! // Parse a firejail profile
//! let mut parser = ProfileParser::new();
//! let policy = parser.parse_file(Path::new("firefox.profile")).unwrap();
//!
//! // Create enforcement engine
//! let engine = PolicyEngine::new(policy);
//!
//! // Check file access
//! match engine.check_file_access(Path::new("/etc/shadow"), FileAccessMode::Read) {
//!     Ok(()) => println!("Access allowed"),
//!     Err(e) => println!("Access denied: {}", e),
//! }
//! ```

pub mod engine;
pub mod error;
pub mod macros;
pub mod parser;
pub mod rules;

// Re-export main types for convenience
pub use engine::{DefaultPolicy, PolicyEngine};
pub use error::{ParseError, PolicyViolation};
pub use macros::MacroExpander;
pub use parser::ProfileParser;
pub use rules::{
    CommandRule, FileAccessMode, FilesystemRule, Policy, RuleAction,
};
