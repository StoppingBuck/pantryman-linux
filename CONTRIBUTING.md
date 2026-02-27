# Contributing to Pantryman (Linux)

## Setup

```bash
# Dependencies (Arch)
sudo pacman -S gtk4 libadwaita

# Dependencies (Debian/Ubuntu)
sudo apt install libgtk-4-dev libadwaita-1-dev

# Clone engine as sibling
git clone https://github.com/StoppingBuck/janus-engine ../janus-engine

# Compile
./dev.sh compile
```

## Compile loop

After every edit, run:
```bash
./dev.sh compile
```
Fix all errors before running again. Don't accumulate errors.

## Architecture rules

- **Business logic belongs in janus-engine.** If it's not UI code, it doesn't belong here.
- **Message passing.** All state changes go through `AppMsg` — no direct mutation from widget callbacks.
- **Async for slow I/O.** DataManager loads go in background threads with `mpsc::channel`; never block the GTK main thread.
- Each tab (pantry, recipes, kb) is a split module under its own directory.

## Tests

```bash
./dev.sh test              # requires display
./dev.sh test-headless     # via xvfb-run (CI-friendly)
cargo test -p pantryman-linux <test_name>   # single test
```

Tests in `tests/` cover formatting, data loading, and UI integration.

## Data directory

During development the app uses `example/data/`. Set `PANTRYMAN_DATA_DIR` or change it in Settings to point at real data.
