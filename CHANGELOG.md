# Changelog

All notable changes to Pantryman (Linux) will be documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

## [0.1.0] — 2026-02-27

Initial release. Extracted from the Pantryman monorepo.

### Added

- GTK4 desktop app built with Relm4
- Pantry tab: browse stock, add/remove/edit items with quantities and dates
- Ingredients tab: full CRUD for ingredient library
- Recipes tab: browse and filter by in-stock ingredients
- Knowledge base tab: ingredient notes and context
- Settings panel: configurable data directory with async load (non-blocking)
- Logging via `env_logger` (`RUST_LOG=debug` for verbose output)
- User settings persisted to `~/.config/pantryman/user_settings.toml`
- Async `DataManager` loading to prevent UI freeze on slow/network filesystems
