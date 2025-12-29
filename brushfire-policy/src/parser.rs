//! Parser for firejail profile files.

use crate::error::ParseError;
use crate::macros::MacroExpander;
use crate::rules::{CommandRule, FilesystemRule, Policy, RuleAction};
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum include depth to prevent infinite recursion.
const MAX_INCLUDE_DEPTH: usize = 10;

/// Parser for firejail profile files.
#[derive(Debug)]
pub struct ProfileParser {
    macros: MacroExpander,
    include_depth: usize,
}

impl ProfileParser {
    /// Create a new profile parser.
    #[must_use]
    pub fn new() -> Self {
        Self {
            macros: MacroExpander::new(),
            include_depth: 0,
        }
    }

    /// Parse a firejail profile file.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the file cannot be read or parsed.
    pub fn parse_file(&mut self, path: &Path) -> Result<Policy, ParseError> {
        // Check include depth
        if self.include_depth >= MAX_INCLUDE_DEPTH {
            return Err(ParseError::MaxIncludeDepth(MAX_INCLUDE_DEPTH));
        }

        // Read the file
        let content = fs::read_to_string(path)?;

        // Parse the content
        let mut policy = Policy::new();
        policy.macros = self.macros.vars.clone();

        for (line_num, line) in content.lines().enumerate() {
            // Strip comments
            let line = if let Some(pos) = line.find('#') {
                &line[..pos]
            } else {
                line
            };

            // Trim whitespace
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Parse the directive
            match self.parse_directive(line, path) {
                Ok(Some(directive)) => self.apply_directive(&mut policy, directive)?,
                Ok(None) => {} // Directive handled internally (e.g., include)
                Err(e) => {
                    return Err(ParseError::InvalidFormat(format!(
                        "Line {}: {} - {}",
                        line_num + 1,
                        line,
                        e
                    )));
                }
            }
        }

        Ok(policy)
    }

