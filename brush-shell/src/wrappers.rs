//! Wrapper utilities setup for `--wrap-coreutils` feature.
//!
//! This module handles extracting wrapper binaries to a temporary directory,
//! setting up the environment, and auto-blacklisting real utility paths.

use brush_core::env::{EnvironmentLookup, EnvironmentScope};
use brush_core::variables::ShellValueLiteral;
use brush_core::Shell;
use brushfire_policy::PolicyEngine;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// List of utilities that have wrappers
const WRAPPED_UTILITIES: &[&str] = &[
    // Original 10
    "cat", "rm", "cp", "mv", "ls", "grep", "head", "tail", "touch", "mkdir",
    // File operations
    "ln", "chmod", "chown", "chgrp", "rmdir", "dd", "file", "stat",
    // Text processing
    "sed", "awk", "cut", "paste", "sort", "uniq", "tr", "wc", "tee", "diff",
    // Archive/search
    "tar", "find",
];

/// Cleanup handler for wrapper setup.
///
/// When dropped, restores the original PATH and cleans up the temporary directory.
pub struct WrapperCleanup {
    pub(crate) _temp_dir: TempDir,
    pub(crate) original_path: String,
}

impl Drop for WrapperCleanup {
    fn drop(&mut self) {
        // Restore original PATH
        // SAFETY: We're restoring the PATH environment variable.
        // This is safe as we're single-threaded during cleanup.
        unsafe {
            env::set_var("PATH", &self.original_path);
        }
        // temp_dir will be cleaned up automatically
    }
}

/// Auto-blacklist real utility paths in the policy engine.
///
/// This should be called BEFORE wrapping the PolicyEngine in an Arc.
///
/// # Arguments
///
/// * `policy` - Mutable reference to the policy engine
///
/// # Errors
///
/// Returns an error if utility paths cannot be resolved
pub fn auto_blacklist_utilities(policy: &mut PolicyEngine) -> Result<(), std::io::Error> {
    // We don't have the wrapper directory yet, so we'll search without excluding it
    // The blacklisting will prevent bypass via absolute paths
    auto_blacklist_real_utilities(policy, WRAPPED_UTILITIES, Path::new(""))?;
    Ok(())
}

/// Auto-whitelist wrapper directory when in default-deny mode.
///
/// When `whitelist_exec` rules are in use (default-deny for commands), wrapped
/// coreutils need to be explicitly allowed. This function adds the wrapper
/// directory to the allowed command patterns and whitelists it for file access.
///
/// **SECURITY WARNING**: This prints a warning to stderr because allowing wrapped
/// coreutils in strict sandboxes may enable policy bypasses or escalation attacks.
///
/// This should be called AFTER the wrappers are set up and we know the wrapper directory.
///
/// # Arguments
///
/// * `policy` - Mutable reference to the policy engine
/// * `wrapper_dir` - Path to the wrapper directory
///
/// # Errors
///
/// Returns an error if the wrapper directory cannot be canonicalized
pub fn auto_whitelist_wrappers(
    policy: &mut PolicyEngine,
    wrapper_dir: &Path,
) -> Result<(), std::io::Error> {
    // Only do this if we're in default-deny mode for commands
    if !policy.is_command_default_deny() {
        return Ok(());
    }

    // Print security warning to stderr
    eprintln!("\n[WARNING] --wrap-coreutils enabled with whitelist_exec rules:");
    eprintln!("[WARNING] Wrapped coreutils will be automatically allowed.");
    eprintln!("[WARNING] This may enable policy bypasses or privilege escalation.");
    eprintln!("[WARNING] In strict sandboxes, consider using explicit command allow rules.\n");

    // Canonicalize the wrapper directory
    let canonical_wrapper = wrapper_dir.canonicalize()?;

    // Add allow rule for all executables in the wrapper directory
    let pattern = format!("{}/*", canonical_wrapper.display());
    policy.add_command_allow_rule(pattern.clone());

    // Also add non-canonical version if different (for symlink handling)
    let non_canonical_pattern = format!("{}/*", wrapper_dir.display());
    if non_canonical_pattern != pattern {
        policy.add_command_allow_rule(non_canonical_pattern);
    }

    // Also whitelist the wrapper directory for filesystem access
    // (needed so the wrappers can be executed)
    policy.add_filesystem_rule(brushfire_policy::rules::FilesystemRule::Whitelist {
        path: canonical_wrapper,
        recursive: true,
    });

    Ok(())
}

/// Resolve real utility paths from the current PATH before modification.
///
/// This ensures we call the version of utilities that were originally in PATH,
/// respecting the user's environment (e.g., Homebrew on macOS).
///
/// # Arguments
///
/// * `shell` - Reference to the shell to read PATH from
///
/// # Returns
///
/// A map of utility names to their resolved paths
fn resolve_real_utility_paths(shell: &Shell) -> Result<Vec<(String, PathBuf)>, std::io::Error> {
    let mut resolved_paths = Vec::new();
    let path_var = shell.env_str("PATH").unwrap_or_default();

    for util_name in WRAPPED_UTILITIES {
        // Search PATH for the utility
        for dir in path_var.split(':') {
            let candidate = Path::new(dir).join(util_name);
            if candidate.exists() && is_executable(&candidate) {
                // Found it - use the canonical path
                match candidate.canonicalize() {
                    Ok(canonical) => {
                        resolved_paths.push((util_name.to_string(), canonical));
                        break;
                    }
                    Err(_) => {
                        // If canonicalization fails, use as-is
                        resolved_paths.push((util_name.to_string(), candidate));
                        break;
                    }
                }
            }
        }
    }

    Ok(resolved_paths)
}

