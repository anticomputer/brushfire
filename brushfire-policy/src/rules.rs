//! Policy rule data structures.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A complete policy loaded from one or more firejail profiles.
#[derive(Debug, Clone)]
pub struct Policy {
    /// Filesystem access rules.
    pub filesystem_rules: Vec<FilesystemRule>,
    /// Command execution rules.
    pub command_rules: Vec<CommandRule>,
    /// Commands that require CWD access checking when invoked without path arguments.
    /// Stores canonical command paths for efficient lookup.
    pub cwd_checking_commands: HashSet<String>,
    /// Macro definitions for variable expansion.
    pub macros: HashMap<String, String>,
    /// Whether to enable safe /dev defaults in restrictive mode.
    /// When true (default), common /dev files are auto-whitelisted in default-deny mode.
    pub enable_safe_dev_defaults: bool,
    /// Prompt messages to display at shell startup.
    /// Multiple prompt directives can be used to build up documentation in sections.
    pub prompts: Vec<String>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            filesystem_rules: Vec::new(),
            command_rules: Vec::new(),
            cwd_checking_commands: HashSet::new(),
            macros: HashMap::new(),
            enable_safe_dev_defaults: true,
            prompts: Vec::new(),
        }
    }
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
    /// This removes both filesystem blacklist rules and command Deny rules.
    pub fn remove_blacklist(&mut self, path: &PathBuf) {
        // Remove filesystem blacklist rules
        self.filesystem_rules.retain(|rule| {
            !matches!(rule, FilesystemRule::Blacklist { path: rule_path, .. } if rule_path == path)
        });

        // Remove command Deny rules for this path
        let pattern = path.display().to_string();
        self.command_rules.retain(|rule| {
            !(rule.pattern == pattern && rule.action == RuleAction::Deny)
        });
    }

    /// Add a command that requires CWD checking when invoked without path arguments.
    ///
    /// # Arguments
    ///
    /// * `command_path` - Path to the command executable
    pub fn add_cwd_checking_command(&mut self, command_path: &Path) {
        // Store both the original and canonical forms for matching
        let original = command_path.display().to_string();
        self.cwd_checking_commands.insert(original);

        // Also add canonical form to handle symlinks
        if let Ok(canonical) = command_path.canonicalize() {
            let canonical_str = canonical.display().to_string();
            self.cwd_checking_commands.insert(canonical_str);
        }
    }

    /// Check if a command requires CWD checking when invoked without path arguments.
    ///
    /// # Arguments
    ///
    /// * `command_path` - Path to the command executable
    ///
    /// # Returns
    ///
    /// `true` if the command requires CWD checking, `false` otherwise.
    pub fn requires_cwd_checking(&self, command_path: &Path) -> bool {
        // Check against stored patterns
        let path_str = command_path.display().to_string();
        if self.cwd_checking_commands.contains(&path_str) {
            return true;
        }

        // Also check canonical form
        if let Ok(canonical) = command_path.canonicalize() {
            let canonical_str = canonical.display().to_string();
            if self.cwd_checking_commands.contains(&canonical_str) {
                return true;
            }
        }

        false
    }
}
