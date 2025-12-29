# Strict whitelist profile - default deny everything
# Only allow access to /tmp and wrapper-managed commands

# Whitelist /tmp - this automatically enables DenyAll mode
# All other paths will be denied by default
whitelist /tmp

# Whitelist system temp directory for wrapper execution
# This is where --wrap-coreutils stores the wrapper binaries
whitelist /var/folders

# Note: When using --wrap-coreutils, the wrapper temp directory
# will be automatically added to PATH and original coreutils will
# be auto-blacklisted, providing command isolation.
