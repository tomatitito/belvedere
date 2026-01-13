# Gas Town Framework Architecture Study

**Study Date:** 2026-01-13
**Source:** https://github.com/steveyegge/gastown
**Purpose:** Inform UI design for Gas Town visualization and operations

---

## 1. Core Concepts

### The Mayor 🎩
- **Role:** Primary AI coordinator
- **Implementation:** Claude Code instance with full workspace context
- **Purpose:** Main interface for users to describe what they want accomplished
- **Key Capability:** Orchestrates entire operations by delegating to other agents

### Town 🏘️
- **Definition:** Workspace directory (e.g., `~/gt/`)
- **Contents:** All projects, agents, and configuration
- **Structure:** Root container for the entire Gas Town system

### Rigs 🏗️
- **Definition:** Project containers
- **Purpose:** Wraps a git repository and manages its associated agents
- **Hierarchy:** Multiple rigs can exist within a Town
- **Example:** Each distinct project/repository gets its own rig

### Crew Members 👤
- **Definition:** Personal workspace within a rig
- **Purpose:** Where humans do hands-on work
- **Location:** `<rig>/crew/<name>`
- **Use Case:** Direct human involvement in a project

### Polecats 🦨
- **Definition:** Ephemeral worker agents
- **Lifecycle:** Spawn → Complete task → Disappear
- **Purpose:** Execute assigned work autonomously
- **Key Feature:** Temporary existence, task-focused

### Hooks 🪝
- **Technology:** Git worktree-based persistent storage
- **Purpose:** Store agent work state that survives crashes and restarts
- **Key Benefit:** Persistent work state across agent restarts
- **Implementation:** Each hook is a git worktree with version control

### Convoys 🚚
- **Definition:** Work tracking units
- **Purpose:** Bundle multiple issues/tasks for assignment to agents
- **Features:** Progress tracking, coordination, visibility across agents
- **Use Case:** Group related work items together

### Beads Integration 📿
- **Technology:** Git-backed issue tracking system
- **Purpose:** Store work state as structured data
- **Features:** Custom types, formulas, dependency tracking
- **Storage:** `.beads/` directory with git synchronization

---

## 2. Workflow Patterns

### MEOW (Mayor-Enhanced Orchestration Workflow) ⭐ RECOMMENDED

**Flow:**
1. **Tell the Mayor** - Human describes desired outcome
2. **Mayor analyzes** - Breaks down into manageable tasks
3. **Convoy creation** - Mayor creates convoy with issues
4. **Agent spawning** - Mayor spawns appropriate agents
5. **Work distribution** - Issues "slung" to agents via hooks
6. **Progress monitoring** - Track through convoy status commands
7. **Completion** - Mayor summarizes results

**Commands:**
```bash
gt mayor attach                                      # Start Mayor session
gt convoy create "Feature X" issue-123 issue-456    # Create convoy
gt sling issue-123 myproject                        # Assign work
gt convoy list                                      # Track progress
```

**Best For:** Complex, multi-issue work requiring coordination

---

### Minimal Mode (No Tmux)

**Flow:**
1. Create convoy manually
2. Sling work to agents
3. Manually start runtime instances (claude/codex)
4. Agents read mail and execute work
5. Check progress via convoy list

**Commands:**
```bash
gt convoy create "Fix bugs" issue-123
gt sling issue-123 myproject
claude --resume                    # or: codex
gt convoy list
```

**Best For:** Simple workflows without tmux overhead

---

### Beads Formula Workflow

**Flow:**
1. Define reusable workflow in TOML formula
2. Execute formula with variables
3. Track formula instance as molecule
4. Monitor step completion

**Formula Structure:**
- Variables (required/optional)
- Steps with dependencies
- Template substitution
- DAG execution

**Commands:**
```bash
bd formula list                          # List available formulas
bd cook release --var version=1.2.0     # Execute formula
bd mol pour release --var version=1.2.0 # Create trackable instance
bd mol list                              # Monitor instances
```

**Best For:** Predefined, repeatable processes (releases, deployments)

---

### Manual Convoy Workflow

**Flow:**
1. Create empty convoy
2. Manually add issues
3. Assign to specific agents
4. Monitor convoy status

