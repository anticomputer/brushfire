//! Policy enforcement engine.

use crate::error::PolicyViolation;
use crate::rules::{
    FileAccessMode, FilesystemRule, Policy, RuleAction,
};
use path_clean::PathClean;
use std::path::{Path, PathBuf};

#[cfg(feature = "webhook")]
use std::sync::Arc;

#[cfg(feature = "webhook")]
use crate::reporter::{CheckResult, PolicyEvent, PolicyReporter};

/// Policy enforcement engine.
pub struct PolicyEngine {
    policy: Policy,
    default_policy: DefaultPolicy,
    default_command_policy: DefaultPolicy,

    #[cfg(feature = "webhook")]
    reporter: Option<Arc<dyn PolicyReporter>>,
}

/// Default policy mode for filesystem access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultPolicy {
    /// Allow all access by default (permissive mode).
    AllowAll,
    /// Deny all access by default (restrictive mode, enabled by first whitelist).
    DenyAll,
}

impl PolicyEngine {
    /// Create a new policy engine with the given policy.
    #[must_use]
    pub fn new(mut policy: Policy) -> Self {
        // If there are any whitelist rules, switch to restrictive mode for filesystem
        let has_whitelist = policy
            .filesystem_rules
            .iter()
            .any(|rule| matches!(rule, FilesystemRule::Whitelist { .. }));

        let default_policy = if has_whitelist {
            DefaultPolicy::DenyAll
        } else {
            DefaultPolicy::AllowAll
        };

        // If there are any whitelist command rules, switch to restrictive mode for commands
        let has_allow_rules = policy
            .command_rules
            .iter()
            .any(|rule| rule.action == RuleAction::Allow);

        let default_command_policy = if has_allow_rules {
            DefaultPolicy::DenyAll
        } else {
            DefaultPolicy::AllowAll
        };

        // Add safe /dev defaults if in restrictive mode and enabled
        if default_policy == DefaultPolicy::DenyAll && policy.enable_safe_dev_defaults {
            Self::add_safe_dev_defaults(&mut policy);
        }

        Self {
            policy,
            default_policy,
            default_command_policy,
            #[cfg(feature = "webhook")]
            reporter: None,
        }
    }

    /// Add safe /dev device files to whitelist.
    ///
    /// Common device files that are generally safe to access:
    /// - /dev/null, /dev/zero - null devices
    /// - /dev/urandom, /dev/random - entropy sources
    /// - /dev/stdin, /dev/stdout, /dev/stderr - standard streams
    /// - /dev/tty - controlling terminal
    fn add_safe_dev_defaults(policy: &mut Policy) {
        const SAFE_DEV_FILES: &[&str] = &[
            "/dev/null",
            "/dev/zero",
            "/dev/urandom",
            "/dev/random",
            "/dev/stdin",
            "/dev/stdout",
            "/dev/stderr",
            "/dev/tty",
        ];

        for dev_file in SAFE_DEV_FILES {
            policy.add_filesystem_rule(FilesystemRule::Whitelist {
                path: PathBuf::from(dev_file),
                recursive: false,
            });
        }
    }

    /// Set the policy reporter for observability.
    ///
    /// # Arguments
    ///
    /// * `reporter` - The reporter to use for policy events
    #[cfg(feature = "webhook")]
    pub fn with_reporter(mut self, reporter: Arc<dyn PolicyReporter>) -> Self {
        self.reporter = Some(reporter);
        self
    }

    /// Set the policy reporter for observability (mutable version).
    ///
    /// # Arguments
    ///
    /// * `reporter` - The reporter to use for policy events
    #[cfg(feature = "webhook")]
    pub fn set_reporter(&mut self, reporter: Arc<dyn PolicyReporter>) {
        self.reporter = Some(reporter);
    }