    /// Parse a single directive line.
    fn parse_directive(
        &mut self,
        line: &str,
        current_file: &Path,
    ) -> Result<Option<Directive>, ParseError> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return Ok(None);
        }

        match parts[0] {
            "whitelist" => {
                let path = self.parse_path(&parts[1..])?;
                Ok(Some(Directive::Whitelist(path)))
            }
            "blacklist" => {
                let path = self.parse_path(&parts[1..])?;
                Ok(Some(Directive::Blacklist(path)))
            }
            "noblacklist" => {
                let path = self.parse_path(&parts[1..])?;
                Ok(Some(Directive::NoBlacklist(path)))
            }
            "whitelist_exec" => {
                let path = self.parse_path(&parts[1..])?;
                Ok(Some(Directive::WhitelistExec(path)))
            }
            "read-only" => {
                let path = self.parse_path(&parts[1..])?;
                Ok(Some(Directive::ReadOnly(path)))
            }
            "noexec" => {
                let path = self.parse_path(&parts[1..])?;
                Ok(Some(Directive::NoExec(path)))
            }
            "include" => {
                let include_path = self.parse_include_path(&parts[1..], current_file)?;
                self.include_depth += 1;
                let included_policy = self.parse_file(&include_path)?;
                self.include_depth -= 1;
                Ok(Some(Directive::Include(included_policy)))
            }
            _ => {
                // Unknown directive - ignore for now to support partial firejail syntax
                Ok(None)
            }
        }
    }

    /// Parse a path argument with macro expansion.
    fn parse_path(&self, parts: &[&str]) -> Result<PathBuf, ParseError> {
        if parts.is_empty() {
            return Err(ParseError::InvalidFormat(
                "Missing path argument".to_string(),
            ));
        }

        // Join all parts (in case path has spaces)
        let path_str = parts.join(" ");

        // Expand macros
        let expanded = self.macros.expand(&path_str);

        Ok(PathBuf::from(expanded))
    }

    /// Parse an include path, resolving relative to current file or system paths.
    fn parse_include_path(
        &self,
        parts: &[&str],
        current_file: &Path,
    ) -> Result<PathBuf, ParseError> {
        let path = self.parse_path(parts)?;

        // If absolute, use as-is
        if path.is_absolute() {
            if path.exists() {
                return Ok(path);
            }
            return Err(ParseError::IncludeNotFound(path));
        }

        // Try relative to current file
        if let Some(parent) = current_file.parent() {
            let relative_path = parent.join(&path);
            if relative_path.exists() {
                return Ok(relative_path);
            }
        }

        // Try system include paths
        let system_paths = [
            PathBuf::from("/etc/firejail"),
            PathBuf::from("/etc/firejail/inc"),
        ];

        for sys_path in &system_paths {
            let full_path = sys_path.join(&path);
            if full_path.exists() {
                return Ok(full_path);
            }
        }

        Err(ParseError::IncludeNotFound(path))
    }

    /// Apply a directive to the policy.
    fn apply_directive(&self, policy: &mut Policy, directive: Directive) -> Result<(), ParseError> {
        match directive {
            Directive::Whitelist(path) => {
                policy.add_filesystem_rule(FilesystemRule::Whitelist {
                    path,
                    recursive: true,
                });
            }
            Directive::Blacklist(path) => {
                // Check if this is a command path (starts with /bin, /usr/bin, etc.)
                if is_command_path(&path) {
                    policy.add_command_rule(CommandRule {
                        pattern: path.display().to_string(),
                        action: RuleAction::Deny,
                    });
                } else {
                    policy.add_filesystem_rule(FilesystemRule::Blacklist {
                        path,
                        recursive: true,
                    });
                }
            }
            Directive::NoBlacklist(path) => {
                policy.remove_blacklist(&path);
            }
            Directive::WhitelistExec(path) => {
                // Create a command rule with Allow action
                let pattern = path.display().to_string();
                policy.add_command_rule(CommandRule {
                    pattern: pattern.clone(),
                    action: RuleAction::Allow,
                });

                // Also add canonicalized version to handle symlinks (e.g., /tmp -> /private/tmp)
                if let Some(canonical_pattern) = canonicalize_pattern(&path) {
                    let canonical_str = canonical_pattern.display().to_string();
                    if canonical_str != pattern {
                        policy.add_command_rule(CommandRule {
                            pattern: canonical_str,
                            action: RuleAction::Allow,
                        });
                    }
                }
            }
            Directive::ReadOnly(path) => {
                policy.add_filesystem_rule(FilesystemRule::ReadOnly {
                    path,
                    recursive: true,
                });
            }
            Directive::NoExec(path) => {
                policy.add_filesystem_rule(FilesystemRule::NoExec {
                    path,
                    recursive: true,
                });
            }
            Directive::Include(included_policy) => {
                // Merge included policy into current policy
                policy.filesystem_rules.extend(included_policy.filesystem_rules);
                policy.command_rules.extend(included_policy.command_rules);
            }
        }

        Ok(())
    }
}

impl Default for ProfileParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal directive representation.
#[derive(Debug)]
enum Directive {
    Whitelist(PathBuf),
    Blacklist(PathBuf),
    NoBlacklist(PathBuf),
    WhitelistExec(PathBuf),
    ReadOnly(PathBuf),
    NoExec(PathBuf),
    Include(Policy),
}

/// Check if a path looks like a command path.
fn is_command_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.starts_with("/bin/")
        || path_str.starts_with("/usr/bin/")
        || path_str.starts_with("/usr/local/bin/")
        || path_str.starts_with("/sbin/")
        || path_str.starts_with("/usr/sbin/")
}