**Commands:**
```bash
gt convoy create "Bug Fixes" --human
gt convoy add-issue bug-101 bug-102
gt sling bug-101 myproject/my-agent
gt convoy show
```

**Best For:** Direct control over work distribution

---

## 3. Information That Needs Visualization

### Dashboard Requirements (from README)

#### Real-time Agent Status
- Active agents list
- Agent states (spawned, active, suspended, completed, archived)
- Current agent assignments
- Agent health/connectivity

#### Convoy Progress Tracking
- List of all convoys
- Per-convoy issue completion status
- Convoy timeline/history
- Issue dependencies within convoys
- Blocking issues identification

#### Hook State Visualization
- Hook lifecycle states (Created → Active → Suspended → Completed → Archived)
- Git worktree locations
- Persistent work state contents
- Hook-to-agent mappings
- Hook git history

#### Configuration Management
- Runtime configurations per rig
- Agent preset definitions
- Default agent settings
- Per-rig settings display

### Additional Visualization Needs (inferred from architecture)

#### Hierarchy Visualization
```
Town
├── Mayor (coordinator)
├── Rig 1
│   ├── Crew Members
│   ├── Hooks (worktrees)
│   └── Polecats (workers)
├── Rig 2
│   ├── Crew Members
│   ├── Hooks (worktrees)
│   └── Polecats (workers)
```

#### Work Flow Visualization
- Issue assignment graph
- Agent → Hook → Work mappings
- Convoy → Issues → Agents relationships
- Dependency DAGs for formulas

#### Formula/Molecule Tracking
- Formula definitions
- Active molecule instances
- Step completion status
- Variable substitutions
- Dependency graphs

#### Agent Communication
- Mail inbox/outbox states
- Message routing (Mayor ↔ Polecats)
- Nudges and notifications
- Escalations

---

## 4. Operations Users Need to Perform

### Workspace Management

**Initial Setup:**
```bash
gt install ~/gt --git                    # Initialize workspace
gt rig add myproject <repo-url>          # Add project
gt crew add <name> --rig <rig>           # Create personal workspace
```

**Ongoing Management:**
```bash
gt rig list                              # List all rigs
gt agents                                # List active agents
gt hooks list                            # List hooks
gt hooks repair                          # Fix hook issues
```

---

### Work Assignment and Tracking

**Work Distribution:**
```bash
gt sling <issue> <rig>                   # Assign work to agent
gt sling <issue> <rig> --agent cursor    # Override runtime
gt convoy create <name> [issues...]      # Create convoy
gt convoy add-issue <issue>              # Add issue to convoy
```

**Monitoring:**
```bash
gt convoy list                           # List all convoys
gt convoy show [id]                      # Show convoy details
gt convoy refresh <convoy-id>            # Force convoy refresh
gt agents                                # Check agent status
```

---

### Mayor Operations

**Starting/Stopping:**
```bash
gt mayor attach                          # Start Mayor session
gt mayor start --agent auggie            # Start with specific runtime
gt mayor detach                          # Stop Mayor session
```

**Within Mayor Session:**
- Create convoys
- Orchestrate work distribution
- Monitor overall progress
- Communicate with agents

---

### Agent Configuration

**Runtime Management:**
```bash
gt config agent set <name> <command>     # Define custom agent
gt config default-agent <name>           # Set default
gt config agent list                     # List available agents
gt config show                           # View configuration
```

**Built-in Agent Presets:** `claude`, `gemini`, `codex`, `cursor`, `auggie`, `amp`

**Per-Rig Runtime Config:** Edit `settings/config.json`
```json
{
  "runtime": {
    "provider": "codex",
    "command": "codex",
    "args": [],
    "prompt_mode": "none"
  }
}
```

---

### Formula/Molecule Operations

**Formula Management:**
```bash
bd formula list                          # List available formulas
bd cook <formula> --var <k>=<v>         # Execute formula
bd mol pour <formula> --var <k>=<v>     # Create trackable instance
bd mol list                              # List active molecules
```

**Formula Development:**
- Create `.beads/formulas/<name>.formula.toml`
- Define variables, steps, dependencies
- Use template substitution (`{{variable}}`)
- Test with `bd cook`