    /// Check if a file operation is allowed.
    ///
    /// # Errors
    ///
    /// Returns a [`PolicyViolation`] if the operation is denied by policy.
    pub fn check_file_access(
        &self,
        path: &Path,
        mode: FileAccessMode,
    ) -> Result<(), PolicyViolation> {
        let canonical_path = self.canonicalize_path(path)?;

        // Check blacklist first (deny takes precedence)
        for rule in &self.policy.filesystem_rules {
            if let FilesystemRule::Blacklist {
                path: rule_path,
                recursive,
            } = rule
            {
                if self.path_matches(&canonical_path, rule_path, *recursive) {
                    #[cfg(feature = "webhook")]
                    if let Some(ref reporter) = self.reporter {
                        let event = PolicyEvent::file_access_check(
                            canonical_path.clone(),
                            mode,
                            CheckResult::Denied,
                            "file_blacklisted".to_string(),
                        );
                        reporter.report(&event);
                    }

                    return Err(PolicyViolation::FileBlacklisted(
                        canonical_path.display().to_string(),
                    ));
                }
            }
        }

        // Check read-only rules for write operations
        if mode == FileAccessMode::Write {
            for rule in &self.policy.filesystem_rules {
                if let FilesystemRule::ReadOnly {
                    path: rule_path,
                    recursive,
                } = rule
                {
                    if self.path_matches(&canonical_path, rule_path, *recursive) {
                        return Err(PolicyViolation::FileReadOnly(
                            canonical_path.display().to_string(),
                        ));
                    }
                }
            }
        }

        // Check noexec rules for execute operations
        if mode == FileAccessMode::Execute {
            for rule in &self.policy.filesystem_rules {
                if let FilesystemRule::NoExec {
                    path: rule_path,
                    recursive,
                } = rule
                {
                    if self.path_matches(&canonical_path, rule_path, *recursive) {
                        return Err(PolicyViolation::NoExec(
                            canonical_path.display().to_string(),
                        ));
                    }
                }
            }
        }

        // Check whitelist in restrictive mode
        if self.default_policy == DefaultPolicy::DenyAll {
            let allowed = self.policy.filesystem_rules.iter().any(|rule| {
                if let FilesystemRule::Whitelist {
                    path: rule_path,
                    recursive,
                } = rule
                {
                    self.path_matches(&canonical_path, rule_path, *recursive)
                } else {
                    false
                }
            });

            if !allowed {
                #[cfg(feature = "webhook")]
                if let Some(ref reporter) = self.reporter {
                    let event = PolicyEvent::file_access_check(
                        canonical_path.clone(),
                        mode,
                        CheckResult::Denied,
                        "file_not_whitelisted".to_string(),
                    );
                    reporter.report(&event);
                }

                return Err(PolicyViolation::FileNotWhitelisted(
                    canonical_path.display().to_string(),
                ));
            }
        }

        // Report allowed access
        #[cfg(feature = "webhook")]
        if let Some(ref reporter) = self.reporter {
            let event = PolicyEvent::file_access_check(
                canonical_path,
                mode,
                CheckResult::Allowed,
                "policy_check_passed".to_string(),
            );
            reporter.report(&event);
        }

        Ok(())
    }

    /// Check if a process spawn is allowed.
    ///
    /// # Errors
    ///
    /// Returns a [`PolicyViolation`] if the command is blocked by policy.
    #[cfg_attr(not(feature = "webhook"), allow(unused_variables))]
    pub fn check_process_spawn(&self, command_path: &Path, args: Option<Vec<String>>) -> Result<(), PolicyViolation> {
        let canonical_path = self.canonicalize_path(command_path)?;

        for rule in &self.policy.command_rules {
            if self.command_matches(&canonical_path, &rule.pattern) {
                match rule.action {
                    RuleAction::Deny => {
                        #[cfg(feature = "webhook")]
                        if let Some(ref reporter) = self.reporter {
                            let event = PolicyEvent::command_spawn_check(
                                canonical_path.clone(),
                                args.clone(),
                                CheckResult::Denied,
                                "command_blacklisted".to_string(),
                            );
                            reporter.report(&event);
                        }

                        return Err(PolicyViolation::CommandBlocked(
                            canonical_path.display().to_string(),
                        ));
                    }
                    RuleAction::Allow => {
                        #[cfg(feature = "webhook")]
                        if let Some(ref reporter) = self.reporter {
                            let event = PolicyEvent::command_spawn_check(
                                canonical_path.clone(),
                                args.clone(),
                                CheckResult::Allowed,
                                "command_explicitly_allowed".to_string(),
                            );
                            reporter.report(&event);
                        }

                        return Ok(());
                    }
                }
            }
        }

        // No rules matched - check default command policy
        if self.default_command_policy == DefaultPolicy::DenyAll {
            #[cfg(feature = "webhook")]
            if let Some(ref reporter) = self.reporter {
                let event = PolicyEvent::command_spawn_check(
                    canonical_path.clone(),
                    args.clone(),
                    CheckResult::Denied,
                    "command_not_explicitly_allowed".to_string(),
                );
                reporter.report(&event);
            }

            return Err(PolicyViolation::CommandBlocked(
                canonical_path.display().to_string(),
            ));
        }

        // Report allowed spawn (no matching rules, default allow mode)
        #[cfg(feature = "webhook")]
        if let Some(ref reporter) = self.reporter {
            let event = PolicyEvent::command_spawn_check(
                canonical_path,
                args,
                CheckResult::Allowed,
                "no_matching_rules".to_string(),
            );
            reporter.report(&event);
        }

        Ok(())
    }

