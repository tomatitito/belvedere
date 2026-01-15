# Gas Town UI

A native desktop application for managing [Gas Town](https://github.com/steveyegge/gastown) multi-agent development environments. Built with [GPUI](https://gpui.rs), the high-performance Rust UI framework from [Zed](https://zed.dev).

## Overview

Gas Town UI provides a visual interface for:

- **Rig Management** — View and manage project containers with their associated agents
- **Agent Coordination** — Monitor Polecats (worker agents) and Crew members
- **Convoy Tracking** — Track bundled work items across agents in real-time
- **Dashboard Display** — View Gas Town system status in a native desktop window

This is an alternative to the Gas Town web dashboard, offering native performance and deeper OS integration.

## Installation

### Prerequisites

- **Rust** (1.80+) with Cargo
- **macOS 14+**, **Linux** (with Wayland/X11), or **Windows 10+**
- **Gas Town CLI** (`gt`) — Install from [steveyegge/gastown](https://github.com/steveyegge/gastown)

### Build from Source

```bash
git clone https://github.com/tomatitito/Gazetown.git
cd Gazetown

# Build the application
cargo build -p gastown --release

# Run
cargo run -p gastown --release
```

The binary will be at `target/release/gastown`.

## Usage

### Launch the Application

```bash
# Run directly
cargo run -p gastown

# Or use the built binary
./target/release/gastown
```

This opens the Gas Town window where you can view your Town structure and agent activity.

### Typical Workflow

1. **Start Gas Town services** — Run your Gas Town backend (`gt` commands)
2. **Launch the UI** — `cargo run -p gastown`
3. **Monitor agents** — View Polecats executing work
4. **Track convoys** — Watch progress on bundled tasks

## Development

### Project Structure

```
crates/gastown/
├── src/
│   ├── main.rs              # Application entry point
│   ├── dashboard_buffer.rs  # Dashboard rendering component
│   └── *_tests.rs           # Test files
├── Cargo.toml
└── README.md
```

### Build Commands

```bash
# Check compilation (fast)
cargo check -p gastown

# Build debug
cargo build -p gastown

# Build release
cargo build -p gastown --release

# Run tests
cargo nextest run -p gastown
```

### Code Style

```bash
cargo fmt --all           # Format code
./script/clippy           # Run linter
```

## Architecture

Gas Town UI is built on the GPUI framework, providing:

- **GPU-accelerated rendering** — Smooth 120fps UI updates
- **Rust safety** — Memory-safe, concurrent operations
- **Native platform integration** — macOS, Linux, Windows support

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Town** | Root workspace directory containing all projects |
| **Rig** | Project container wrapping a git repository |
| **Polecat** | Ephemeral worker agent that executes tasks |
| **Crew** | Personal workspace for human developers |
| **Convoy** | Bundle of issues/tasks assigned to agents |
| **Hook** | Git worktree storing persistent agent state |

### Gas Town Integration

The UI communicates with Gas Town via:
- Reading convoy and agent state from the filesystem
- Parsing beads (`.beads/`) for issue tracking data
- Monitoring hook worktrees for agent work state

## Roadmap

| Phase | Status | Description |
|-------|--------|-------------|
| **PoC** | ✅ Done | Minimal GPUI window with basic rendering |
| **Dashboard** | 🚧 In Progress | Display Gas Town web dashboard content |
| **Rig Panel** | Planned | Visual tree of Town → Rigs → Agents |
| **Agent Monitor** | Planned | Real-time Polecat status and logs |
| **Convoy Tracker** | Planned | Progress visualization for work bundles |
| **MVP** | Planned | Fully functional alternative to web dashboard |

## License

This project is licensed under the [GPL-3.0](LICENSE-GPL).

## Acknowledgments

- **[Zed](https://zed.dev)** — For the GPUI framework and foundational codebase
- **[Gas Town](https://github.com/steveyegge/gastown)** — The multi-agent coordination system this UI serves
- **[Beads](https://github.com/steveyegge/beads)** — Git-native issue tracking integration
