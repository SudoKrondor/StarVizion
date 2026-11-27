# CLAUDE.md

Guidance for Claude Code working in this repository.

## What this is

StarVizion: live controller-input and game-data overlays, on the desktop and in VR. A Rust cargo workspace,
Windows and Linux. Read `README.md` for the layout and `docs/DECISIONS.md` for why it is shaped this way.

The product design (Vizlets, NeuroNodes, StarCalc, HoloFrames, Vizlet windows) lives in `docs/`, starting at
`docs/overview.md`. StarVizion is a standalone application, not an Elite Intel feature and not specific to any game.
If a design doc still assumes Elite Intel as the host, `docs/DECISIONS.md` wins.

## Commands

```bash
cargo build                                          # debug build
cargo run                                            # run the app
cargo test --workspace                               # all tests
cargo fmt --all                                      # format
cargo clippy --workspace --all-targets -- -D warnings  # lint, as CI runs it
```

## Rules

Follow `docs/CODING_STANDARD.md`. The rules below take precedence where they overlap.

- **Read-only towards games.** Never send, intercept or remap input, never write a game file, never read
  game memory or inject into a process. Game data comes only from providers.
- **Standalone first.** Nothing a pilot needs for controller visualization may depend on a plugin or on
  Elite Intel being installed.
- **`starvizion-provider-api` is public.** Third-party developers build against it: change it only
  compatibly, and version it.
- **No sandboxed packaging.** Installers are a setup `.exe`, `.deb` and `.rpm`. Never Flatpak or Snap.
- Record a new architectural decision in `docs/DECISIONS.md`, with who decided it and when.