    /// Canonicalize a path, handling non-existent paths gracefully.
    fn canonicalize_path(&self, path: &Path) -> Result<PathBuf, PolicyViolation> {
        // First try direct canonicalization
        if let Ok(canonical) = path.canonicalize() {
            return Ok(canonical);
        }

        // If that fails, try to canonicalize the parent and append the filename
        if let Some(parent) = path.parent() {
            if let Ok(canonical_parent) = parent.canonicalize() {
                if let Some(filename) = path.file_name() {
                    return Ok(canonical_parent.join(filename));
                }
            }
        }

        // Last resort: normalize the path using path-clean
        // This resolves . and .. components even when the path doesn't exist
        if path.is_absolute() {
            Ok(path.clean())
        } else {
            // Make relative path absolute first, then normalize
            std::env::current_dir()
                .map(|cwd| cwd.join(path).clean())
                .or_else(|_| Ok(path.clean()))
        }
    }

    /// Check if a test path matches a rule path.
    fn path_matches(&self, test_path: &Path, rule_path: &Path, recursive: bool) -> bool {
        // Canonicalize the rule path for comparison
        let canonical_rule = self.canonicalize_path(rule_path).ok();

        if let Some(ref canonical_rule) = canonical_rule {
            if recursive {
                test_path.starts_with(canonical_rule)
            } else {
                test_path == canonical_rule
            }
        } else {
            // Fallback to direct comparison if canonicalization fails
            if recursive {
                test_path.starts_with(rule_path)
            } else {
                test_path == rule_path
            }
        }
    }

    /// Check if a command path matches a glob pattern.
    fn command_matches(&self, command_path: &Path, pattern: &str) -> bool {
        glob::Pattern::new(pattern)
            .ok()
            .and_then(|p| Some(p.matches_path(command_path)))
            .unwrap_or(false)
    }

    /// Add a blacklist rule dynamically to the policy.
    ///
    /// This is used by the `--wrap-coreutils` feature to automatically blacklist
    /// real utility paths to prevent bypass via absolute paths.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to blacklist
    /// * `recursive` - Whether to blacklist recursively
    pub fn add_blacklist_rule(&mut self, path: PathBuf, recursive: bool) {
        self.policy.add_filesystem_rule(FilesystemRule::Blacklist {
            path,
            recursive,
        });
    }

    /// Add a filesystem rule dynamically to the policy.
    ///
    /// This is used by the `--wrap-coreutils` feature to automatically whitelist
    /// wrapper directories in default-deny mode.
    ///
    /// # Arguments
    ///
    /// * `rule` - The filesystem rule to add
    pub fn add_filesystem_rule(&mut self, rule: FilesystemRule) {
        self.policy.add_filesystem_rule(rule);
    }

    /// Add a command rule dynamically to the policy.
    ///
    /// This is used by the `--wrap-coreutils` feature to automatically blacklist
    /// real utility paths to prevent bypass via absolute paths.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The command pattern to block (e.g., "/bin/cat")
    pub fn add_command_deny_rule(&mut self, pattern: String) {
        self.policy.add_command_rule(crate::rules::CommandRule {
            pattern,
            action: crate::rules::RuleAction::Deny,
        });
    }

