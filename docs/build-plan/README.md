# Build plan

How StarVizion gets built: in **sections** (one per area of the product), each split into **slices** small enough
for one session. Each section has a brief, `docs/build-plan/<section>.md`. The brief carries what a long session
would otherwise keep in its head: what exists, what is next and what the last slice found. So every session starts
fresh from the brief, not from conversation history.

Start a session with `/next-slice <section>`.

## The routine

1. **Check the branch.** `git fetch` and `git status -sb`. If the branch is behind its remote, pull first
   (`git pull --ff-only`). If the working tree has uncommitted changes, settle them before starting.
2. **Read.** This README, the section's brief, the design doc sections the slice depends on (end to end, not a
   search result), `docs/DECISIONS.md` and `docs/CODING_STANDARD.md`.
3. **Take the slice marked next.** If it is blocked, say why, name a slice that could be done instead, and ask.
4. **Plan, then wait.** State which slice it is and what it will produce. Show the **sources table**: every field,
   behaviour or file the slice touches, and where each comes from:
   - an existing crate, module or type, with its path
   - a design doc section
   - an external crate (call out new dependencies)
   - "to build"

   Flag anything the docs disagree about, and anything the slice needs from another crate or would add to the
   public `starvizion-provider-api`. The user approves the sources, not the code.
5. **Build that slice and nothing else.** Out-of-scope findings go into the brief's hand-off notes, not the code.
6. **Check.** Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` and
   `cargo test --workspace`, and fix what they report. Anything needing hardware (a controller, a headset) or the
   other OS, the user checks and reports back.
7. **Close the slice.**
   - In the brief, mark it **done** with the date and add a hand-off note at the top of the slice list. The note
     says what was decided and where it is recorded, what was found, and what the next slice needs.
   - Mark the following slice **next**, and update the brief's "Built" table.
   - Check that no doc the slice touched names a crate, module, type or file that does not exist.
   - Give the user a commit message.
8. **Stop.** Name the next slice and do not start it. The next session starts fresh.

## Rules every section inherits

- `CLAUDE.md` and `docs/CODING_STANDARD.md` apply in full. `docs/DECISIONS.md` wins over any design doc that still
  assumes Elite Intel as the host.
- One slice per session.
- A decision made while building is recorded, with its date, in the **design doc it belongs to**, not only in the
  brief. An architectural decision also goes into `docs/DECISIONS.md` with who decided it. A gap the design doc
  leaves open is settled there, not silently in code.
- Tests never need a controller, a headset or a running game. Logic sits behind seams so it can be tested without
  them.
- A new dependency is proposed in the sources table and approved before it is added.
- Claude never commits, pushes or merges. The user does.
- Commit messages: a one-line summary, then short bullet points. Lines are not hard-wrapped, and the message ends
  with the attribution line Claude's instructions specify.

## Sections

In build order. A section can start once what it depends on has the slices it needs, not necessarily all of them.

| Section | Brief | Crate | Design doc | Depends on | Status |
|---|---|---|---|---|---|
| StarCalc | `starcalc.md` | `crates/starcalc` | `starcalc-expression-engine.md` | none | in progress |
| Input | `input.md` | `crates/input` | `vizlet-and-node-system.md` (Input Capture) | input backend decision | not started |
| Model and files | `model.md` | `crates/model` | `vizlet-and-node-system.md`, `holoframes-and-persistence.md` | StarCalc | not started |
| Render and editor | `render.md` | `crates/render`, `crates/starvizion` | `ui-layout.md`, `vizlet-and-node-system.md` | UI toolkit decision, Model | not started |
| VR | `vr.md` | `crates/vr` | `vr-overlay.md` | Render, VR runtime decision | not started |
| Plugins | `plugins.md` | `crates/provider-api` | `DECISIONS.md` D8, `plugin-development.md` | Model; starts with the D8 spike | not started |
| Elite Intel plugin | `elite-intel.md` | `plugins/elite-intel` | `DECISIONS.md` D4, D7 | Plugins, Elite Intel 2.0 instrumentation | not started |
| Packaging | `packaging.md` | workspace | `DECISIONS.md` D6 | a runnable app | not started |

When a brief is written, set its status here to "in progress". When its last slice is done, set it to "done".

## Writing a new brief

Writing a brief is a slice of its own: read the section's design doc end to end, write the brief and stop. Create
`docs/build-plan/<section>.md` with these parts, in this order:

1. **Goal.** What the section delivers, in a few lines, in the product's vocabulary.
2. **Sources.** The design doc sections and `DECISIONS.md` entries the section rests on.
3. **Built.** A table of what exists: path, what it is, and the slice that made it. It starts empty or lists the
   scaffold.
4. **Open questions.** Gaps or contradictions in the design docs that a slice must settle, each naming the slice
   that will settle it. Settled answers move into the design doc, and the question is removed here.
5. **Slices.** Numbered, in build order, with hand-off notes at the top, newest first. Each slice has:
   - a status: **next**, **todo**, **blocked** (with what blocks it) or **done** (with the date)
   - what it produces, in one or two lines
   - how it is known to be done: the tests or behaviour that show it
   - what it depends on, if anything beyond the previous slice

   A slice should fit one session. If it cannot be described in a few lines, split it. Exactly one slice is
   **next** at a time.
6. **Out of scope.** What belongs to another section or to the roadmap, so slices do not drift into it.

Then update the section's row in the table above.
