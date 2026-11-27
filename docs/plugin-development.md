# StarVizion — Plugin Development

**Audience:** third-party developers who want to bring a game's data, or any tool's data, into StarVizion.
**Status:** first cut, written before any of it is built. It is the anchor for the plugin R&D, and it will change as
that work proves or disproves its assumptions. Nothing here is a published API yet.

Each part of this document is marked with how firm it is:

| Mark | Meaning |
|---|---|
| **Settled** | Recorded in [Decisions](DECISIONS.md). Changing it needs a new decision. |
| **Proposed** | The current best guess. Expect it to change once the [D8 spike](#rd-plan) runs. |
| **Open** | Not decided. Listed in [Open questions](#open-questions). |

Until StarVizion 1.0, nothing outside the project depends on any of this ([D9](DECISIONS.md#d9-no-compatibility-obligations-before-10)).
The provider API is still the one part held to the compatibility rule from its first published version
(`CLAUDE.md`, [API & Compatibility](CODING_STANDARD.md#api--compatibility)), so it is published only after the
spike.

---

## What a plugin is for

StarVizion knows no game ([D7](DECISIONS.md#d7-game-agnostic-anything-game-specific-comes-from-a-provider)).
Straight after install it shows controller state, and nothing else
([D3](DECISIONS.md#d3-usable-on-its-own-extended-through-providers)). Everything game-specific comes in through a
plugin:

- **Fields:** named, typed values such as `shields`, `fuel_main`, `gear_down` or `target_name`. A pilot binds a
  field to a NeuroNode exactly like a controller axis, and uses it in StarCalc
  ([Binding a NeuroNode](vizlet-and-node-system.md#binding-a-neuronode)).
- **Game modes:** the modes the plugin can detect for its game (on foot, in a ship, docked) and the current one.
  They drive [Mode-Based Visibility](ui-layout.md#mode-based-visibility) and automatic
  [HoloFrame switching](holoframes-and-persistence.md#mode-based-holoframe-switching).
- **Cards:** declarative content (text, gauges, lists, images) that StarVizion draws inside a Vizlet. Use a card
  when the content does not fit a handful of fields, such as a route list or a mission board.
- **Vizlet templates:** ready-made Vizlets that appear in the Vizlet palette, so a pilot gets something useful from
  the plugin without building it themselves.

A plugin never draws. It sends data and cards, and StarVizion renders them on the desktop and in VR. A plugin
cannot slow the render loop or the VR overlay, and the same plugin works whatever render mode the pilot uses.
**(Settled, D8)**

## What a plugin may not do

These follow from StarVizion's hard rule: it is read-only towards games (`CLAUDE.md`). **(Settled)**

- No sending, intercepting or remapping input. A plugin does not see the pilot's controller or keyboard input at
  all.
- No writing game files. File access a plugin is granted is read-only.
- No reading game memory, no injecting into a process, no talking to the game process.
- No access the user has not approved. A plugin gets exactly the permissions its manifest asks for and the user
  accepted, nothing more ([Permissions](#permissions)).

Reading what a game publishes on purpose is what plugins are for: journal or log files, a local telemetry port, an
official local API. If a game exposes data only through its memory, StarVizion cannot use it.

---

## Kinds of plugin

| Kind | Who writes it | How it runs | Status |
|---|---|---|---|
| **Third-party plugin** | Anyone | A `.svplugin` file: a WebAssembly component in StarVizion's sandbox | **Settled** (D8). This document. |
| **Bundled plugin** | The StarVizion project | A `.svplugin` like any other, in the same sandbox, shipped with StarVizion already installed and approved (Elite Intel, [D4](DECISIONS.md#d4-elite-intel-is-a-bundled-plugin-elite-intel-20-integrates); FrameShift) | **Settled** ([D12](DECISIONS.md#d12-our-game-data-overlays-are-plugins-hardware-visualization-is-built-in)). |
| **Feeding application** | Anyone | A running application sends fields and cards to StarVizion over a local socket | **Open.** May become a second tier later (D8). |

Bundled plugins and third-party plugins are built the same way, against the same API, and run in the same sandbox.
To a pilot they look the same.

Hardware visualization is not a plugin. It is part of StarVizion, driven by StarVizion's own input capture, and
works with no plugin installed. No plugin sees controller input ([D12](DECISIONS.md#d12-our-game-data-overlays-are-plugins-hardware-visualization-is-built-in)).

---

## The developer's path

What writing a plugin should feel like, end to end. **(Proposed)**, except where marked.

1. **Pick a language.** Rust is first-class: StarVizion's own tooling and examples are Rust. Any language that
   builds a WebAssembly component also works. **(Settled, D8)**
2. **Start from a template.** A template project has the manifest, the generated bindings and a build script
   already wired up.
3. **Describe what the plugin offers:** its fields, modes, cards and Vizlet templates.
4. **Read the data source** (a file, a local port) through the access the manifest asks for, and turn it into field
   updates and cards.
5. **Build** the component, then **pack** it with its manifest and assets into a `.svplugin`.
6. **Test** it without StarVizion's UI, against a host that replays recorded data
   ([Testing](#testing-a-plugin)).
7. **Try it in StarVizion:** add the `.svplugin` from the UI, approve its permissions, place one of its Vizlet
   templates, fly.
8. **Ship it** wherever you like. StarVizion has no plugin store; a pilot downloads the file and adds it.

### A plugin in Rust (sketch)

What the Rust side might look like. The crate name, trait and types are placeholders until the API exists.

```rust
use starvizion_plugin::{export_plugin, Field, FieldType, Host, Mode, Plugin, PluginInfo, Update};

struct JournalPlugin {
    reader: JournalReader, // the plugin's own code: tails the game's journal file
}

impl Plugin for JournalPlugin {
    fn start(host: &Host) -> Result<(Self, PluginInfo), String> {
        let info = PluginInfo::new()
            .field(Field::new("fuel_main", "Main tank", FieldType::Number).unit("t").range(0.0, 32.0))
            .field(Field::new("gear_down", "Landing gear down", FieldType::Boolean))
            .field(Field::new("system_name", "Current system", FieldType::Text))
            .mode(Mode::new("ship", "Ship flight"))
            .mode(Mode::new("on_foot", "On foot"));
        let reader = JournalReader::open(host.granted_dir("journal")?)?;
        Ok((JournalPlugin { reader }, info))
    }

    fn poll(&mut self, _host: &Host) -> Vec<Update> {
        // Called by StarVizion at the rate the manifest asks for. Returns only what changed.
        self.reader.new_events().flat_map(to_updates).collect()
    }
}

export_plugin!(JournalPlugin);
```

---

## The `.svplugin` file

A zip with three parts. **(Settled, D8)**

```
my-plugin.svplugin
├── manifest.toml   identity, provider API version, permissions
├── plugin.wasm     the WebAssembly component
└── assets/         images for cards, Vizlet templates
```

The same file runs on Windows and Linux. There is no per-OS build.

### `manifest.toml`

D8 settles what the manifest holds: id, name, version, author, provider API version and requested permissions.
The exact keys below are **proposed**.

```toml
[plugin]
id = "com.example.journal-reader"   # reverse-DNS, unique, never changes between versions
name = "Example Journal Reader"
version = "0.3.1"                   # the plugin's own version (semver)
author = "Example Dev"
description = "Fuel, landing gear and current system for Example Game."
homepage = "https://example.com/journal-reader"
provider_api = "1.0"                # the starvizion:provider version it was built against

[permissions]
# Each entry is shown to the user, with its reason, before they approve.
files = [
  { id = "journal", path = "{home}/Saved Games/Example Game", access = "read", reason = "Read the game's journal" },
]
network = [
  { host = "127.0.0.1", port = 20777, reason = "Example Game's telemetry port" },
]

[runtime]
poll_hz = 10                        # how often StarVizion calls poll(); capped by StarVizion
```

---

## The contract: `starvizion:provider`

The contract between StarVizion and a plugin is a versioned WIT interface, `starvizion:provider@1.0.0`, published
by `starvizion-provider-api`. Every language generates its bindings from that one file. **(Settled, D8)**

Its contents are **proposed**. A first sketch:

```wit
package starvizion:provider@1.0.0;

interface types {
  variant value { number(f64), boolean(bool), color(u32), text(string) }

  enum value-type { number, boolean, color, text }

  record field {
    id: string,              // stable, used in saved Vizlets
    label: string,           // shown in the binding picker
    value-type: value-type,
    unit: option<string>,
    range: option<tuple<f64, f64>>,
  }

  record mode { id: string, label: string }

  variant update {
    field-value(tuple<string, value>),
    field-cleared(string),   // the value is unknown right now
    current-mode(option<string>),
    card(card),
  }

  // card: see "Cards" below
}

interface host {
  log: func(level: log-level, message: string);
  // file and socket access come through WASI, limited to what the manifest granted
}

world plugin {
  import host;
  export start: func(config: config) -> result<plugin-info, string>;
  export poll: func(now-ms: u64) -> list<update>;
  export stop: func();
}
```

### Fields

- A field's type is one of StarCalc's four value types: **number, boolean, color, text**
  ([StarCalc — Language Rules](starcalc-expression-engine.md#language-rules)). StarCalc converts nothing implicitly, so a field announced as
  a number always arrives as a number.
- A field's `id` is what a pilot's saved Vizlet refers to. Renaming it breaks their Vizlets. Treat field ids like
  public API: add new ones, keep old ones.
- A field with no value yet, or whose source has gone away, is **cleared**, not set to zero. The NeuroNode then
  falls back the way any failing expression does: last good value, or its default
  ([StarCalc — Error Handling](starcalc-expression-engine.md#error-handling)).
- The plugin sends only what changed. StarVizion keeps the latest value of each field.

### Modes

A plugin announces the modes it can detect and reports the current one, or none when it cannot tell. Mode detection
is best-effort: when a plugin stops reporting, Vizlets fall back to their manual state
([Where Modes Come From](ui-layout.md#where-modes-come-from)). StarVizion debounces mode changes itself; a plugin
reports what it sees as soon as it sees it.

**Open:** what happens when two plugins both report a mode (two games running, or two plugins for the same game).

### Cards

A card is a small declarative tree that StarVizion lays out and draws inside a Vizlet. **(Settled, D8:** text,
gauges, lists, images. **The element set is proposed.)**

| Element | Holds |
|---|---|
| `text` | a string, with size, weight and an optional colour role |
| `gauge` | a value, a range, an optional label and unit, and an optional colour role |
| `list` | rows of elements |
| `stack` | elements laid out vertically or horizontally |
| `image` | a file from the plugin's `assets/` |

A card has an id; sending a card with the same id replaces it. A card has no scripts, no HTML and no free-form
drawing. That keeps it cheap to render in VR and safe to accept from untrusted code.

**Colour comes from the pilot's theme, not the plugin. (Settled, D11)** A card element names a colour role, and
StarVizion takes the actual colour from the active theme. A card cannot carry a literal colour. Cards from every
plugin therefore match each other and the pilot's own Vizlets, whether they are matched to the game's HUD or to an
OBS scene. Leave the role out and StarVizion uses the theme's default for that element. Use a role for its meaning:
`warning` for low fuel, not because you want orange.

The roles are **proposed**:

| Role | For |
|---|---|
| `primary` | normal content; the default |
| `accent` | the thing to look at first |
| `muted` | secondary detail and labels |
| `ok` | a good or ready state |
| `warning` | needs attention soon |
| `danger` | needs attention now |

```wit
enum color-role { primary, accent, muted, ok, warning, danger }
```

**Open:** whether StarVizion tints `image` elements with the theme (monochrome images drawn in a role's colour), or
shows them as they are.

A `color` *field* is different: it is data, such as a star's class colour, and a pilot chooses whether to bind it.
Plugins may offer colour fields.

### Vizlet templates

A plugin can ship Vizlet templates in `assets/`, in StarVizion's own Vizlet format, bound to its fields. They appear
in the Vizlet palette; a pilot places one and edits it like any other Vizlet. **(Settled, D8; the file layout is
proposed.)** A template's colours are theme roles, never literal colours **(Settled, D11)**, so it matches the rest of
the pilot's cockpit as soon as it is placed.

The Vizlet file format is not settled before 1.0 (D9), so a template may need rebuilding against a newer StarVizion
until then.

---

## Lifecycle

**(Proposed)**

1. **Added.** The pilot picks a `.svplugin` in the StarVizion UI. StarVizion reads the manifest and checks it is
   valid and that it supports the plugin's `provider_api` version.
2. **Approved.** StarVizion shows the plugin's name, author and each permission with its reason. Nothing runs until
   the pilot approves. A new version that asks for more than was approved asks again.
3. **Started.** StarVizion loads the component and calls `start`. The plugin returns what it offers.
4. **Running.** StarVizion calls `poll` at the manifest's rate, off the render thread, and applies the updates.
5. **Stopped.** On shutdown, or when the pilot disables or removes it, StarVizion calls `stop` and drops the
   instance. Its fields clear.

### When a plugin misbehaves

A plugin is a deliberate failure boundary ([Coding Standard](CODING_STANDARD.md)):

- A trap (the WebAssembly equivalent of a crash) stops that plugin only. Its fields clear, the pilot sees it
  flagged, StarVizion and the other plugins carry on.
- A `poll` that runs too long is cut off by a time budget.
- Everything a plugin sends is untrusted: StarVizion validates it, caps its size and rate, and drops what does not
  match what the plugin announced (a text value for a number field, an unknown field id).

**Open:** the actual budgets (time per `poll`, memory, updates per second, card size), and whether a plugin that
keeps trapping is restarted, and how often.

---

## Permissions

A plugin runs in `wasmtime` with WASI, and gets only what its manifest asked for and the pilot approved.
**(Settled, D8:** files and local network ports. **The details are proposed.)**

| Permission | Grants | Never grants |
|---|---|---|
| `files` | Read access to one directory (or file), mounted for the plugin | Writing, or anything outside that path |
| `network` | A connection to one local host and port | Any non-local address |

Paths can use placeholders (`{home}`, `{documents}`, and the like) so one manifest works on Windows and Linux.

**Open:** whether a plugin may reach the internet at all (for example a game's official web API). As proposed, no.

---

## Languages

| Language | Toolchain (current candidates) | Notes |
|---|---|---|
| Rust | `cargo` with the `wasm32-wasip2` target, `wit-bindgen` | First-class. Smallest plugins, best tooling. |
| JavaScript / TypeScript | `jco` / `componentize-js` | Bundles a JS engine; larger files. |
| Python | `componentize-py` | Bundles an interpreter (tens of MB). Most packages with C extensions will not work. |
| Go | TinyGo with WASI Preview 2 | |
| C / C++ | `wit-bindgen` C bindings, `wasi-sdk` | |
| C# | `componentize-dotnet` | |

Only Rust and Python are in the spike; the rest are listed because they build components today, not because they
are tested. Which of them StarVizion documents and ships templates for is **open**.

---

## Testing a plugin

**(Proposed)**

- **Unit tests** in the plugin's own language, for the parsing and mapping code, the way any library is tested.
- **A headless host** (`starvizion-plugin-host` or a subcommand of a packing tool) that loads a `.svplugin`, grants
  it a directory of recorded files or a replayed port, calls `poll`, and prints or asserts the updates. No StarVizion
  UI, no game.
- **In StarVizion**, a developer view that lists each plugin's fields with their live values, its current mode and
  its last error.

Recorded sample data (a journal from a real session, a capture of a telemetry stream) is what makes a plugin
testable without the game running. Templates should come with a place for it.

---

## Versioning

- **The provider API** follows semver ([API & Compatibility](CODING_STANDARD.md#api--compatibility)): additive
  changes in a minor version, breaking ones only in a new major version, with explicit approval. A plugin built
  against `1.0` keeps working on every StarVizion that supports `1.x`. **(Settled)**
- **How many major versions StarVizion keeps loading at once** is **open**.
- **The plugin's own version** is the author's. StarVizion uses it to tell an update from a new plugin (same `id`,
  higher `version`).

---

## Security

A plugin is untrusted code from a stranger. StarVizion has to assume that some plugin, some day, is written to do
harm, and keep the pilot safe anyway.

### What keeps a pilot safe

The sandbox does, not reviewing or scanning plugins. **(Settled, D8)**

- A plugin is a WebAssembly component. It has no system calls of its own. It can do only what StarVizion hands it.
- It cannot start programs, load native code, write any file, see controller or keyboard input, open windows, touch
  the GPU, or reach other processes. Ordinary malware behaviour (dropping files, persisting, spreading, encrypting
  files, logging keys) cannot be expressed in a plugin.
- It can read only the paths and reach only the local ports in its manifest that the pilot approved
  ([Permissions](#permissions)).
- A plugin that crashes or hangs stops on its own, and everything it sends is validated
  ([When a plugin misbehaves](#when-a-plugin-misbehaves)).

The rules below close the gaps that remain inside that sandbox. They are **proposed** unless marked otherwise.

### Read access

Read-only access is still access. A plugin granted `{home}` could read SSH keys, browser profiles and saved
passwords.

- StarVizion refuses a manifest that asks for a drive root, the home directory itself, or a known-sensitive
  location (`.ssh`, browser profile folders, the OS credential stores, StarVizion's own settings).
- The permission prompt shows the resolved path itself, prominently. The plugin's `reason` is shown beside it, but
  it is the author's text and may lie.
- A link inside a granted folder cannot lead outside it. wasmtime's directory access is meant to guarantee this.
  StarVizion tests it on Windows (junctions, shortcuts) and Linux (symlinks) rather than assume it.

### Local ports

"Local only" is not the same as harmless. A local port can belong to OBS's WebSocket (which controls OBS), another
tool's API, or a proxy that relays to the internet. A plugin with file access and such a port could send what it
reads somewhere else.

- The prompt names what is listening on the port when StarVizion can tell, and warns when a plugin asks for both
  file access and a port.
- `localhost` means the pilot's own machine. Other machines on the LAN are a separate permission, if they are
  allowed at all (**open**).

**Open:** internet access. As proposed, no plugin gets it. If it is ever allowed, it must not be combined with file
access in the same plugin, because that pairing is what turns a plugin into a way to steal files.

### Resource limits

A plugin must not be able to slow the pilot's machine while they fly in VR.

- CPU: each `poll` runs under a time budget (wasmtime epoch interruption) and is cut off when it overruns.
- Memory: a hard cap per plugin (wasmtime `ResourceLimiter`). A plugin that hits it traps.
- Output: caps on updates per second, card size and log volume, so a plugin cannot fill the disk through `log`.

**Open:** the numbers (see [When a plugin misbehaves](#when-a-plugin-misbehaves)).

### The `.svplugin` file

The file itself is attacker input, before any of its code runs.

- The zip is read without extracting paths as given: no `..`, no absolute paths, no links (zip slip).
- Sizes are capped before decompressing, to stop zip bombs.
- The manifest is parsed strictly. Unknown keys are refused, not ignored.
- Images in `assets/` are decoded with memory-safe Rust decoders, with limits on dimensions and file size.

### Identity and updates

Today any plugin can claim `author = "Elite Intel"`, and anyone can ship a "new version" under another plugin's
`id`.

- A plugin is signed by its author's key. The pilot sees which key signed it, not only the author name.
- An update is accepted as an update only when it has the same `id` **and** the same signing key as the version the
  pilot approved. Otherwise it is a different plugin and is approved from scratch.
- A new version that asks for more permissions asks the pilot again. **(Settled, D8)**

Signing says who wrote a plugin. It does not say the plugin is safe; the sandbox does that.

**Open:** what "passing our validation" means for a third-party plugin. The automatic checks on this page run on
every plugin added. Whether there is also a curated list of known plugins, or human review before a plugin is
listed, is not decided. Review cannot be relied on to catch a hostile plugin, so it would be in addition to the
sandbox, never instead of it.

### StarVizion's own side

- **wasmtime** is the wall. A bug in it is the worst case: a plugin escaping the sandbox. StarVizion tracks
  wasmtime's security releases and ships them promptly, which is one reason to have an update check.
- **Host functions** (`log`, and any added later) are code StarVizion runs on a plugin's behalf. Each one validates
  its arguments like any other plugin input.
- **Card text** is drawn as text, never interpreted. A card cannot hold a link, a command or markup.

### Distribution

**(Proposed)**

- A `.svplugin` is a file the pilot downloads and adds. There is no store, no account and no install step outside
  StarVizion's UI.
- The sandbox and the permission prompt are what keep a pilot safe, not where the file came from.

**Open:** a curated list of known plugins, and automatic update checks.

---

## R&D plan

The order the plugin work is likely to go in. The build-plan brief (`docs/build-plan/plugins.md`, not written yet)
will carry the real slices.

1. **D8 spike.** One Rust plugin and one Python plugin, each reading a file and talking to a local port, loaded by a
   bare `wasmtime` host. Proves WASI file and socket access and measures plugin size and start-up time.
   ([D8 risks](DECISIONS.md#d8-third-party-plugins-are-webassembly-components-starvizion-renders-their-cards))
2. **The WIT contract.** Fields, modes, updates; then cards. Lives in `crates/provider-api`.
3. **The host in StarVizion.** Load, approve, start, poll, contain failures, feed fields into NeuroNode bindings.
4. **Developer tooling.** Rust template, packing tool, headless host.
5. **First publish** of `starvizion:provider@1.0.0`, after which compatibility rules apply.

---

## Open questions

- `poll` called by StarVizion, or plugins that push updates themselves (WASI 0.3 async)? Polling is simpler and
  keeps StarVizion in charge of the rate; pushing suits event-driven sources such as a socket.
- Time, memory, rate and size budgets for a plugin.
- Restart policy for a plugin that keeps trapping.
- Two plugins reporting a mode at once.
- Field id namespacing: does StarVizion prefix field ids with the plugin id, so two plugins can both offer `fuel`?
- Per-plugin settings a pilot can change (an IP address, a units choice), and how the UI shows them.
- Internet access, LAN access, the signing scheme, what "passing our validation" means, and update checks
  ([Security](#security)).
- Which non-Rust languages get documentation and templates.
- Whether feeding applications over a local socket become a supported tier (D8).
- How a plugin's Vizlet templates relate to [CortexUnit](roadmap.md#cortexunit-reusableshareable-vizlet-templates),
  if CortexUnit is ever designed.

---

## Changes to this document

| Date | Change |
|---|---|
| 2026-11-25 | First cut, from D3, D7, D8 and the design docs. |
| 2026-11-26 | Cards and Vizlet templates use theme colour roles, not literal colours (D11). |
| 2026-11-26 | Added the Security section; Distribution and trust moved under it. |
| 2026-11-26 | Bundled plugins are `.svplugin` files in the same sandbox; hardware visualization is not a plugin (D12). |