/// Set up coreutils wrappers environment.
///
/// This function:
/// 1. Resolves real utility paths from current PATH
/// 2. Uses provided temp directory (or creates one if None)
/// 3. Copies wrapper binaries to the temp directory
/// 4. Sets BRUSHFIRE_POLICY and BRUSHFIRE_*_PATH environment variables
/// 5. Optionally sets BRUSHFIRE_WEBHOOK_URL and BRUSHFIRE_SESSION_ID for webhook reporting
/// 6. Prepends temp directory to PATH in both process and shell environments
///
/// Note: Auto-blacklisting should be done separately before creating the shell.
///
/// # Arguments
///
/// * `profile_path` - Path to the policy profile file
/// * `shell` - Mutable reference to the shell to update its PATH
/// * `webhook_url` - Optional webhook URL for policy event reporting
/// * `session_id` - Optional session ID for correlating webhook events
/// * `temp_dir` - Optional pre-created temp directory (for auto-whitelisting)
///
/// # Returns
///
/// A WrapperCleanup handle that will restore PATH when dropped
///
/// # Errors
///
/// Returns an error if:
/// - Temp directory cannot be created
/// - Wrapper binaries cannot be copied
pub fn setup_coreutils_wrappers(
    profile_path: &Path,
    shell: &mut Shell,
    webhook_url: Option<&String>,
    session_id: Option<&String>,
    temp_dir: Option<TempDir>,
) -> Result<WrapperCleanup, std::io::Error> {
    // 1. Resolve real utility paths BEFORE modifying PATH
    let real_paths = resolve_real_utility_paths(shell)?;

    // 2. Use provided temp directory or create new one
    let temp_dir = temp_dir.map_or_else(tempfile::tempdir, Ok)?;

    // 3. Copy wrapper binaries to temp directory
    copy_wrapper_binaries(temp_dir.path())?;

    // 4. Set BRUSHFIRE_POLICY environment variable
    // Set in both process environment and shell environment
    // SAFETY: Setting environment variable early in program execution.
    // This is safe as we control the shell initialization.
    unsafe {
        env::set_var("BRUSHFIRE_POLICY", profile_path);
    }

    shell
        .env
        .update_or_add(
            "BRUSHFIRE_POLICY",
            ShellValueLiteral::Scalar(profile_path.display().to_string()),
            |var| {
                var.export();
                Ok(())
            },
            EnvironmentLookup::Anywhere,
            EnvironmentScope::Global,
        )
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to set BRUSHFIRE_POLICY: {}", e),
            )
        })?;

    // Set BRUSHFIRE_WEBHOOK_URL and BRUSHFIRE_SESSION_ID if provided
    if let Some(url) = webhook_url {
        unsafe {
            env::set_var("BRUSHFIRE_WEBHOOK_URL", url);
        }
        shell
            .env
            .update_or_add(
                "BRUSHFIRE_WEBHOOK_URL",
                ShellValueLiteral::Scalar(url.clone()),
                |var| {
                    var.export();
                    Ok(())
                },
                EnvironmentLookup::Anywhere,
                EnvironmentScope::Global,
            )
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to set BRUSHFIRE_WEBHOOK_URL: {}", e),
                )
            })?;
    }

    if let Some(id) = session_id {
        unsafe {
            env::set_var("BRUSHFIRE_SESSION_ID", id);
        }
        shell
            .env
            .update_or_add(
                "BRUSHFIRE_SESSION_ID",
                ShellValueLiteral::Scalar(id.clone()),
                |var| {
                    var.export();
                    Ok(())
                },
                EnvironmentLookup::Anywhere,
                EnvironmentScope::Global,
            )
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to set BRUSHFIRE_SESSION_ID: {}", e),
                )
            })?;
    }

    // 5. Set BRUSHFIRE_*_PATH environment variables for each resolved utility
    // This allows wrappers to call the correct version of the real utility
    for (util_name, real_path) in &real_paths {
        let env_var_name = format!("BRUSHFIRE_{}_PATH", util_name.to_uppercase());
        let path_str = real_path.display().to_string();

        // Set in process environment
        unsafe {
            env::set_var(&env_var_name, &path_str);
        }

        // Set in shell environment and export
        shell
            .env
            .update_or_add(
                &env_var_name,
                ShellValueLiteral::Scalar(path_str),
                |var| {
                    var.export();
                    Ok(())
                },
                EnvironmentLookup::Anywhere,
                EnvironmentScope::Global,
            )
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to set {}: {}", env_var_name, e),
                )
            })?;
    }

    // 4. Prepend temp directory to PATH
    let original_path = shell.env_str("PATH").unwrap_or_default().to_string();
    let new_path = format!("{}:{}", temp_dir.path().display(), original_path);

    // Update shell's internal PATH and export it
    shell
        .env
        .update_or_add(
            "PATH",
            ShellValueLiteral::Scalar(new_path.clone()),
            |var| {
                var.export();
                Ok(())
            },
            EnvironmentLookup::Anywhere,
            EnvironmentScope::Global,
        )
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to set PATH: {}", e),
            )
        })?;

    // Also update process environment for consistency
    // SAFETY: Setting PATH environment variable early in program execution.
    unsafe {
        env::set_var("PATH", &new_path);
    }

    Ok(WrapperCleanup {
        _temp_dir: temp_dir,
        original_path,
    })
}

