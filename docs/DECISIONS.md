# Decisions

Decisions about StarVizion as a standalone project. Newest at the bottom. Each records who decided and when, and why.

## D1. StarVizion is its own project, not an Elite Intel feature

*Krondor, 2026-11-17, following a preliminary agreement with Alan.*

StarVizion leaves Elite Intel and becomes its own application and repository (`SudoKrondor/StarVizion`).
Elite Intel and StarVizion are independent: either installs and works without the other.

**Why:** pilots of other simulators share the same problem (springless sticks in VR), and would never find
StarVizion inside an Elite Dangerous voice assistant. A 90-144 Hz render loop, VR submission every frame and global
input hooks are also poor work for the JVM: Elite Intel's own HUD overlay is already a native child process for
that reason.

**What it supersedes in the Elite Intel spec:** anything that makes Elite Intel the host: Swing UI, `AppPaths`,
`elite.intel.devices` as the input source, and the Device Service owning frame-rate polling. StarVizion owns its
own input, paths, and UI.

## D2. Rust

*Krondor, 2026-11-17.*

Native code for the render loop, VR and input hooks, with memory safety and one cargo build for Windows and Linux.
Python was ruled out (thread model, installers). C++ works, but Rust removes the class of bugs that manual memory
management brings to Win32, X11 and OpenVR code.

## D3. Usable on its own; extended through providers

*Krondor, 2026-11-17.*

Straight after install, with no other software, StarVizion shows live controller state on the desktop and in VR.
Everything else arrives through providers (`starvizion-provider-api`), which any developer can write.

**Why:** EDO StellarCore stalled because a plugin host gives players nothing until its plugins exist. StarVizion's
first release must not need a plugin to be worth installing.

## D4. Elite Intel is a bundled plugin; Elite Intel 2.0 integrates

*Krondor, 2026-11-17.*

The Elite Intel integration ships with StarVizion and stays idle until it finds Elite Intel running, then needs no
setup. StarVizion is a candidate to display Elite Intel's overlays in VR. The Elite Intel side of the integration
lands in Elite Intel 2.0.

## D5. Keyboard capture is opt-in

*Krondor, 2026-11-17.*

Global keyboard capture is off until the user turns it on. On Linux it needs evdev access the user must grant, and
it is keylogger-shaped access that should be explained, not assumed. Many users will only want controller state
and game data in VR, where a keyboard is of little use anyway.

## D6. Native installers only

*Krondor, 2026-11-17.*

A setup `.exe` on Windows; `.deb` and `.rpm` on Linux (gamers mostly run Fedora- or Debian-derived distributions).
No Flatpak, no Snap, no storefront: a sandbox breaks global input hooks, desktop overlays and talking to other
local apps such as Elite Intel. Leading candidate tool: `cargo-packager` (NSIS on Windows), built per OS in GitHub
Actions.

## D7. Game-agnostic: anything game-specific comes from a provider

*Krondor, 2026-11-22.*

StarVizion is not specific to Elite Dangerous. It is hardware visual feedback for any game. StarVizion itself knows
no game: game data, and the game modes that drive mode-based visibility and HoloFrame switching, come only from
providers and plugins. The first release ships Elite Dangerous mode detection through the bundled Elite Intel plugin.
Other developers can write Vizlets and providers for other games against `starvizion-provider-api`.

**Why:** the problem StarVizion solves (seeing your controls when you cannot see your hands) is the same in every
simulator. Keeping the game out of the core is what lets third-party plugins cover other games without changing
StarVizion.

**What it supersedes in the carried-over design:** mode detection as a built-in journal parser, and journal and CAPI
telemetry as StarVizion subsystems.

## D8. Third-party plugins are WebAssembly components; StarVizion renders their cards

*Krondor, 2026-11-24.*