    /// Add a command allow rule dynamically to the policy.
    ///
    /// This is used by the `--wrap-coreutils` feature to automatically whitelist
    /// wrapper paths when in default-deny mode.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The command pattern to allow (e.g., "/tmp/wrappers/*")
    pub fn add_command_allow_rule(&mut self, pattern: String) {
        self.policy.add_command_rule(crate::rules::CommandRule {
            pattern,
            action: crate::rules::RuleAction::Allow,
        });
    }

    /// Check if command execution is in default-deny mode.
    ///
    /// Returns true if whitelist rules are active and commands are
    /// denied by default unless explicitly allowed.
    #[must_use]
    pub fn is_command_default_deny(&self) -> bool {
        self.default_command_policy == DefaultPolicy::DenyAll
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{CommandRule, FilesystemRule};

    #[test]
    fn test_blacklist_enforcement() {
        let mut policy = Policy::new();
        policy.add_filesystem_rule(FilesystemRule::Blacklist {
            path: PathBuf::from("/etc/shadow"),
            recursive: false,
        });

        let engine = PolicyEngine::new(policy);

        // Should block access to blacklisted file
        assert!(engine
            .check_file_access(&PathBuf::from("/etc/shadow"), FileAccessMode::Read)
            .is_err());

        // Should allow access to non-blacklisted file
        assert!(engine
            .check_file_access(&PathBuf::from("/etc/hosts"), FileAccessMode::Read)
            .is_ok());
    }

    #[test]
    fn test_readonly_enforcement() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        let mut policy = Policy::new();
        policy.add_filesystem_rule(FilesystemRule::ReadOnly {
            path: readonly_dir.clone(),
            recursive: true,
        });

        let engine = PolicyEngine::new(policy);
        let test_file = readonly_dir.join("file");

        // Should allow reads
        assert!(engine
            .check_file_access(&test_file, FileAccessMode::Read)
            .is_ok());

        // Should block writes
        assert!(engine
            .check_file_access(&test_file, FileAccessMode::Write)
            .is_err());
    }

    #[test]
    fn test_noexec_enforcement() {
        let mut policy = Policy::new();
        policy.add_filesystem_rule(FilesystemRule::NoExec {
            path: PathBuf::from("/tmp"),
            recursive: true,
        });

        let engine = PolicyEngine::new(policy);

        // Should block execution
        assert!(engine
            .check_file_access(&PathBuf::from("/tmp/script.sh"), FileAccessMode::Execute)
            .is_err());

        // Should allow read/write
        assert!(engine
            .check_file_access(&PathBuf::from("/tmp/file.txt"), FileAccessMode::Read)
            .is_ok());
    }

    #[test]
    fn test_command_blocking() {
        let mut policy = Policy::new();
        policy.add_command_rule(CommandRule {
            pattern: "/usr/bin/curl".to_string(),
            action: RuleAction::Deny,
        });

        let engine = PolicyEngine::new(policy);

        // Should block curl (if it exists)
        if PathBuf::from("/usr/bin/curl").exists() {
            assert!(engine
                .check_process_spawn(&PathBuf::from("/usr/bin/curl"), None)
                .is_err());
        }
    }

    #[test]
    fn test_whitelist_restrictive_mode() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path().join("allowed");
        let other_dir = temp_dir.path().join("other");
        fs::create_dir(&allowed_dir).unwrap();
        fs::create_dir(&other_dir).unwrap();

        let mut policy = Policy::new();
        policy.add_filesystem_rule(FilesystemRule::Whitelist {
            path: allowed_dir.clone(),
            recursive: true,
        });

        let engine = PolicyEngine::new(policy);

        // Should be in restrictive mode
        assert_eq!(engine.default_policy, DefaultPolicy::DenyAll);

        // Should allow whitelisted paths
        assert!(engine
            .check_file_access(&allowed_dir.join("file"), FileAccessMode::Read)
            .is_ok());

        // Should block non-whitelisted paths
        assert!(engine
            .check_file_access(&other_dir.join("file"), FileAccessMode::Read)
            .is_err());
    }
}
