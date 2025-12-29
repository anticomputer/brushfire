//! Wrapper for `cp` utility with policy enforcement.

use brushfire_wrappers::{exec_real_utility, get_policy_engine, parse_args, print_policy_error};
use brushfire_wrappers::schemas::CP_SCHEMA;
use std::path::Path;

fn main() {
    // Load policy from environment
    let policy = match get_policy_engine() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("brushfire: Failed to load policy: {}", e);
            std::process::exit(2);
        }
    };

    // Parse arguments to extract file paths
    let args: Vec<String> = std::env::args().skip(1).collect();
    let file_args = parse_args(&CP_SCHEMA, &args);

    // Check policy for each file
    for file_arg in file_args {
        let path = Path::new(&file_arg.path);
        if let Err(e) = policy.check_file_access(path, file_arg.access_mode) {
            print_policy_error(CP_SCHEMA.name, path, &e);
            std::process::exit(1);
        }
    }

    // All checks passed - exec real utility
    exec_real_utility(&CP_SCHEMA, &args);
}
