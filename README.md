# Pantryman (Linux / GTK4)

A cross-platform recipe and pantry manager built for neurodivergents and others who struggle with cooking. This is the GTK4 desktop frontend, built with [Relm4](https://relm4.org/) on top of [Janus Engine](https://github.com/StoppingBuck/janus-engine).

***NOTE:** This project makes heavy use of AI in its development. See [AI.md](AI.md) for more information.*

---

## Vision

Pantryman was launched with four ambitions:

1. Made for neurodivergent people and others, who struggle in the kitchen to connect what they **have** (pantry) to what they **can do with it** (recipes).
2. Privacy by design (PbD) through allowing you to sync (or not) in any way you want.
3. Unified backend, freedom to frontend: Cram as much of the hard logic into a unified backend, and then have any number of frontends that can make use of it. I'm not an arbiter of fine UX. If you think my app is butt-ugly, you should be free to code a different frontend without having to fork the entire project. Having a strong backend (written in Rust, because of course it is) decoupled gracefully from the UI should make it easier to ensure that every platform has one (or more) app that looks just right for *it*. Alternative frontends are welcome for all platforms.
4. Maintain a knowledge base with information about ingredients, to engender familiarity with cooking as a (food) science and not just as an incomprehensible art form.

Pantryman is meant for people who relate to the quote "*I hate when I go to the kitchen looking for food, and all I find is ingredients.*" Its main purpose is to make it easy to keep an up-to-date overview of what you have in your pantry — and then use that information to show you what recipes you can make. Some people have the ability to look in the fridge and improvise — for everybody else, there is this app.

There's a gazillion cooking apps on the market already. Most of them attempt to tie you to an ecosystem or website of some kind — "create your CookWorld.com user to favorite your recipes, oops we leaked your personal info", etc. This app has nothing like that. No ecosystem, no website, no user creation, etc. Instead, it's BYOB — Bring Your Own Backend. Pantryman stores the ingredients, pantry and recipes as simple text files (YAML for ingredients and pantry, Markdown with YAML frontmatter for recipes). You can put the data directory containing these wherever you want — a local folder, a flash drive, your own self-hosted Nextcloud, a server, Dropbox or your cloud provider of choice... Pantryman doesn't care. It just needs to be able to read it. Your data stays yours by design.

---

## Features (v0.1.0)

- Browse, create, edit, and delete recipes (Markdown with YAML frontmatter)
- Pantry management: track what you have in stock with quantities and units
- Filter recipes by available ingredients
- Ingredient library with categories and tags
- Knowledge Base for culinary notes and technique articles
- Folder picker to point the app at any local or cloud-synced directory
- Light/dark/system theme

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
