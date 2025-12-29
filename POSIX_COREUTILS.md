# POSIX Coreutils Wrapper Implementation Status

## Currently Implemented (10)
- [x] cat - Read files
- [x] rm - Remove files
- [x] cp - Copy files
- [x] mv - Move files
- [x] ls - List directory
- [x] grep - Search files
- [x] head - Read file head
- [x] tail - Read file tail
- [x] touch - Create/update files
- [x] mkdir - Create directories

## High Priority - File Operations (8)
- [ ] ln - Create links (symlinks and hard links)
- [ ] chmod - Change file permissions
- [ ] chown - Change file owner
- [ ] chgrp - Change file group
- [ ] rmdir - Remove directories
- [ ] dd - Copy/convert files (low-level)
- [ ] file - Determine file type (reads files)
- [ ] stat - Display file status (reads metadata)

## High Priority - Text Processing (10)
- [ ] sed - Stream editor (reads/writes files)
- [ ] awk - Pattern scanning (reads files)
- [ ] cut - Cut out sections (reads files)
- [ ] paste - Merge lines (reads files)
- [ ] sort - Sort lines (reads/writes files)
- [ ] uniq - Report/filter repeated lines (reads files)
- [ ] tr - Translate characters (typically stdin/stdout but can read files)
- [ ] wc - Word/line/byte count (reads files)
- [ ] tee - Read stdin, write to file and stdout
- [ ] diff - Compare files

## High Priority - Archive/Search (2)
- [ ] tar - Archive utility (reads/writes files)
- [ ] find - Search filesystem (reads directories)

## Medium Priority - Additional Utilities (5)
- [ ] cmp - Compare files byte by byte
- [ ] comm - Compare sorted files line by line
- [ ] basename - Strip directory from pathname (path operation)
- [ ] dirname - Strip filename from pathname (path operation)
- [ ] readlink - Display symlink target

## Lower Priority - Process/System (not filesystem-focused)
- echo, printf, test, expr, env, etc. - These don't typically access files directly

## Total Wrappers to Implement
- Currently: 10
- High Priority: 20 additional
- **Target: 30 total wrappers**
