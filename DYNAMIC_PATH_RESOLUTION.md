# Dynamic Path Resolution for Wrappers

## Problem

Originally, wrapper schemas used hardcoded paths like `/bin/cat`, `/usr/bin/grep`, etc. This had several issues:

1. **Non-portable**: Paths differ across systems (macOS, Linux, BSD)
2. **Ignores user environment**: Doesn't respect custom installations (Homebrew, custom builds)
3. **Wrong version**: Might call a different version than what was in the user's PATH

## Solution

We now **dynamically resolve** real utility paths from the original PATH before modifying it. This ensures wrappers call the exact version that was originally available.

## How It Works

### 1. Path Resolution (Before PATH Modification)

In `brush-shell/src/wrappers.rs::resolve_real_utility_paths()`:

```rust
// Before modifying PATH, search the original PATH for each utility
for util_name in WRAPPED_UTILITIES {
    for dir in path_var.split(':') {
        let candidate = Path::new(dir).join(util_name);
        if candidate.exists() && is_executable(&candidate) {
            // Found it! Store the canonical path
            resolved_paths.push((util_name, canonical_path));
            break;
        }
    }
}
```

### 2. Environment Variable Storage

For each resolved utility, we set an environment variable:

```bash
BRUSHFIRE_CAT_PATH=/bin/cat
BRUSHFIRE_AWK_PATH=/opt/homebrew/Cellar/awk/20250116/bin/awk
BRUSHFIRE_GREP_PATH=/usr/bin/grep
# ... etc for all 30 utilities
```

These are set in **both**:
- Process environment (via `env::set_var`)
- Shell environment (via `shell.env.update_or_add` with export)

### 3. Wrapper Lookup

In `brushfire-wrappers/src/schemas.rs::UtilitySchema::get_real_path()`:

```rust
pub fn get_real_path(&self) -> String {
    let env_var_name = format!("BRUSHFIRE_{}_PATH", self.name.to_uppercase());
    // Check environment variable first, fall back to hardcoded path
    std::env::var(&env_var_name).unwrap_or_else(|_| self.real_path.to_string())
}
```

### 4. Execution

In `brushfire-wrappers/src/executor.rs::exec_real_utility()`:

```rust
// Get the real path (from env var or fallback)
let real_path = schema.get_real_path();

// Execute the correct version
Command::new(&real_path).args(args).exec();
```

## Example

### User's Environment
```bash
# User has Homebrew awk installed
$ which awk
/opt/homebrew/bin/awk
```

### Without Dynamic Resolution (Old Behavior)
```rust
// Hardcoded in schema
real_path: "/usr/bin/awk"

// Would call system awk, ignoring Homebrew version!
```

### With Dynamic Resolution (New Behavior)
```bash
# Step 1: Resolve before PATH modification
BRUSHFIRE_AWK_PATH=/opt/homebrew/Cellar/awk/20250116/bin/awk

# Step 2: Wrapper reads env var
get_real_path() -> "/opt/homebrew/Cellar/awk/20250116/bin/awk"

# Step 3: Execute the correct version
exec("/opt/homebrew/Cellar/awk/20250116/bin/awk", args)
```

## Benefits

1. **Portable**: Works on any system with standard utilities
2. **Respects user environment**: Uses the version the user selected via PATH
3. **Consistent behavior**: Wrappers behave exactly like running the utility directly
4. **Fallback support**: Still has hardcoded paths if resolution fails

## Testing

Verify dynamic resolution is working:

```bash
# Check environment variables are set
cargo run -p brush-shell --features policy -- \
  --profile test-profiles/wrapper-simple-test.profile \
  --wrap-coreutils \
  -c 'env | grep BRUSHFIRE'

# Should show resolved paths from your system:
# BRUSHFIRE_AWK_PATH=/opt/homebrew/Cellar/awk/20250116/bin/awk
# BRUSHFIRE_CAT_PATH=/bin/cat
# ... etc
```

## Implementation Files

- `brush-shell/src/wrappers.rs`: Path resolution and environment variable setup
- `brushfire-wrappers/src/schemas.rs`: `UtilitySchema::get_real_path()` method
- `brushfire-wrappers/src/executor.rs`: Uses dynamic path for exec()
