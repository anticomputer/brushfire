//! Argument schemas for POSIX utilities.
//!
//! This module defines how different POSIX utilities handle their arguments,
//! allowing us to identify which arguments are file paths that need policy checks.

use brushfire_policy::FileAccessMode;

/// Describes how a POSIX utility processes arguments
#[derive(Debug, Clone)]
pub struct UtilitySchema {
    /// Name of the utility (e.g., "cat", "cp")
    pub name: &'static str,

    /// Path to the real system utility (fallback if env var not set)
    pub real_path: &'static str,

    /// Argument pattern rules
    pub arg_pattern: ArgPattern,
}

impl UtilitySchema {
    /// Get the real path for this utility, checking environment variable first.
    ///
    /// This allows dynamic path resolution based on the original PATH before
    /// wrapper directory was prepended. Falls back to the hardcoded path if
    /// the environment variable is not set.
    ///
    /// # Returns
    ///
    /// The path to the real utility executable
    pub fn get_real_path(&self) -> String {
        let env_var_name = format!("BRUSHFIRE_{}_PATH", self.name.to_uppercase());
        std::env::var(&env_var_name).unwrap_or_else(|_| self.real_path.to_string())
    }
}

/// Argument pattern for a utility
#[derive(Debug, Clone)]
pub enum ArgPattern {
    /// All positional args are files (e.g., cat, rm, ls)
    AllPositionalFiles { access_mode: FileAccessMode },

    /// Skip first N positional args, rest are files (e.g., grep PATTERN [FILE]...)
    SkipThenFiles {
        skip_count: usize,
        access_mode: FileAccessMode,
    },

    /// Source files and destination (e.g., cp, mv)
    SourceDest {
        source_mode: FileAccessMode,
        dest_mode: FileAccessMode,
    },
}

/// A file argument extracted from command line
#[derive(Debug, Clone)]
pub struct FileArg {
    /// Path to the file
    pub path: String,

    /// Access mode required for this file
    pub access_mode: FileAccessMode,
}

impl FileArg {
    /// Create a new file argument
    pub fn new(path: String, access_mode: FileAccessMode) -> Self {
        Self { path, access_mode }
    }
}

// Schema definitions for each utility

/// Schema for `cat` utility - all args are read files
pub const CAT_SCHEMA: UtilitySchema = UtilitySchema {
    name: "cat",
    real_path: "/bin/cat",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `rm` utility - all args are files to delete
pub const RM_SCHEMA: UtilitySchema = UtilitySchema {
    name: "rm",
    real_path: "/bin/rm",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Write,
    },
};

/// Schema for `cp` utility - sources are read, dest is write
pub const CP_SCHEMA: UtilitySchema = UtilitySchema {
    name: "cp",
    real_path: "/bin/cp",
    arg_pattern: ArgPattern::SourceDest {
        source_mode: FileAccessMode::Read,
        dest_mode: FileAccessMode::Write,
    },
};

/// Schema for `mv` utility - sources are read, dest is write
pub const MV_SCHEMA: UtilitySchema = UtilitySchema {
    name: "mv",
    real_path: "/bin/mv",
    arg_pattern: ArgPattern::SourceDest {
        source_mode: FileAccessMode::Read,
        dest_mode: FileAccessMode::Write,
    },
};

