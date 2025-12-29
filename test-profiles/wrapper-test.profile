# Test profile for --wrap-coreutils feature
# This profile blocks access to sensitive files

blacklist /etc/shadow
blacklist /etc/passwd
blacklist ${HOME}/.ssh
