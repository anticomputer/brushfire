//! Policy rule data structures.

use std::collections::HashMap;
use std::path::PathBuf;

/// A complete policy loaded from one or more firejail profiles.
#[derive(Debug, Clone, Default)]
pub struct Policy {
    /// Filesystem access rules.
    pub filesystem_rules: Vec<FilesystemRule>,
    /// Command execution rules.
    pub command_rules: Vec<CommandRule>,
    /// Macro definitions for variable expansion.
    pub macros: HashMap<String, String>,
}

/// Filesystem access control rules.
#[derive(Debug, Clone)]
pub enum FilesystemRule {
    /// Allow access to a path (recursive by default).
    Whitelist {
        /// Path to whitelist.
        path: PathBuf,
        /// Whether to apply recursively to subdirectories.
        recursive: bool,
    },
    /// Deny access to a path (recursive by default).
    Blacklist {
        /// Path to blacklist.
        path: PathBuf,
        /// Whether to apply recursively to subdirectories.
        recursive: bool,
    },
    /// Allow only read access to a path.
    ReadOnly {
        /// Path to make read-only.
        path: PathBuf,
        /// Whether to apply recursively to subdirectories.
        recursive: bool,
    },
    /// Prevent execution from a directory.
    NoExec {
        /// Directory where execution is forbidden.
        path: PathBuf,
        /// Whether to apply recursively to subdirectories.
        recursive: bool,
    },
}

/// Command execution control rules.
#[derive(Debug, Clone)]
pub struct CommandRule {
    /// Glob pattern matching command paths (e.g., "/bin/curl", "/usr/bin/*").
    pub pattern: String,
    /// Action to take when pattern matches.
    pub action: RuleAction,
}

/// Action to take when a rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleAction {
    /// Allow the operation.
    Allow,
    /// Deny the operation.
    Deny,
}

/// File access mode for policy checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAccessMode {
    /// Read access.
    Read,
    /// Write access.
    Write,
    /// Execute access.
    Execute,
}

impl Policy {
    /// Create a new empty policy.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a filesystem rule to the policy.
    pub fn add_filesystem_rule(&mut self, rule: FilesystemRule) {
        self.filesystem_rules.push(rule);
    }

    /// Add a command rule to the policy.
    pub fn add_command_rule(&mut self, rule: CommandRule) {
        self.command_rules.push(rule);
    }

    /// Remove blacklist rules for a path (noblacklist directive).
    pub fn remove_blacklist(&mut self, path: &PathBuf) {
        self.filesystem_rules.retain(|rule| {
            !matches!(rule, FilesystemRule::Blacklist { path: rule_path, .. } if rule_path == path)
        });
    }
}
