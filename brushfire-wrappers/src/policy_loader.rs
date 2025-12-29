//! Policy loading for wrapper binaries.
//!
//! This module handles loading the brushfire policy from the environment variable
//! and caching it for the wrapper's lifetime.

use brushfire_policy::{PolicyEngine, ProfileParser};
use once_cell::sync::OnceCell;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during policy loading
#[derive(Error, Debug)]
pub enum PolicyError {
    /// BRUSHFIRE_POLICY environment variable not set
    #[error("BRUSHFIRE_POLICY environment variable not set")]
    NoPolicyEnvVar,

    /// Failed to parse policy profile
    #[error("Failed to parse policy profile: {0}")]
    PolicyParse(#[from] brushfire_policy::ParseError),

    /// IO error reading profile
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Global policy engine instance, loaded once per wrapper invocation
static POLICY_ENGINE: OnceCell<PolicyEngine> = OnceCell::new();

/// Get the policy engine, loading it if necessary.
///
/// The policy is loaded from the file path specified in the BRUSHFIRE_POLICY
/// environment variable. It's cached using OnceLock so subsequent calls return
/// the same instance.
///
/// # Errors
///
/// Returns an error if:
/// - BRUSHFIRE_POLICY environment variable is not set
/// - The profile file cannot be read
/// - The profile file contains invalid syntax
pub fn get_policy_engine() -> Result<&'static PolicyEngine, PolicyError> {
    POLICY_ENGINE.get_or_try_init(|| {
        // Get profile path from environment
        let profile_path =
            std::env::var("BRUSHFIRE_POLICY").map_err(|_| PolicyError::NoPolicyEnvVar)?;

        // Parse the profile
        let mut parser = ProfileParser::new();
        let policy = parser.parse_file(Path::new(&profile_path))?;

        // Create and return the policy engine
        Ok(PolicyEngine::new(policy))
    })
}
