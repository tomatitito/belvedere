# Dashboard Buffer Display Tests

This document describes the test suite for displaying Gastown's dashboard in a Zed buffer.

## Test File Location

`crates/gastown/src/dashboard_buffer_tests.rs`

## Architecture

The dashboard buffer uses a **trait-based data source** pattern:

```rust
pub trait DashboardDataSource: Send + Sync {
    fn fetch(&self) -> Result<DashboardData, DashboardError>;
    fn is_available(&self) -> bool;
}
```

This allows:
- **MockDataSource** for testing (no external dependencies)
- **DirectDataSource** (future) for pulling data from Gastown internals
- Easy swapping of implementations without changing the buffer code

## Test Coverage

### Data Models
- `DashboardData` - agents, convoys, rigs
- `AgentInfo` with status (Active, Idle, Error)
- `ConvoyInfo` with progress percentage
- `RigInfo` with name and path

### Buffer Tests

| Test | Description | Status |
|------|-------------|--------|
| `test_dashboard_buffer_displays_content` | Buffer shows agent/convoy/rig data | ✅ Passing |
| `test_dashboard_buffer_is_read_only` | Buffer is read-only | ✅ Passing |
| `test_dashboard_buffer_refresh_updates_timestamp` | Refresh updates timestamp | ✅ Passing |
| `test_dashboard_shows_error_when_unavailable` | Shows error when data unavailable | ✅ Passing |
| `test_dashboard_shows_connected_status` | Tracks connection status | ✅ Passing |
| `test_dashboard_tab_shows_correct_title` | Tab shows "Dashboard" | ✅ Passing |

### Formatter Tests

| Test | Description | Status |
|------|-------------|--------|
| `test_dashboard_formatter_handles_empty_data` | Empty data shows placeholders | ✅ Passing |
| `test_dashboard_formatter_shows_agent_status` | Status icons (●/○/✗) | ✅ Passing |
| `test_dashboard_formatter_shows_convoy_progress` | Progress bars and percentages | ✅ Passing |
| `test_dashboard_formatter_shows_rigs` | Rig name → path mapping | ✅ Passing |

### Data Source Tests

| Test | Description | Status |
|------|-------------|--------|
| `test_data_source_trait_with_mock` | Mock returns data correctly | ✅ Passing |
| `test_data_source_unavailable_returns_error` | Unavailable returns error | ✅ Passing |

## Running Tests

```bash
# Run all gastown tests
cargo test -p gastown

# Run specific test
cargo test -p gastown test_dashboard_buffer_displays_content
```

## Next Steps for Implementation

1. **Move types to production code** - Extract `DashboardBufferView`, `DashboardDataSource`, etc. from test file to `src/dashboard_buffer.rs`

2. **Implement DirectDataSource** - Create implementation that pulls data directly from Gastown's internal state

3. **Register in workspace** - Add command to open dashboard buffer in a pane

4. **Auto-refresh** - Add timer-based refresh mechanism

## Dependencies

- `gpui` with test-support feature
- `workspace` with test-support feature

## Related Issues

- **hq-s1l**: Write tests for dashboard display in buffer (this issue)
- **hq-dvh**: Display gastown dashboard in read-only buffer (blocked by this)
