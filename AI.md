# AI Development Notes

This codebase is developed with significant AI assistance (Claude Code).

## How AI is used

- Feature implementation and bug fixing via natural language prompts
- Relm4/GTK4 widget wiring and message plumbing
- Test generation
- Documentation drafting

## What AI does well here

- Keeping the Relm4 message loop correct (AppMsg variants, update handlers)
- Generating widget hierarchies from verbal descriptions
- Identifying GTK layout issues (halign/hexpand interactions, etc.)

## What AI doesn't replace

- Visual design intuition — you have to run the app to see if it looks right
- GTK version-specific behaviour knowledge
- Accessibility and UX judgement

## Trust boundary

All AI-generated code is compiled and manually tested before committing. GTK UI
changes are always verified visually.

## Notes for AI assistants working on this repo

- Use `./dev.sh compile` after every edit to the GTK app — fix all errors before continuing
- Business logic goes in janus-engine, not here
- All state changes go through `AppMsg` — never mutate app state from callbacks directly
- Async loads (DataManager) use `mpsc::channel` + background threads, never block the GTK main thread
- Data dir defaults to `example/data/`; env var `PANTRYMAN_DATA_DIR` overrides it