A third-party plugin is a `.svplugin` file: a zip holding `manifest.toml` (id, name, version, author, provider API
version, requested permissions), `plugin.wasm` (a WebAssembly component) and `assets/`. StarVizion runs it in an
embedded WebAssembly runtime (`wasmtime`). The contract is a versioned WIT interface (`starvizion:provider@1.0.0`),
which is what `starvizion-provider-api` publishes. A plugin describes the fields it offers, streams their values and
emits cards. Rust is the first-class plugin language. Any language that builds a WebAssembly component (JavaScript,
Python, Go, C/C++, C#) can also be used.

The user picks a `.svplugin` file from the StarVizion UI, sees what it asks to access (files, local network ports)
and approves it. Its Vizlet templates then appear in the Vizlet palette and are placed on the canvas like any other
Vizlet. The plugin sends data and declarative cards (text, gauges, lists, images). StarVizion's Vizlets render them,
so no plugin touches the GPU or the render thread.

Plugins StarVizion bundles and whose other side we own, such as Elite Intel (D4), are first-party Rust built into
StarVizion *(superseded by D12: bundled plugins are `.svplugin` files too)*. They connect to their application themselves. Running applications that feed StarVizion directly over a
local socket, using the same field and card model, may become a second tier later. That is not decided yet.

**Why:**
- Rust has no stable ABI, so native Rust plugins would only load safely when built with StarVizion's exact compiler
  and crate versions.
- Native plugins need a separate build for each OS. They also run in-process, where a panic or a blocking call
  freezes the render loop and the VR overlay.
- StarVizion cannot make downloaded native code honour "read-only towards games". A sandbox that grants only declared
  permissions can.
- Python or JavaScript run as a plain script would make users install a runtime. A component bundles it, so one file
  works on Windows and Linux.

**Risks to prove before the API is published:**
- WASI sockets and file access are newer parts of the standard. Prototype one Rust plugin and one Python plugin that
  read a game file and talk to a local port.
- Python plugins bundle an interpreter (tens of MB), and most packages with C extensions will not work in them.

**What it supersedes:** "any language" in `README.md`, which assumed an interface that no language-neutral
mechanism backed.

## D9. No compatibility obligations before 1.0

*Krondor, 2026-11-24.*

Until StarVizion 1.0 (the first public release) ships, everything is in-house. The StarCalc language, the Vizlet and HoloFrame file
formats and other user-facing formats may change without migrations or backward compatibility. From 1.0 on, the
compatibility rules in `docs/CODING_STANDARD.md` (API & Compatibility) apply in full. `starvizion-provider-api`
keeps the rule in `CLAUDE.md`.

**Why:** nobody outside the project has files or habits to break yet. Settling the language and formats by use,
before 1.0, beats carrying early guesses forever.

## D10. The first release is not localized

*Krondor, 2026-11-24.*

The first release is in English and formats numbers US-style (`.` for decimals) whatever the operating system's
locale.

**Why:** Rust's standard library cannot format by locale, and doing it properly needs large dependencies (ICU4X).
Nothing compelling justifies that cost before the product exists.

## D11. StarVizion owns colour; plugins name colour roles

*Krondor, 2026-11-26.*

Every colour a plugin's content shows comes from the pilot's theme. A plugin never chooses colours. A card names a
colour role (such as `primary`, `accent`, `warning`, `danger` or `muted`), and StarVizion fills it in from the active
theme. The card contract has no way to send a literal colour. The Vizlet templates a plugin ships use theme roles for
their colours. They contain no colour literals.

A pilot's own Vizlets can still use literal colours, because the pilot is choosing them. A plugin may still offer
`color` fields, such as a star's class colour or the game's HUD colours. These are data, and a pilot decides whether
to bind them.

**Why:** a pilot runs Vizlets from several plugins at once, next to their own. They should look like one cockpit,
matched to the game's HUD or to an OBS scene, whoever wrote the plugin. `starvizion-provider-api` is public and can
only change compatibly. Literal colours in cards would have to be supported for good once third-party plugins ship,
so the contract has to leave them out from the start.

**Still open:** the full set of roles, how a pilot edits a theme, theme overrides per HoloFrame, how StarCalc
refers to a role, and whether card images are tinted by the theme (see
[Roadmap — Theme and Skin System](roadmap.md#post-first-release-roadmap)).

## D12. Our game-data overlays are plugins; hardware visualization is built in

*Krondor, 2026-11-26.*

StarVizion has two kinds of Vizlet content:

- **Hardware visualization** is part of StarVizion. It shows the pilot's controllers from StarVizion's own input
  capture, works with no plugin installed (D3), and is optional: the pilot configures it and shows or hides it like
  any other Vizlet. The plugin architecture does not handle it, and no plugin ever sees controller input.
- **Data-driven overlays** come from plugins. Our own, the Elite Intel overlay cards and FrameShift's cards, are
  `.svplugin` WebAssembly components built against the same `starvizion-provider-api` and run in the same sandbox
  as any third-party plugin. They differ only in shipping with StarVizion, already installed and approved, so the
  Elite Intel integration still needs no setup (D4).

**Why:** our plugins are the first users of the plugin API and the sandbox, and we can change all three sides
freely before the first public release. Building them the way a third party would finds the gaps in the contract,
the permissions and the tooling before outside developers depend on them. It also leaves one plugin model to
build, test and secure instead of two.

**What it supersedes in D8:** bundled plugins as native Rust built into StarVizion. Bundled plugins are now
`.svplugin` files that ship with StarVizion.
