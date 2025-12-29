//! Execution of real system utilities.
//!
//! This module handles replacing the current wrapper process with the real
//! system utility using exec().

use crate::schemas::UtilitySchema;
use std::process::Command;

/// Execute the real utility, replacing the current process.
///
/// This function uses exec() to replace the current wrapper process with the
/// real system utility. If successful, this function never returns. If exec()
/// fails, the function exits with status code 127.
///
/// # Arguments
///
/// * `schema` - The utility schema containing the path to the real utility
/// * `args` - The command line arguments to pass to the utility
///
/// # Panics
///
/// This function never panics, but will exit the process if exec() fails.
#[cfg(unix)]
pub fn exec_real_utility(schema: &UtilitySchema, args: &[String]) -> ! {
    use std::os::unix::process::CommandExt;

    // Get the real path (from env var or fallback to hardcoded)
    let real_path = schema.get_real_path();

    // Build command for the real utility
    let error = Command::new(&real_path)
        .args(args)
        .exec(); // This replaces the current process and never returns

    // If we get here, exec failed
    eprintln!("brushfire: Failed to exec {}: {}", schema.name, error);
    std::process::exit(127);
}

/// Execute the real utility on non-Unix platforms.
///
/// On non-Unix platforms (like Windows), we can't use exec() to replace the
/// process, so we spawn the utility and wait for it to complete.
///
/// # Arguments
///
/// * `schema` - The utility schema containing the path to the real utility
/// * `args` - The command line arguments to pass to the utility
///
/// # Panics
///
/// This function never panics, but will exit the process with the utility's
/// exit code or 127 if spawning fails.
#[cfg(not(unix))]
pub fn exec_real_utility(schema: &UtilitySchema, args: &[String]) -> ! {
    // Get the real path (from env var or fallback to hardcoded)
    let real_path = schema.get_real_path();

    // On Windows, we can't use exec, so spawn and wait
    match Command::new(&real_path).args(args).status() {
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            std::process::exit(code);
        }
        Err(e) => {
            eprintln!("brushfire: Failed to execute {}: {}", schema.name, e);
            std::process::exit(127);
        }
    }
}