/// Try to canonicalize a glob pattern by canonicalizing its base path.
/// For patterns like `/tmp/bin/*`, this extracts `/tmp/bin`, canonicalizes it,
/// and reconstructs the pattern as `/private/tmp/bin/*` (on macOS where /tmp is a symlink).
fn canonicalize_pattern(pattern: &Path) -> Option<PathBuf> {
    let pattern_str = pattern.to_string_lossy();

    // Find the first wildcard character
    let wildcard_pos = pattern_str.find('*').or_else(|| pattern_str.find('?'));

    if let Some(pos) = wildcard_pos {
        // Extract base path (everything before the wildcard)
        let base_path_str = &pattern_str[..pos];
        let base_path = PathBuf::from(base_path_str.trim_end_matches('/'));

        // Try to canonicalize the base path
        if let Ok(canonical_base) = base_path.canonicalize() {
            // Reconstruct the pattern with the canonical base
            let wildcard_part = &pattern_str[pos..];
            let canonical_pattern = format!("{}/{}", canonical_base.display(), wildcard_part.trim_start_matches('/'));
            return Some(PathBuf::from(canonical_pattern));
        }
    } else {
        // No wildcards, try to canonicalize the entire path
        return pattern.canonicalize().ok();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_whitelist() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "whitelist /tmp/test").unwrap();

        let mut parser = ProfileParser::new();
        let policy = parser.parse_file(file.path()).unwrap();

        assert_eq!(policy.filesystem_rules.len(), 1);
        assert!(matches!(
            &policy.filesystem_rules[0],
            FilesystemRule::Whitelist { .. }
        ));
    }

    #[test]
    fn test_parse_blacklist() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "blacklist /etc/shadow").unwrap();

        let mut parser = ProfileParser::new();
        let policy = parser.parse_file(file.path()).unwrap();

        assert_eq!(policy.filesystem_rules.len(), 1);
        assert!(matches!(
            &policy.filesystem_rules[0],
            FilesystemRule::Blacklist { .. }
        ));
    }

    #[test]
    fn test_parse_command_blacklist() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "blacklist /usr/bin/curl").unwrap();

        let mut parser = ProfileParser::new();
        let policy = parser.parse_file(file.path()).unwrap();

        // Should create a command rule instead of filesystem rule
        assert_eq!(policy.command_rules.len(), 1);
        assert_eq!(policy.command_rules[0].pattern, "/usr/bin/curl");
        assert_eq!(policy.command_rules[0].action, RuleAction::Deny);
    }

    #[test]
    fn test_parse_whitelist_exec() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "whitelist_exec /tmp/bin/*").unwrap();

        let mut parser = ProfileParser::new();
        let policy = parser.parse_file(file.path()).unwrap();

        // Should create at least one command rule (may create 2 if canonicalization differs)
        assert!(!policy.command_rules.is_empty());

        // First rule should be the original pattern
        assert_eq!(policy.command_rules[0].pattern, "/tmp/bin/*");
        assert_eq!(policy.command_rules[0].action, RuleAction::Allow);
    }

    #[test]
    fn test_parse_comments() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "whitelist /tmp # inline comment").unwrap();

        let mut parser = ProfileParser::new();
        let policy = parser.parse_file(file.path()).unwrap();

        assert_eq!(policy.filesystem_rules.len(), 1);
    }

    #[test]
    fn test_macro_expansion() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "whitelist ${{HOME}}/test").unwrap();

        let mut parser = ProfileParser::new();
        let policy = parser.parse_file(file.path()).unwrap();

        if let FilesystemRule::Whitelist { path, .. } = &policy.filesystem_rules[0] {
            // Should contain expanded HOME path
            assert!(!path.to_string_lossy().contains("${HOME}"));
        } else {
            panic!("Expected whitelist rule");
        }
    }

    #[test]
    fn test_noblacklist() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "blacklist /tmp/test").unwrap();
        writeln!(file, "noblacklist /tmp/test").unwrap();

        let mut parser = ProfileParser::new();
        let policy = parser.parse_file(file.path()).unwrap();

        // Blacklist should be removed
        assert_eq!(policy.filesystem_rules.len(), 0);
    }

    #[test]
    fn test_whitelist_exec_canonicalization() {
        // Create /tmp/bin if it doesn't exist
        let _ = std::fs::create_dir_all("/tmp/bin");

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "whitelist_exec /tmp/bin/*").unwrap();

        let mut parser = ProfileParser::new();
        let policy = parser.parse_file(file.path()).unwrap();

        // Should create at least one command rule
        assert!(!policy.command_rules.is_empty());

        // Check if both patterns exist (original and canonical)
        let patterns: Vec<&str> = policy.command_rules.iter().map(|r| r.pattern.as_str()).collect();
        assert!(patterns.contains(&"/tmp/bin/*"));

        // On macOS, /tmp is a symlink to /private/tmp, so we should also have the canonical version
        #[cfg(target_os = "macos")]
        {
            if patterns.len() > 1 {
                assert!(patterns.iter().any(|p| p.contains("/private/tmp/bin/")));
            }
        }
    }
}