---

### Dashboard Operations

**Starting Dashboard:**
```bash
gt dashboard --port 8080                 # Start web interface
open http://localhost:8080              # Open in browser
```

**Dashboard Features:**
- View real-time agent status
- Monitor convoy progress
- Visualize hook states
- Manage configurations

---

### Context Recovery

**After crashes or restarts:**
```bash
gt prime                                 # Load full role context
gt mail check --inject                   # Check for new assignments
```

---

### Communication

**Agent Messaging:**
```bash
gt mail inbox                            # Check mail
gt nudge <polecat> <message>            # Send message to agent
gt broadcast <message>                   # Message all agents
```

---

## 5. Key Architecture Patterns

### The Propulsion Principle
- **Concept:** Hooks as propulsion mechanism via git worktrees
- **Benefits:**
  - Persistent state survives agent restarts
  - Version control tracks all changes
  - Rollback capability to any previous state
  - Multi-agent coordination through git

### Hook Lifecycle States
```
[*] → Created (agent spawned)
    → Active (work assigned)
    → Suspended (agent paused)
    → Active (agent resumed)
    → Completed (work done)
    → Archived (hook archived)
    → [*]
```

### Multi-Agent Scaling
- **Problem:** 4-10 agents become chaotic
- **Solution:** Persistent hooks + structured work distribution
- **Result:** Comfortable scaling to 20-30 agents

---

## 6. UI Design Implications

### Primary User Flows to Support

1. **Mayor-First Flow** (Most Common)
   - Simple text input: "Tell the Mayor what you want"
   - Automatic convoy creation
   - Automatic agent spawning
   - Progress visualization

2. **Manual Convoy Management**
   - Convoy creation form
   - Issue selection/addition
   - Agent assignment interface
   - Progress dashboard

3. **Formula Execution**
   - Formula browser/selector
   - Variable input form
   - Step-by-step progress view
   - Dependency graph visualization

4. **Monitoring & Troubleshooting**
   - Agent health dashboard
   - Hook state inspector
   - Convoy progress tracker
   - Configuration viewer/editor

### Key UI Components Needed

1. **Hierarchy Browser**
   - Town → Rigs → Agents tree view
   - Expandable/collapsible navigation

2. **Convoy Dashboard**
   - Table/card view of all convoys
   - Per-convoy issue lists
   - Completion percentage
   - Blocking issues highlighted

3. **Agent Monitor**
   - Live agent status grid
   - State indicators (active/suspended/etc.)
   - Current assignments
   - Hook connections

4. **Formula Manager**
   - Formula library browser
   - Variable input forms
   - Execution history
   - Molecule tracking

5. **Configuration Editor**
   - Runtime configuration forms
   - Agent preset management
   - Per-rig settings

6. **Communication Center**
   - Mail inbox/outbox
   - Nudge/broadcast interface
   - Notification management

---

## 7. Technology Stack

### Required Dependencies
- Go 1.23+ (Gas Town implementation)
- Git 2.25+ (worktree support)
- beads (bd) 0.44.0+ (issue tracking)
- tmux 3.0+ (recommended for full experience)
- Claude Code CLI / Codex CLI / other runtimes

### Integration Points
- Git worktrees for persistence
- Git hooks for automation
- TOML for formula definitions
- JSON for configuration
- Beads JSONL for issue storage

---

## Summary

Gas Town is a sophisticated multi-agent orchestration system built on git infrastructure. Its key innovation is using git worktrees as persistent "hooks" to maintain work state across agent restarts, enabling reliable coordination of 20-30 agents.

The **Mayor-Enhanced Orchestration Workflow (MEOW)** represents the recommended user flow: humans interact primarily with the Mayor coordinator, which handles all the complexity of breaking down work, spawning agents, and tracking progress.

For UI design, the critical elements are:
- **Visibility:** Real-time agent status, convoy progress, hook states
- **Control:** Work assignment, agent configuration, formula execution
- **Communication:** Mail system, nudges, notifications
- **Recovery:** Context restoration, troubleshooting tools

The system scales through structured work distribution (convoys), persistent state (hooks), and clear role separation (Mayor vs Polecats vs Crew).
