# StarVizion — Roadmap and Open Questions

## Post-First-Release Roadmap

**Third-Party Vizlets and Providers** — other developers write plugins and providers against the public `starvizion-provider-api`, supplying game data, game modes and Vizlet content for games StarVizion itself knows nothing about. The API is versioned and changes only compatibly.

**Richer Elite Dangerous Data** — anything beyond what the bundled Elite Intel plugin offers (for example Frontier's Commander API data: market prices, fleet carrier status, stored loadouts) belongs in a provider, not in StarVizion. StarVizion never reads game files itself.

**Theme and Skin System** — a global colour theme system for Vizlets, with preset palettes applying consistently across all nodes, making it easy to rebrand an entire cockpit layout without editing each node individually. Pilots use it to match their Vizlets to the game's HUD or to an OBS scene. Already settled ([D11](DECISIONS.md#d11-starvizion-owns-colour-plugins-name-colour-roles)): plugins name colour roles and never choose colours, so plugin cards and templates follow the theme. Still to design: the full set of roles, the theme editor, per-HoloFrame theme overrides, how StarCalc refers to a role, and whether card images are tinted.

**Moving VR Overlays with Motion Controllers** — for pilots who keep their VR controllers charged: grab an overlay in the headset in a layout mode, with actions bound through SteamVR Input. See [VR Overlay — Positioning](vr-overlay.md#vr-overlay-positioning-and-persistence).

**Controller Map Integration** — when BindForge's own (also-roadmap) Controller Map mode is implemented, StarVizion could serve as the rendering engine for an interactive 3D controller model, sharing the input capture subsystem.

**Vizlet-Wide Shared Locals** — see [below](#vizlet-wide-shared-locals).

**Mode Detection for More Games** — StarVizion's input capture is game-agnostic, and mode detection comes from providers. The first release has it for Elite Dangerous only, through the Elite Intel plugin; providers for other games add it there.

**CortexUnit (Reusable/Shareable Vizlet Templates)** — see [below](#cortexunit-reusableshareable-vizlet-templates).

### CortexUnit (Reusable/Shareable Vizlet Templates)

**Confirmed as a wanted feature, not yet designed.** The purpose is community sharing: letting a Vizlet, or a reusable piece of one, be packaged and shared, so v1 doesn't depend on one person personally authoring every Vizlet that ships with it. "CortexUnit" is the confirmed working name.

**Prerequisite substantially satisfied, but CortexUnit's own design has not started — and that's fine, it's confirmed low-priority, not something to pick up right away.** CortexUnit's actual shape — what exactly gets templated (a whole Vizlet? a single NeuroNode? a reusable named group of nodes? possibly a whole HoloFrame, which raises the question of whether it replaces HoloFrames' existing plain export/import entirely), what's parameterized versus fixed at template-definition time, and how a placed instance relates back to its template after import — was blocked on StarVizion's own foundational structure being broken down first. That breakdown (every NeuroNode primitive, the Vizlet Editor's structure, Live/Static mode, HoloFrames, frame-rate input, keyboard capture, and VR rendering modes) is now largely done — see [Vizlets and the Node System](vizlet-and-node-system.md), [UI Layout](ui-layout.md), and [HoloFrames and Persistence](holoframes-and-persistence.md). The prerequisite being satisfied just means CortexUnit *could* be designed whenever it's actually wanted — it doesn't make it a priority.

An earlier design pass used this same name for a similar idea — a reusable, parameterized node template with per-instance parameter overrides (e.g., a "Throttle Arc" template with an overridable color and radius) — described using different underlying terminology (**NeuroFrame** instead of the current **NeuroNode**, **VizCluster** instead of **HoloFrame**, which itself was called **VizLoadout** for most of this documentation pass before being renamed) than the current spec uses. That prior description is a useful reference point, not a locked-in design (recorded in the Elite Intel repository's design archive, `docs/99-archive/conflicts-and-open-questions.md`, item 21).

### Vizlet-Wide Shared Locals

Local expressions are currently scoped to the single NeuroNode that defines them (see [StarCalc — Local Expressions](starcalc-expression-engine.md#local-intermediate-expressions)). A Vizlet with multiple nodes that all need the same derived value must define that local separately in each node.

Two options have been identified for a Vizlet-wide version, neither decided:

- **Option A — window and time variables only.** A shared locals block visible to every node, but restricted to window/time variables, not device inputs. Simpler; no new input-binding concept required.
- **Option B — Vizlet-level input bindings.** The Vizlet itself would have its own input-bindings section, and shared locals could reference those bindings directly. More powerful, but introduces a new layer in the data model and raises questions about binding identity and conflict resolution between Vizlet-level and node-level bindings of the same name.

Decisions needed before implementing either: which option (or both) to support; how Vizlet-wide locals interact with node-level locals of the same name; how Option B bindings would be displayed and edited in the editor; whether Vizlet-wide locals appear in a node's variable list in the expression editor.

## Open Questions

### Carried from the punch list, 2026-09-09

- **Vizlet Editor interaction design.** The Node Stack Panel, Property Panel, Device Input Monitor and
  Input Simulation Panel are all described in prose and built in the mockup, but none is designed at the
  *interaction* level — what dragging, selecting and editing actually feel like.
- **Expression editor panel layout.** Docked, floating, or a lightweight popover for the expanded `fx`
  editing surface. Previously deferred to "whichever integrates cleanest with the UI toolkit", which was
  a Godot-era answer; StarVizion's Rust UI toolkit is not chosen yet, so it needs a real choice.
- **Default bundled font.** The fallback when a Vizlet's referenced system font is missing on import. A
  HUD or space aesthetic is preferred; nothing is chosen.


**Composition/blending rules for overlapping NeuroNodes** — Z-order determines paint order, but exactly how one node's output blends with what's beneath it (simple alpha-over vs. something more nuanced) has never been explicitly stated as a rule anywhere. Likely simple, but not yet written down.

**Vizlet-wide shared locals, Option A vs. B** — see [above](#vizlet-wide-shared-locals).

Resolved items formerly tracked here — Frame-Rate Polling Ownership (StarVizion's own input layer owns it), Global Keyboard Capture (global/platform-native and opt-in for v1), and A Full Breakdown of Every Part of a Vizlet (substantially done) — now live in [Vizlets and the Node System](vizlet-and-node-system.md) and [above](#cortexunit-reusableshareable-vizlet-templates).
