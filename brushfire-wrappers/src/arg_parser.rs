//! Argument parsing for utility wrappers.
//!
//! This module parses command line arguments according to utility schemas
//! to identify which arguments are file paths that need policy checks.

use crate::schemas::{ArgPattern, FileArg, UtilitySchema};

/// Parse command line arguments according to a utility schema.
///
/// Extracts file paths from the arguments based on the utility's argument pattern.
/// Flags (arguments starting with `-`) are ignored unless they are positional
/// arguments (appearing after `--` delimiter).
///
/// # Arguments
///
/// * `schema` - The utility schema describing how to parse arguments
/// * `args` - The command line arguments (excluding program name)
///
/// # Returns
///
/// A vector of FileArg containing the paths and their access modes
pub fn parse_args(schema: &UtilitySchema, args: &[String]) -> Vec<FileArg> {
    // Separate positional args from flags
    let positional_args = extract_positional_args(args);

    // Apply schema pattern to extract file arguments
    match &schema.arg_pattern {
        ArgPattern::AllPositionalFiles { access_mode } => {
            // All positional args are files
            positional_args
                .into_iter()
                .map(|path| FileArg::new(path, *access_mode))
                .collect()
        }

        ArgPattern::SkipThenFiles {
            skip_count,
            access_mode,
        } => {
            // Skip first N args, rest are files
            positional_args
                .into_iter()
                .skip(*skip_count)
                .map(|path| FileArg::new(path, *access_mode))
                .collect()
        }

        ArgPattern::SourceDest {
            source_mode,
            dest_mode,
        } => {
            // All but last are sources, last is destination
            if positional_args.is_empty() {
                return Vec::new();
            }

            let mut file_args = Vec::new();

            // Sources (all but last)
            for path in &positional_args[..positional_args.len() - 1] {
                file_args.push(FileArg::new(path.clone(), *source_mode));
            }

            // Destination (last)
            if let Some(dest) = positional_args.last() {
                file_args.push(FileArg::new(dest.clone(), *dest_mode));
            }

            file_args
        }
    }
}

/// Extract positional arguments from command line.
///
/// Filters out flags (arguments starting with `-`) unless they appear after
/// the `--` delimiter or are the special `-` stdin indicator.
///
/// # Arguments
///
/// * `args` - The command line arguments
///
/// # Returns
///
/// A vector of positional argument strings
fn extract_positional_args(args: &[String]) -> Vec<String> {
    let mut positional = Vec::new();
    let mut after_delimiter = false;

    for arg in args {
        if after_delimiter {
            // After --, everything is positional
            positional.push(arg.clone());
        } else if arg == "--" {
            // -- delimiter: everything after is positional
            after_delimiter = true;
        } else if arg == "-" {
            // - is a positional arg (stdin indicator)
            positional.push(arg.clone());
        } else if !arg.starts_with('-') {
            // Not a flag, it's positional
            positional.push(arg.clone());
        }
        // else: it's a flag, skip it
    }

    positional
}

#[cfg(test)]
mod tests {
    use super::*;
    use brushfire_policy::FileAccessMode;

    #[test]
    fn test_extract_positional_simple() {
        let args = vec!["file1.txt".to_string(), "file2.txt".to_string()];
        let result = extract_positional_args(&args);
        assert_eq!(result, vec!["file1.txt", "file2.txt"]);
    }

    #[test]
    fn test_extract_positional_with_flags() {
        let args = vec![
            "-n".to_string(),
            "file1.txt".to_string(),
            "--verbose".to_string(),
            "file2.txt".to_string(),
        ];
        let result = extract_positional_args(&args);
        assert_eq!(result, vec!["file1.txt", "file2.txt"]);
    }

    #[test]
    fn test_extract_positional_after_delimiter() {
        let args = vec![
            "file1.txt".to_string(),
            "--".to_string(),
            "-file2.txt".to_string(),
        ];
        let result = extract_positional_args(&args);
        assert_eq!(result, vec!["file1.txt", "-file2.txt"]);
    }

    #[test]
    fn test_extract_positional_stdin() {
        let args = vec!["-n".to_string(), "-".to_string()];
        let result = extract_positional_args(&args);
        assert_eq!(result, vec!["-"]);
    }

    #[test]
    fn test_parse_all_positional() {
        let schema = UtilitySchema {
            name: "cat",
            real_path: "/bin/cat",
            arg_pattern: ArgPattern::AllPositionalFiles {
                access_mode: FileAccessMode::Read,
            },
        };

        let args = vec!["file1.txt".to_string(), "file2.txt".to_string()];
        let result = parse_args(&schema, &args);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].path, "file1.txt");
        assert_eq!(result[1].path, "file2.txt");
    }

    #[test]
    fn test_parse_skip_then_files() {
        let schema = UtilitySchema {
            name: "grep",
            real_path: "/usr/bin/grep",
            arg_pattern: ArgPattern::SkipThenFiles {
                skip_count: 1,
                access_mode: FileAccessMode::Read,
            },
        };

        let args = vec![
            "pattern".to_string(),
            "file1.txt".to_string(),
            "file2.txt".to_string(),
        ];
        let result = parse_args(&schema, &args);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].path, "file1.txt");
        assert_eq!(result[1].path, "file2.txt");
    }

    #[test]
    fn test_parse_source_dest() {
        let schema = UtilitySchema {
            name: "cp",
            real_path: "/bin/cp",
            arg_pattern: ArgPattern::SourceDest {
                source_mode: FileAccessMode::Read,
                dest_mode: FileAccessMode::Write,
            },
        };

        let args = vec!["src.txt".to_string(), "dest.txt".to_string()];
        let result = parse_args(&schema, &args);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].path, "src.txt");
        assert_eq!(result[1].path, "dest.txt");
    }

    #[test]
    fn test_parse_source_dest_multiple_sources() {
        let schema = UtilitySchema {
            name: "cp",
            real_path: "/bin/cp",
            arg_pattern: ArgPattern::SourceDest {
                source_mode: FileAccessMode::Read,
                dest_mode: FileAccessMode::Write,
            },
        };

        let args = vec![
            "src1.txt".to_string(),
            "src2.txt".to_string(),
            "dest/".to_string(),
        ];
        let result = parse_args(&schema, &args);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].path, "src1.txt");
        assert_eq!(result[1].path, "src2.txt");
        assert_eq!(result[2].path, "dest/");
    }
}