/// Schema for `ls` utility - all args are read paths
pub const LS_SCHEMA: UtilitySchema = UtilitySchema {
    name: "ls",
    real_path: "/bin/ls",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `grep` utility - skip pattern arg, rest are read files
pub const GREP_SCHEMA: UtilitySchema = UtilitySchema {
    name: "grep",
    real_path: "/usr/bin/grep",
    arg_pattern: ArgPattern::SkipThenFiles {
        skip_count: 1,
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `head` utility - all args are read files
pub const HEAD_SCHEMA: UtilitySchema = UtilitySchema {
    name: "head",
    real_path: "/usr/bin/head",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `tail` utility - all args are read files
pub const TAIL_SCHEMA: UtilitySchema = UtilitySchema {
    name: "tail",
    real_path: "/usr/bin/tail",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `touch` utility - all args are files to create/modify
pub const TOUCH_SCHEMA: UtilitySchema = UtilitySchema {
    name: "touch",
    real_path: "/usr/bin/touch",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Write,
    },
};

/// Schema for `mkdir` utility - all args are directories to create
pub const MKDIR_SCHEMA: UtilitySchema = UtilitySchema {
    name: "mkdir",
    real_path: "/bin/mkdir",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Write,
    },
};

// File operations

/// Schema for `ln` utility - sources are read, dest is write
pub const LN_SCHEMA: UtilitySchema = UtilitySchema {
    name: "ln",
    real_path: "/bin/ln",
    arg_pattern: ArgPattern::SourceDest {
        source_mode: FileAccessMode::Read,
        dest_mode: FileAccessMode::Write,
    },
};

/// Schema for `chmod` utility - all args are files to modify
pub const CHMOD_SCHEMA: UtilitySchema = UtilitySchema {
    name: "chmod",
    real_path: "/bin/chmod",
    arg_pattern: ArgPattern::SkipThenFiles {
        skip_count: 1, // Skip mode argument
        access_mode: FileAccessMode::Write,
    },
};

/// Schema for `chown` utility - all args are files to modify
pub const CHOWN_SCHEMA: UtilitySchema = UtilitySchema {
    name: "chown",
    real_path: "/usr/sbin/chown",
    arg_pattern: ArgPattern::SkipThenFiles {
        skip_count: 1, // Skip owner[:group] argument
        access_mode: FileAccessMode::Write,
    },
};

/// Schema for `chgrp` utility - all args are files to modify
pub const CHGRP_SCHEMA: UtilitySchema = UtilitySchema {
    name: "chgrp",
    real_path: "/usr/bin/chgrp",
    arg_pattern: ArgPattern::SkipThenFiles {
        skip_count: 1, // Skip group argument
        access_mode: FileAccessMode::Write,
    },
};

/// Schema for `rmdir` utility - all args are directories to remove
pub const RMDIR_SCHEMA: UtilitySchema = UtilitySchema {
    name: "rmdir",
    real_path: "/bin/rmdir",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Write,
    },
};

/// Schema for `dd` utility - reads if= and writes of=
pub const DD_SCHEMA: UtilitySchema = UtilitySchema {
    name: "dd",
    real_path: "/bin/dd",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Write, // Conservative: assume write access needed
    },
};

/// Schema for `file` utility - all args are files to read
pub const FILE_SCHEMA: UtilitySchema = UtilitySchema {
    name: "file",
    real_path: "/usr/bin/file",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `stat` utility - all args are files to read
pub const STAT_SCHEMA: UtilitySchema = UtilitySchema {
    name: "stat",
    real_path: "/usr/bin/stat",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

// Text processing

/// Schema for `sed` utility - last arg is typically input file
pub const SED_SCHEMA: UtilitySchema = UtilitySchema {
    name: "sed",
    real_path: "/usr/bin/sed",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read, // Simplified: treat all positional as read
    },
};

/// Schema for `awk` utility - skip program arg, rest are files
pub const AWK_SCHEMA: UtilitySchema = UtilitySchema {
    name: "awk",
    real_path: "/usr/bin/awk",
    arg_pattern: ArgPattern::SkipThenFiles {
        skip_count: 1, // Skip program argument
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `cut` utility - all args are read files
pub const CUT_SCHEMA: UtilitySchema = UtilitySchema {
    name: "cut",
    real_path: "/usr/bin/cut",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `paste` utility - all args are read files
pub const PASTE_SCHEMA: UtilitySchema = UtilitySchema {
    name: "paste",
    real_path: "/usr/bin/paste",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `sort` utility - all args are read files
pub const SORT_SCHEMA: UtilitySchema = UtilitySchema {
    name: "sort",
    real_path: "/usr/bin/sort",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `uniq` utility - all args are read files
pub const UNIQ_SCHEMA: UtilitySchema = UtilitySchema {
    name: "uniq",
    real_path: "/usr/bin/uniq",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `tr` utility - typically uses stdin/stdout, but treat positional as read
pub const TR_SCHEMA: UtilitySchema = UtilitySchema {
    name: "tr",
    real_path: "/usr/bin/tr",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `wc` utility - all args are read files
pub const WC_SCHEMA: UtilitySchema = UtilitySchema {
    name: "wc",
    real_path: "/usr/bin/wc",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

/// Schema for `tee` utility - all args are write files
pub const TEE_SCHEMA: UtilitySchema = UtilitySchema {
    name: "tee",
    real_path: "/usr/bin/tee",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Write,
    },
};

/// Schema for `diff` utility - all args are read files
pub const DIFF_SCHEMA: UtilitySchema = UtilitySchema {
    name: "diff",
    real_path: "/usr/bin/diff",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};

// Archive/search

/// Schema for `tar` utility - complex, treat all positional as read/write
pub const TAR_SCHEMA: UtilitySchema = UtilitySchema {
    name: "tar",
    real_path: "/usr/bin/tar",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Write, // Conservative: assume write needed
    },
};

/// Schema for `find` utility - all args are paths to search
pub const FIND_SCHEMA: UtilitySchema = UtilitySchema {
    name: "find",
    real_path: "/usr/bin/find",
    arg_pattern: ArgPattern::AllPositionalFiles {
        access_mode: FileAccessMode::Read,
    },
};
