# Gazetown Development Environment

A multi-component system for managing Gazetown instances, overseer operations, and background services.

## Components

### Core Services
- **mayor/** - Overseer management system
  - Mail inbox management (`gt mail inbox`)
  - Rig operations (`gt rig list`)
  - Patrol coordination (`gt patrol start`)

- **daemon/** - Background activity tracking and monitoring
- **deacon/** - Service lifecycle management

### Gazetown Instances
- **gastown/** - Primary Gazetown instance
- **gazetown/** - Secondary Gazetown instance with extended documentation

Both instances include:
- **crew/** - Team and agent management
- **polecats/** - Task execution and coordination
- **refinery/** - Data processing and transformation
- **witness/** - Event observation and logging

### Extensions
- **plugins/** - Plugin system for extending functionality
- **settings/** - Global configuration management

## Quick Start

### Prerequisites
- Beads CLI (`bd`) for issue tracking
- GT tools for Gazetown operations

### Basic Operations

**Check system status:**
```bash
gt mail inbox         # Check overseer mail
gt rig list          # List available rigs
gt patrol start      # Start patrol operations
```

**Issue tracking:**
```bash
bd ready             # View available work
bd list              # List all issues
bd show <id>         # View issue details
```

## Development Workflow

This project uses **Beads** for AI-native issue tracking. All issues are tracked locally in `.beads/issues.jsonl`.

### Working with Issues

```bash
# Find work
bd ready

# Start working
bd update <id> --status=in_progress

# Complete work
bd close <id>

# Sync changes
bd sync --flush-only
```

### Session Management

**Starting a session:**
```bash
bd prime             # Restore context (auto-called by hooks)
bd ready             # Find available work
```

**Ending a session:**
```bash
bd sync --flush-only # Export beads to JSONL
```

## Project Structure

```
gt/
├── mayor/           # Overseer management
├── gastown/         # Gazetown instance 1
├── gazetown/        # Gazetown instance 2
├── daemon/          # Background services
├── deacon/          # Service management
├── plugins/         # Extension system
├── settings/        # Configuration
└── .beads/          # Issue tracking data
```

## Configuration

- **No git remote**: This repository operates in local-only mode
- **Issue tracking**: Local JSONL storage via Beads
- **Context recovery**: Automated via Claude Code hooks

## Documentation

- **CLAUDE.md / AGENTS.md** - AI agent workflow instructions
- **mayor/CLAUDE.md** - Mayor-specific commands and context
- **.beads/README.md** - Beads issue tracking guide

## Learn More

- **Beads Documentation**: [github.com/steveyegge/beads](https://github.com/steveyegge/beads)
- **GT Tools**: Check individual component directories for specific documentation

---

*A development environment for Gazetown operations and multi-agent coordination*
