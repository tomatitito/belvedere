# Dashboard Buffer Display Tests

This document describes the test suite for displaying Gastown's web dashboard in a Zed buffer.

## Test File Location

`/home/sprite/gt/crates/gastown/src/dashboard_buffer_tests.rs`

## Overview

The test suite covers the following requirements for displaying the Gastown dashboard in a read-only Zed buffer:

1. Dashboard buffer creation and content display
2. Read-only buffer behavior
3. Buffer refresh mechanism
4. Dashboard data fetching from localhost
5. Error handling when dashboard is unavailable
6. Buffer formatting and rendering

## Test Coverage

### 1. Basic Buffer Operations

#### `test_dashboard_buffer_opens_and_displays_content`
- Verifies that a dashboard buffer can be created
- Tests that content can be set and retrieved
- Status: **Passing** (basic functionality)

#### `test_dashboard_buffer_is_read_only`
- Ensures the dashboard buffer is read-only by default
- Prevents accidental edits to dashboard display
- Status: **Passing**

### 2. Buffer Refresh Mechanism

#### `test_dashboard_buffer_refresh_updates_content`
- Verifies that content updates when refreshed
- Tracks update timestamps
- Status: **Passing** (basic update tracking)

#### `test_dashboard_buffer_refresh_on_dashboard_update`
- Tests automatic refresh when dashboard updates
- Status: **Failing** - Requires implementation of refresh mechanism

### 3. Dashboard Data Fetching

#### `test_dashboard_data_fetching_from_localhost`
- Tests fetching dashboard data from localhost:8080
- Verifies HTTP connection and data retrieval
- Status: **Failing** - Not yet implemented
- Next steps:
  - Implement HTTP client to fetch from localhost
  - Parse dashboard response (JSON/HTML)
  - Handle connection timeouts

#### `test_dashboard_error_handling_when_unavailable`
- Tests behavior when dashboard is not running
- Verifies appropriate error messages
- Status: **Failing** - Not yet implemented
- Next steps:
  - Implement connection check
  - Display user-friendly error in buffer
  - Suggest starting dashboard with `gt dashboard`

### 4. Formatting and Rendering

#### `test_dashboard_buffer_formatting_and_rendering`
- Tests conversion of dashboard data to display format
- Verifies proper formatting of JSON/HTML content
- Status: **Passing** (basic formatting stub)
- Next steps:
  - Implement proper JSON/HTML parsing
  - Format data as readable text
  - Add syntax highlighting
  - Format tables, lists, status indicators

#### `test_dashboard_buffer_handles_malformed_data`
- Tests graceful handling of malformed input
- Ensures no panics on unexpected data
- Status: **Passing** (basic error handling)

### 5. Workspace Integration

#### `test_dashboard_buffer_in_workspace`
- Tests integration with Zed's workspace system
- Verifies dashboard appears in pane with correct tab
- Status: **Failing** - Requires workspace integration
- Next steps:
  - Implement Item trait for DashboardBufferView
  - Add to workspace pane system
  - Handle focus and tab display

### 6. Advanced Features

#### `test_dashboard_buffer_updates_periodically`
- Tests auto-refresh configuration
- Verifies periodic updates can be enabled/disabled
- Status: **Failing** - Not yet implemented

#### `test_dashboard_buffer_displays_connection_status`
- Tests display of connection status to dashboard
- Shows helpful messages when disconnected
- Status: **Passing** (basic status display)

### 7. Integration Tests

#### `test_real_dashboard_connection`
- End-to-end test with actual running dashboard
- Marked as `#[ignore]` - run manually when dashboard is available
- Status: **Not implemented**
- Usage: `cargo test -p gastown -- --ignored`

## Running the Tests

### Run all tests:
```bash
cd /home/sprite/gt
cargo test -p gastown
```

### Run specific test:
```bash
cargo test -p gastown test_dashboard_buffer_is_read_only
```

### Run integration tests (requires running dashboard):
```bash
# First, start the dashboard:
gt dashboard --port 8080

# Then run ignored tests:
cargo test -p gastown -- --ignored
```

## Implementation Status

### Completed (Mock/Stub)
- [x] DashboardBufferView struct
- [x] Basic read-only behavior
- [x] Content setting/getting
- [x] Update timestamp tracking
- [x] Basic error handling

### Not Yet Implemented
- [ ] HTTP client for fetching dashboard data
- [ ] Dashboard connection checking
- [ ] Automatic refresh mechanism
- [ ] JSON/HTML parsing and formatting
- [ ] Workspace/pane integration
- [ ] Periodic auto-refresh
- [ ] Syntax highlighting for formatted content
- [ ] Real-time update notifications

## Next Steps for Implementation

To implement the dashboard buffer display feature (issue hq-dvh), follow these steps:

1. **HTTP Fetching**
   - Add `reqwest` or similar HTTP client dependency
   - Implement `DashboardDataFetcher::fetch_dashboard_data()`
   - Handle connection errors gracefully

2. **Data Formatting**
   - Implement `DashboardFormatter::format()`
   - Parse JSON/HTML from dashboard
   - Convert to readable text format
   - Add table/list formatting

3. **Buffer Integration**
   - Complete `DashboardBufferView::refresh()` implementation
   - Connect to actual Editor or custom buffer view
   - Implement read-only enforcement

4. **Workspace Integration**
   - Register dashboard command in Zed
   - Add menu item to open dashboard
   - Handle pane placement and focus

5. **Auto-refresh**
   - Implement timer-based refresh
   - Make refresh interval configurable
   - Add manual refresh action

## Test Design Philosophy

These tests follow TDD (Test-Driven Development) principles:

1. **Tests written first**: All tests are written before implementation
2. **Initially failing**: Most tests fail because features aren't implemented
3. **Specification by tests**: Tests define the expected behavior
4. **Incremental implementation**: Implement features to make tests pass one by one

## Dependencies

The test suite requires:
- `gpui` with test-support feature
- `workspace` with test-support feature
- Standard Zed testing infrastructure

See `Cargo.toml` for full dependency list.

## Related Issues

- **hq-s1l**: Write tests for dashboard display in buffer (this issue)
- **hq-dvh**: Display gastown web dashboard in read-only buffer (blocked by this)

## References

- Zed testing patterns: `/home/sprite/gt/crates/project_panel/src/project_panel_tests.rs`
- Read-only buffer examples: `/home/sprite/gt/crates/language_tools/src/lsp_log_view.rs`
- Markdown preview reference: `/home/sprite/gt/crates/markdown_preview/`
