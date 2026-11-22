# StarVizion — Overview

**Product:** StarVizion, a standalone application for Windows and Linux, written in Rust (see [Decisions](DECISIONS.md))
**Status:** Design consolidated from prior planning; not yet built. An earlier prototype explored some of these ideas but was never wired into a finished, user-facing state — this specification is a fresh design target, not a description of that prototype.

---

## Purpose

StarVizion lets pilots and players of any game create, arrange, and display live hardware-input visualization overlays on their desktop or in VR — without touching the game, without remapping inputs, and without adding any latency to their control chain.

StarVizion is its own application, not a feature of another one. It needs no other software to be useful: controller visualization works straight after install, whatever game is running. Game data and extra Vizlet content come in through plugins and providers built against the public [`starvizion-provider-api`](../crates/provider-api), which any developer can write against. Elite Intel 2.0 integrates through one such plugin, bundled with StarVizion and idle unless Elite Intel is running; StarVizion never requires Elite Intel.

## What StarVizion Does

StarVizion creates **Vizlets**: frameless, borderless, transparent, always-on-top windows that float above the desktop and the game, or — for VR players — overlays rendered directly into the VR compositor. Each Vizlet is a visual composition that can show:

- Live axis positions (joystick, throttle, rudder, sliders, rotaries)
- Button press/release states, including multi-stage triggers and 3-position switches
- Keyboard and mouse states
- Decorative HUD framing elements — scales, grids, cockpit-style graphics
- Computed displays driven by expression formulas that combine multiple inputs
- Game data from providers — for example, Elite Dangerous data through the bundled Elite Intel plugin

Vizlets are designed for players who fly in VR using springless HOTAS setups with no physical centre or stop feedback, who need visual confirmation of their current control states inside the headset. They also serve desktop players who want a live readout of their inputs visible alongside the game.

## What StarVizion Does Not Do

- It does not send, intercept, or remap any input to the game
- It does not read or modify any game's files, or read game memory
- It does not interact with the running game process
- It is a pure visualization and read-only monitoring tool

## Game Scope

StarVizion is not tied to any one game. Controller visualization is game-agnostic and works regardless of what game is running. Anything game-specific — game data, and the game modes used by the mode-based visibility system (which shows or hides Vizlets based on what the player is doing in-game — see [UI Layout & Mode-Based Visibility](ui-layout.md#mode-based-visibility)) — comes from a provider. The first game with mode detection is Elite Dangerous: Odyssey, through the bundled Elite Intel plugin; other games gain it when a provider for them exists.

## Document Map

- [Vizlets and the Node System](vizlet-and-node-system.md) — the Vizlet window, its NeuroNode building blocks, and the visual primitives available
- [StarCalc Expression Engine](starcalc-expression-engine.md) — the expression language that drives Live and Static behaviour
- [HoloFrames and Persistence](holoframes-and-persistence.md) — saving, grouping, exporting, and importing Vizlets
- [UI Layout](ui-layout.md) — the editor UI and mode-based visibility system
- [VR Overlay](vr-overlay.md) — desktop vs. VR rendering
- [Roadmap](roadmap.md) — features beyond the first release, and open questions
- [Decisions](DECISIONS.md) — why StarVizion is a standalone Rust application, and the other architectural decisions
