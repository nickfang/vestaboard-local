# Standardize Dry-Run Implementation Across Commands

## Problem Description
The current dry-run functionality has inconsistent patterns across different commands, making the user experience confusing and the codebase harder to maintain.

## Current Inconsistencies

### 1. Different CLI Patterns
- **Send Command**: Uses `--dry-run` flag
  ```bash
  vbl send --dry-run text "hello"
  ```
- **Schedule Command**: Uses `dry-run` subcommand
  ```bash
  vbl schedule dry-run
  ```

### 2. Variable Naming Inconsistency
- Uses both `dry_run` and `test_mode` variables for the same concept in `main.rs`
- `dry_run` parameter in resolver functions
- `test_mode` variable in main execution logic

### 3. Missing Dry-Run Support
- **Daemon Command**: Has no dry-run capability at all
- No global `--dry-run` flag that could affect all commands

### 4. Implementation Differences
- **Send**: Preview single widget execution, user controls dry-run
- **Schedule**: Preview all scheduled tasks, always runs in dry-run mode
- **Daemon**: Always executes normally, no preview option

## Proposed Solutions

### Option A: Standardize on Flags (Recommended)
```bash
# Consistent flag pattern across all commands
vbl send --dry-run text "hello"
vbl schedule list --dry-run        # Preview what would be executed
vbl daemon --dry-run               # Run daemon in preview mode
```

### Option B: Standardize on Subcommands
```bash
# Consistent subcommand pattern
vbl send dry-run text "hello"
vbl schedule dry-run
vbl daemon dry-run
```

### Option C: Global Flag
```bash
# Global dry-run flag affecting all commands
vbl --dry-run send text "hello"
vbl --dry-run schedule list
vbl --dry-run daemon
```

## Implementation Tasks

### Phase 1: Code Standardization
- [ ] Standardize variable naming: use `dry_run` consistently, eliminate `test_mode`
- [ ] Update function signatures to use consistent parameter names
- [ ] Ensure all commands support dry-run in their core logic

### Phase 2: CLI Standardization (Choose Option A, B, or C)
- [ ] Update `cli_setup.rs` to implement chosen pattern
- [ ] Update command parsing logic in `main.rs`
- [ ] Add dry-run support to daemon command

### Phase 3: Feature Parity
- [ ] Implement dry-run for daemon command (preview scheduled executions)
- [ ] Ensure consistent behavior across all commands
- [ ] Update help text and documentation

### Phase 4: Testing & Documentation
- [ ] Add tests for dry-run functionality across all commands
- [ ] Update CLI help text and examples
- [ ] Update development guide with dry-run patterns

## Technical Notes

### Current Implementation Locations
- `src/cli_setup.rs`: CLI argument definitions
- `src/main.rs`: Command processing and `test_mode` variable
- `src/widgets/resolver.rs`: Core dry-run logic in `execute_widget()`
- `src/scheduler.rs`: Schedule dry-run via `print_schedule()`
- `src/daemon.rs`: No dry-run support

### Backward Compatibility
- Consider deprecation warnings for old patterns
- Maintain existing functionality during transition
- Update help text to guide users to new patterns

## User Experience Goals
1. **Consistency**: Same dry-run pattern across all commands
2. **Predictability**: Users know how to preview any operation
3. **Discoverability**: Help text clearly shows dry-run options
4. **Safety**: Easy to test changes before applying them

## Priority
Medium - Improves user experience and code maintainability, but doesn't block current functionality

## Estimated Effort
2-3 days for full implementation and testing

## Related Issues
- Links to any widget separation or CLI improvement issues
- Consider grouping with other CLI standardization work
