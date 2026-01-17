# Rig Git Diff Tests Summary

## Overview
Created comprehensive test suite for git diff viewing in rig directories.
File: `/home/sprite/gt/crates/git_ui/src/rig_git_diff_tests.rs`

## Test Coverage

### 1. test_git_panel_shows_rig_changes
Tests that the git panel correctly displays changes (modified and added files) in rig directories.

**Key assertions:**
- Git panel shows modified files from rig
- Git panel shows added files from rig
- Status codes are correctly tracked

### 2. test_diff_view_for_rig_files
Tests that the diff view displays correctly for modified files within rigs.

**Key assertions:**
- FileDiffView opens successfully for rig files
- Diff content is rendered correctly
- Line-by-line changes are displayed

### 3. test_diff_refresh_on_agent_changes
Tests that the diff view automatically refreshes when an agent makes changes to rig files.

**Key assertions:**
- Initial diff state is correct
- Agent modifications are detected
- Diff updates after debounce period
- New changes appear in diff view

### 4. test_multi_rig_git_status
Tests handling of multiple rigs (multiple git repositories) simultaneously.

**Key assertions:**
- Multiple worktrees with separate .git directories
- Each rig's changes are tracked independently
- Git panel shows status from multiple repos

### 5. test_commit_in_rig
Tests that commit operations work correctly in rig directories.

**Key assertions:**
- New files appear as "added" status
- Git status is tracked correctly pre-commit
- Rig repository is recognized

### 6. test_revert_in_rig
Tests that revert operations work correctly in rig directories.

**Key assertions:**
- Modified files are detected in rig
- Pre-revert state is correct
- Status tracking works for revert scenarios

### 7. test_staging_in_rig
Tests that staging changes works correctly in rig directories.

**Key assertions:**
- Multiple files show as unstaged initially
- StageStatus is tracked correctly
- Staging state is maintained for rig files

### 8. test_rig_repository_identification
Tests that the git panel correctly identifies and displays rig repository information.

**Key assertions:**
- Rig is recognized as a git repository
- Repository selector shows the rig
- Changes are tracked in the rig context

### 9. test_binary_file_diff_in_rig
Tests that binary files in rigs are handled gracefully by the diff view.

**Key assertions:**
- Binary files don't cause crashes
- Diff view handles non-text files
- Graceful error handling or appropriate messaging

### 10. test_rig_with_many_changes
Tests performance and correctness with a large number of file changes in a rig.

**Key assertions:**
- Git panel handles 100+ changed files
- No performance degradation
- All changes are tracked

## Integration Points

These tests verify integration with Zed's existing systems:
- `FileDiffView` - for displaying file diffs
- `GitPanel` - for showing git status
- `FakeFs` - for simulating file system
- Editor test utilities - for verifying diff rendering

## Expected Behavior

All tests should **initially fail** because:
1. Rig-specific git operations are not yet implemented
2. Multi-repository support may not be complete
3. Repository selector may not handle rig directories

The tests serve as specifications for the implementation in task hq-oy2.

## Running the Tests

```bash
# Run all rig git diff tests
cargo test --package git_ui --lib rig_git_diff_tests

# Run a specific test
cargo test --package git_ui --lib test_git_panel_shows_rig_changes
```

## Notes

- Tests use FakeFs to simulate rig directory structures
- Rig paths follow the pattern: `/rigs/{rig_name}/`
- Each rig is a separate git repository (has its own `.git` directory)
- Tests follow Zed's existing test patterns from git_panel.rs and file_diff_view.rs
