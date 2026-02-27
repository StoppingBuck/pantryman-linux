# Pantryman (Linux / GTK4)

A GTK4 desktop app for managing your pantry and recipes. Built with [Relm4](https://relm4.org/) on top of [Janus Engine](https://github.com/StoppingBuck/janus-engine).

## Features

- Browse and edit your ingredient library
- Track pantry stock (quantities + dates)
- Browse and plan recipes, filtered by what's in stock
- Knowledge base with ingredient notes
- Configurable data directory (defaults to `example/data/` during development)
- Settings stored in `~/.config/pantryman/user_settings.toml`

## Requirements

- Rust (stable)
- GTK4 development libraries
- libadwaita

On Arch Linux:
```bash
sudo pacman -S gtk4 libadwaita
```

On Debian/Ubuntu:
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev
```

## Getting started

```bash
# Clone janus-engine as a sibling directory first
git clone https://github.com/StoppingBuck/janus-engine ../janus-engine

# Build and run
./dev.sh run
```

## Commands

```bash
./dev.sh run            # Build and run
./dev.sh compile        # Compile without running
./dev.sh test           # Run all tests (requires display)
./dev.sh test-headless  # Run tests via xvfb-run
./dev.sh check          # cargo check
./dev.sh clean          # Clean build artifacts
```

For verbose logging: `RUST_LOG=debug ./dev.sh run`

## Data directory

By default, the app uses `example/data/` (pre-seeded with sample data). Point it at your real data via:

1. The `PANTRYMAN_DATA_DIR` environment variable, or
2. Settings → Data Directory in the app

The data format is documented in [janus-engine](https://github.com/StoppingBuck/janus-engine).

## Architecture

```
src/
  main.rs        — entry point
  lib.rs         — re-exports for tests
  app.rs         — AppModel (Relm4 SimpleComponent), message loop
  types.rs       — AppMsg enum and shared types
  pantry/        — pantry tab (list, detail, format)
  recipes/       — recipes tab (list, detail, edit)
  settings.rs    — settings panel
  sidebar.rs     — navigation sidebar
  kb.rs          — knowledge base tab
  dialogs.rs     — shared dialog helpers
  i18n.rs        — localisation helpers
  user_settings.rs — persisted user prefs
  config.rs      — compile-time defaults
```

## License

MIT