/// Copy wrapper binaries to the temporary directory.
///
/// For now, this copies from the build target directory. In production, these
/// would be embedded in the binary using include_bytes!.
fn copy_wrapper_binaries(dest_dir: &Path) -> Result<(), std::io::Error> {
    // TODO: In production, use embedded binaries via include_bytes!
    // For now, copy from the build directory for testing

    // Try to find the wrapper binaries in the target directory
    let current_exe = env::current_exe()?;
    let exe_dir = current_exe
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find exe dir"))?;

    for util_name in WRAPPED_UTILITIES {
        let src_path = exe_dir.join(util_name);

        // Only copy if the wrapper exists
        if src_path.exists() {
            let dest_path = dest_dir.join(util_name);
            fs::copy(&src_path, &dest_path)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&dest_path, fs::Permissions::from_mode(0o755))?;
            }
        }
    }

    Ok(())
}

/// Auto-blacklist real utility paths to prevent bypass via absolute paths.
///
/// This function:
/// - Searches standard directories and PATH for real utilities
/// - Canonicalizes paths to resolve symlinks
/// - Blacklists both the canonical path and any symlinks
///
/// # Arguments
///
/// * `policy` - Mutable reference to the policy engine
/// * `utilities` - List of utility names to blacklist
/// * `wrapper_dir` - Directory containing our wrappers (to skip)
fn auto_blacklist_real_utilities(
    policy: &mut PolicyEngine,
    utilities: &[&str],
    wrapper_dir: &Path,
) -> Result<(), std::io::Error> {
    for util_name in utilities {
        // Find all real utility paths (excluding our wrapper directory)
        let real_paths = find_all_real_utility_paths(util_name, wrapper_dir)?;

        for path in real_paths {
            // Try to canonicalize to resolve symlinks
            match path.canonicalize() {
                Ok(canonical_path) => {
                    // Add command deny rule for the canonical path
                    policy.add_command_deny_rule(canonical_path.display().to_string());

                    // If different, also block the symlink path
                    if path != canonical_path {
                        policy.add_command_deny_rule(path.display().to_string());
                    }
                }
                Err(_) => {
                    // If canonicalization fails, just block the path as-is
                    policy.add_command_deny_rule(path.display().to_string());
                }
            }
        }
    }

    Ok(())
}

/// Find all real utility paths for a given utility name.
///
/// Searches:
/// - Standard system directories (/bin, /usr/bin, /usr/local/bin)
/// - Current PATH (excluding wrapper directory)
///
/// # Arguments
///
/// * `util_name` - Name of the utility to find
/// * `wrapper_dir` - Directory to exclude from search
///
/// # Returns
///
/// A vector of paths where the utility was found
fn find_all_real_utility_paths(
    util_name: &str,
    wrapper_dir: &Path,
) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut paths = Vec::new();
    let wrapper_dir_canonical = wrapper_dir.canonicalize().ok();

    // Search standard locations
    let standard_dirs = ["/bin", "/usr/bin", "/usr/local/bin"];

    for dir in &standard_dirs {
        let candidate = Path::new(dir).join(util_name);
        if candidate.exists() && is_executable(&candidate) {
            // Skip if this is in our wrapper directory
            if let Some(ref wrapper_canon) = wrapper_dir_canonical {
                if let Some(candidate_parent) = candidate.parent()
                    .and_then(|p| p.canonicalize().ok())
                {
                    if candidate_parent == *wrapper_canon {
                        continue;
                    }
                }
            }
            paths.push(candidate);
        }
    }

    // Also search current PATH
    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':') {
            // Skip our wrapper directory
            if let Some(ref wrapper_canon) = wrapper_dir_canonical {
                if let Ok(dir_canon) = Path::new(dir).canonicalize() {
                    if dir_canon == *wrapper_canon {
                        continue;
                    }
                }
            }

            let candidate = Path::new(dir).join(util_name);
            if candidate.exists() && is_executable(&candidate) {
                // Avoid duplicates
                if !paths.contains(&candidate) {
                    paths.push(candidate);
                }
            }
        }
    }

    Ok(paths)
}

/// Check if a file is executable.
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = fs::metadata(path) {
        let permissions = metadata.permissions();
        permissions.mode() & 0o111 != 0
    } else {
        false
    }
}

/// Check if a file is executable on non-Unix platforms.
#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    // On Windows, we could check file extension, but for now just return true
    true
}
