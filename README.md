# StarVizion

Live overlays for simulator pilots, on the desktop and in VR: where your stick, throttle and
pedals sit right now, which buttons are held, and whatever game data you choose to show.
Built for pilots flying springless HOTAS or HOSAS setups in a headset, who can feel neither a centre
nor an end stop and cannot see their own hands.

StarVizion is read-only. It never sends, intercepts or remaps input, never touches the game's
files, and never interacts with the game process.

StarVizion is a standalone application for Windows and Linux, written in Rust. It is not tied to
any one game: controller visualization works whatever you are playing.

**Status:** scaffold only. Development starts after Elite Intel 1.2 ships (December 2026).

## Standalone, with plugins

StarVizion is useful straight after installing it, with no other software: build Vizlets showing
your controllers' live state, group them into HoloFrames and show them on the desktop or in VR.

Everything beyond that comes in through **providers**: a provider announces the fields it offers
and streams their values, and Vizlets bind to them like any controller axis. Any developer can write
one, in any language, against [`starvizion-provider-api`](crates/provider-api), to bring data from
any game or tool into StarVizion.

**Elite Intel** integration is one such plugin, shipped with StarVizion, for Elite Intel 2.0. It stays
idle until Elite Intel is running. Neither app needs the other: some pilots will run only one of them.

## Layout

| Path | What it is |
|---|---|
| `crates/starvizion` | The application: wires everything together and hosts the editor |
| `crates/model` | Vizlets, NeuroNodes, HoloFrames, and their files |
| `crates/starcalc` | The StarCalc expression language |
| `crates/input` | Controllers, mouse, and opt-in keyboard capture |
| `crates/render` | Composes a frame and presents it on the desktop |
| `crates/vr` | VR compositor overlays |
| `crates/provider-api` | The public API for providers and plugins |
| `plugins/elite-intel` | The bundled Elite Intel plugin |

## Building

Install Rust with [rustup](https://rustup.rs), then:

```bash
cargo build             # debug build of the app
cargo run               # run it
cargo test --workspace  # all tests
```

## Platforms

Windows and Linux. Installers: a setup `.exe` on Windows, `.deb` and `.rpm` on Linux.

## Design

The product design lives in [`docs/`](docs), starting at [docs/overview.md](docs/overview.md).
Why StarVizion is shaped this way is in [docs/DECISIONS.md](docs/DECISIONS.md).

The design was first written while StarVizion was planned as a feature inside Elite Intel. It has
been carried over here and revised for StarVizion as a standalone application; the Elite Intel
repository's `docs/02-features/starvizion/` copy is historical.

## Licence

MIT. See [LICENSE](LICENSE).
