# Sandbox profile for untrusted script execution
# Demonstrates practical brushfire usage

# Block dangerous commands
blacklist /usr/bin/curl
blacklist /usr/bin/wget
blacklist /usr/bin/nc
blacklist /usr/bin/ssh
blacklist /usr/bin/scp
blacklist /bin/rm
blacklist /usr/bin/sudo
blacklist /usr/bin/su

# Prevent execution from temp directories
noexec /tmp
noexec ${HOME}/Downloads

# Block access to sensitive directories
blacklist ${HOME}/.ssh
blacklist ${HOME}/.gnupg

# Allow workspace access
# (In a real scenario, you'd add whitelist rules here)
