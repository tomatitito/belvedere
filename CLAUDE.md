# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status=in_progress  # Claim work
bd close <id>         # Complete work
bd sync --flush-only  # Export to JSONL (local-only, no git remote)
```

## Project Structure

- **mayor/** - Overseer management (mail, rigs, patrols)
- **gastown/** - Gazetown instance with crew, polecats, refinery, witness
- **gazetown/** - Another Gazetown instance with documentation
- **daemon/** - Background activity tracking
- **deacon/** - Service management
- **plugins/** - Extensibility system
- **settings/** - Configuration files

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Sync beads** - Export to JSONL:
   ```bash
   bd sync --flush-only
   ```
5. **Verify** - All changes saved and beads exported
6. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until beads are synced
- NEVER stop before syncing - that loses tracking context
- This repo has NO git remote - issues are local-only
- Use `bd sync --flush-only` to export beads to JSONL

## Common Workflows

**Starting work:**
```bash
bd ready              # Find available work
bd show <id>          # Review issue details
bd update <id> --status=in_progress  # Claim it
```

**Completing work:**
```bash
bd close <id1> <id2> ...    # Close all completed issues at once
bd sync --flush-only        # Export to JSONL
```

**Creating dependent work:**
```bash
bd create --title="Implement feature X" --type=feature --priority=2
bd create --title="Write tests for X" --type=task --priority=2
bd dep add beads-yyy beads-xxx  # Tests depend on feature
```

## Context Recovery

After session compaction, clear, or when starting new session:
```bash
bd prime    # Restore workflow context
```

Hooks auto-call this in Claude Code when `.beads/` is detected.
