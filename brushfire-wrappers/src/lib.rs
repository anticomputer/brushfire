//! Policy-aware wrappers for POSIX utilities.
//!
//! This crate provides thin wrapper binaries for common POSIX utilities that check
//! brushfire policies before executing the real system utilities. The wrappers:
//!
//! 1. Load the policy from the `BRUSHFIRE_POLICY` environment variable
//! 2. Parse command line arguments to identify file paths
//! 3. Check each file path against the policy
//! 4. If all checks pass, exec() the real utility with the original arguments
//! 5. If any check fails, print an error message and exit
//!
//! ## Use Case
//!
//! These wrappers are designed for AI agent accident prevention, not adversarial security.
//! They help prevent accidental file operations outside of allowed directories.
//!
//! ## Architecture
//!
//! - `schemas`: Defines argument patterns for each utility
//! - `policy_loader`: Loads and caches the policy from environment
//! - `arg_parser`: Extracts file paths from command line arguments
//! - `executor`: Replaces current process with real utility using exec()

pub mod arg_parser;
pub mod executor;
pub mod policy_loader;
pub mod schemas;

// Re-export commonly used items
pub use arg_parser::parse_args;
pub use executor::exec_real_utility;
pub use policy_loader::get_policy_engine;
pub use schemas::{FileArg, UtilitySchema};

use brushfire_policy::PolicyViolation;
use std::path::Path;

/// Print a policy error message to stderr.
///
/// This function prints a clear, helpful error message for AI agents when a
/// policy violation occurs.
///
/// # Arguments
///
/// * `command` - The command name (e.g., "cat")
/// * `file_path` - The file path that triggered the violation
/// * `violation` - The policy violation details
pub fn print_policy_error(command: &str, file_path: &Path, violation: &PolicyViolation) {
    eprintln!("brushfire: Policy violation");
    eprintln!("  Command: {} {}", command, file_path.display());
    eprintln!("  Reason: {}", violation);
    eprintln!();
    eprintln!("This operation was blocked by brushfire policy enforcement.");
    eprintln!("Check the active profile for details.");
}
