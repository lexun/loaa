# Process Compose & Devenv Integration

This document captures what we learned about managing services with process-compose and devenv.

## Overview

We use **devenv** which wraps **process-compose** to manage our development services (database and web server). Understanding how these tools interact is crucial for a smooth workflow.

## Key Concepts

### How It Works

1. **devenv.nix** declares our processes in `processes = { db = {...}, web = {...} }`
2. **devenv** generates a process-compose YAML config in the Nix store
3. **process-compose** orchestrates the actual processes using that config
4. Communication happens via a **Unix socket** at `.devenv/state/*/pc.sock`

### The `--keep-project` Problem

**Critical Discovery:** devenv adds the `--keep-project` flag to process-compose by default.

**What this means:**
- `process-compose down` stops processes but **doesn't terminate process-compose itself**
- Processes go to "Completed" state but TUI stays open
- This is intentional - preserves your TUI session across process restarts

**Implications:**
- Can't use `process-compose down` alone to fully shut down
- Need to kill the process-compose process itself with `pkill`
- This is why our `just stop` uses both commands

## Justfile Commands

### `just start`
```bash
devenv up
```
- Starts services in **foreground** with TUI
- Blocks the terminal
- Recommended for interactive development
- Press Ctrl+C to stop

### `just start-bg`
```bash
devenv processes up --detach
```
- Starts services in **background** (detached mode)
- Returns immediately, no TUI shown
- Good for automated workflows

### `just attach`
```bash
process-compose attach
```
- Connects to running process-compose instance via socket
- Shows TUI without starting new processes
- Works regardless of who started the services or how

### `just stop`
```bash
process-compose down           # Stop processes gracefully
pkill -f "process-compose.*devenv"  # Kill process-compose itself
```
**Why both commands:**
- First command stops processes gracefully
- Second command terminates process-compose (due to `--keep-project` flag)
- Without the pkill, TUI stays open with "Completed" processes

### `just restart`
```bash
process-compose down
pkill -f "process-compose.*devenv"
devenv processes up --detach
```
**Why this approach:**
- Clean shutdown and restart
- Always starts in background mode
- Consistent regardless of who started services
- TUI will close if open; run `just attach` to reconnect

## Common Workflows

### Developer Working Interactively
```bash
just start          # See TUI, monitor processes
# ... work in other terminals ...
Ctrl+C              # Stop when done
```

### Agent (AI) Needs to Restart Services
```bash
just restart        # Restarts in background
just attach         # Optional: show TUI
```

### Cross-Terminal Control
- **Terminal 1:** `just start` (TUI running)
- **Terminal 2:** `just stop` (shuts down TUI in Terminal 1)
- Works because both use the same socket

## Process-Compose Commands

### Socket-Based Commands (Work Remotely)
These connect via Unix socket and work from any terminal:

```bash
process-compose attach              # Connect TUI to running instance
process-compose down                # Stop processes
process-compose process list        # List all processes
process-compose process restart db  # Restart specific process
process-compose process logs web    # View process logs
```

### Direct Commands (Used by devenv)
```bash
process-compose up                  # Start (what devenv up does)
devenv processes up --detach        # Start in background
devenv processes down               # Stop (doesn't work with foreground!)
```

## Gotchas & Lessons Learned

### 1. `devenv processes down` Only Works for Detached Mode
- If you started with `devenv up` (foreground), `devenv processes down` **will fail**
- Error: "Process with PID XXX not found"
- **Solution:** Use `process-compose down` instead (works via socket)

### 2. Individual Process Restarts Are Flaky
- `process-compose process restart <name>` can be unreliable
- Web process would show "Skipped" or instantly go to "Completed"
- **Solution:** Use full `down + up` pattern instead

### 3. Hardcoding Process Names is Brittle
- Don't list service names in justfile (duplicates devenv.nix)
- Let devenv.nix be the single source of truth
- Use `down + up` to restart everything declaratively

### 4. Port Already in Use on Reload
If you see "port already in use" errors:
- Check if processes from previous instance are still running
- `pkill -f cargo-leptos` and `pkill -f surreal` to clean up
- Or use `just stop` which handles this

## Process States in TUI

- **Running** - Process is active
- **Ready** - Process passed readiness probe
- **Completed** - Process exited (may restart if `restart: on_failure`)
- **Failed** - Process crashed and won't restart

**Note:** "Completed" doesn't always mean stopped! Check if process is actually running.

## Debugging

### Check What's Running
```bash
ps aux | grep -E "process-compose|cargo-leptos|surreal"
```

### Check Process-Compose Socket
```bash
process-compose process list -o json
```

### View Logs
```bash
just logs           # Combined logs
just logs web       # Specific service
just logs db
```

### Manual Cleanup
```bash
pkill -f "process-compose.*devenv"
pkill -f cargo-leptos
pkill -f "surreal start"
```

## Architecture Decisions

### Why Down + Up Instead of Restart?
1. **Reliability** - Clean state every time
2. **Declarative** - Respects devenv.nix configuration
3. **Simplicity** - No process-specific logic in justfile
4. **Consistency** - Works the same for everyone

### Why Background Mode for Restart?
- Agents can restart without blocking
- User can choose to attach TUI afterwards
- Predictable: always same mode after restart

### Why Not Remove `--keep-project`?
- It's added by devenv's wrapper, not configurable
- Would require forking devenv or complex workarounds
- The pkill fallback is simpler and works reliably

## Future Improvements

### Potential Enhancements
- [ ] Investigate if devenv will add config option for `--keep-project`
- [ ] Consider background mode by default with systemd-style daemon
- [ ] Add health check commands to justfile
- [ ] Auto-restart on file changes (cargo-watch integration)

### Known Issues
- Web process shows "Completed" immediately (investigate separately)
- Reload port 3001 conflicts (hot-reload feature issue)

## References

- [devenv processes documentation](https://devenv.sh/processes/)
- [process-compose documentation](https://github.com/F1bonacc1/process-compose)
- [process-compose flags](https://github.com/F1bonacc1/process-compose#command-line-flags)
