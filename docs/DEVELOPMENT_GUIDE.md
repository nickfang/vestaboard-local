# Vestaboard Local - Development Guide

## Architecture Overview

### Layer Structure
- **UI Layer**: `main.rs`, `cli_display.rs`, `cli_setup.rs`, `config.rs` - CLI and user interaction
- **Execution Layer**: `scheduler.rs`, `daemon.rs`, `cycle.rs` - Different execution modes
- **Widgets Module**: Self-contained content generation with resolver interface
- **Translation Layer**: `api_broker.rs` - Message-to-code conversion and validation
- **Communication Layer**: `api.rs` - Direct Vestaboard API calls
- **Logging Layer**: `logging.rs` - File, console, and Vestaboard display logging

### Core Rules
1. **Widgets are isolated** - only use `widget_utils`, accessed via resolver
2. **Use resolver interface** - never access widgets directly from outside
3. **Respect layer boundaries** - UI → Execution → Widgets, UI → Translation → Communication
4. **Single validation point** - all messages validated before display/transmission
5. **Follow logging patterns** - file + console + Vestaboard display for new components

## Module Structure

```
src/
├── main.rs              # CLI entry point (UI Layer)
├── cli_display.rs       # Display utilities (UI Layer)
├── cli_setup.rs         # CLI parsing (UI Layer)
├── config.rs            # Configuration (UI Layer)
├── logging.rs           # File/console/Vestaboard logging (Logging Layer)
├── daemon.rs            # Background execution (Execution Layer)
├── scheduler.rs         # Schedule management (Execution Layer)
├── cycle.rs             # Loop execution (Execution Layer)
├── api_broker.rs        # Message translation (Translation Layer)
├── api.rs               # Vestaboard API (Communication Layer)
└── widgets/             # Self-contained content generation
    ├── resolver.rs      # Central widget execution
    ├── widget_utils.rs  # Shared utilities
    └── [widget_types]/  # Individual widgets
```

## Key Patterns

### Widget Execution
```rust
// ✅ Use resolver interface
let message = execute_widget("text", &input).await?;

// ❌ Never access widgets directly
use crate::widgets::text::get_text; // WRONG
```

### Logging Pattern
```rust
// Follow 3-tier logging for new components:
log::info!("Component action");           // File logging
println!("User message");                // Console output
display_message(error_message).await;    // Vestaboard display
```

### Widget Error Handling
```rust
// ✅ Widgets handle user communication and return structured errors
println!("Reading file: {}", file_path);
match std::fs::read_to_string(&file_path) {
    Ok(content) => Ok(process_content(content)),
    Err(e) => {
        eprintln!("Failed to read file: {}", file_path);  // User feedback
        Err(widget_utils::file_read_error(&file_path, e))  // Structured error
    }
}

// ✅ Use widget_utils for error creation (maintains separation)
use crate::widgets::widget_utils::{widget_error, io_error, network_error};

// ❌ Don't import errors directly in widgets
use crate::errors::VestaboardError;  // WRONG - breaks isolation
```

### Dry-Run Support
All execution modes support dry-run: Send (`--dry-run`), Scheduler (`dryrun`), Cycle (future)
- `dry_run = true`: Errors → display messages
- `dry_run = false`: Errors → propagated failures

## Development Guidelines

### Adding Widget Types
1. Create `src/widgets/[name]/` module (self-contained, use `widget_utils` only)
2. Add to `resolver.rs` match statement
3. Update `cli_setup.rs` for parsing
4. Add tests for normal + dry-run modes

### Adding Execution Modes
1. Follow scheduler/daemon patterns
2. Use resolver for widget execution
3. Support dry-run functionality
4. Implement 3-tier logging (file/console/Vestaboard)

### Code Standards
- **Format**: `cargo fmt` using `rustfmt.toml`
- **Comments**: Use `//` for regular comments, `///` for documentation comments only
- **Testing**: Unit + integration tests, verify dry-run
- **Logging**: File (`log::`), console (`println!`), Vestaboard (`display_message`)
- **Errors**: Use `error_to_display_message()` for consistency

## Common Mistakes

❌ **Don't**:
- Access widgets directly: `use crate::widgets::text::get_text`
- Import external modules in widgets: `use crate::api_broker::`
- Duplicate execution logic across files
- Skip dry-run testing

✅ **Do**:
- Use resolver: `execute_widget(type, input)`
- Keep widgets self-contained with `widget_utils`
- Follow layer boundaries: UI → Execution → Widgets
- Implement 3-tier logging for new components

## Developer Checklist

- [ ] Widgets isolated (only use `widget_utils`)
- [ ] External code uses resolver interface only
- [ ] Layer boundaries respected
- [ ] Dry-run works across all execution modes
- [ ] 3-tier logging implemented (file/console/Vestaboard)
- [ ] Code formatted (`cargo fmt`)
- [ ] Tests pass (normal + dry-run modes)

## References
- [Widget Execution Refactoring](./20250719-widget-execution-refactoring.md)
- `src/widgets/resolver.rs` - Central execution logic
- `src/logging.rs` - Logging patterns
