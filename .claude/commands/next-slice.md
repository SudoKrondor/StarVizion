---
description: Start the next StarVizion build slice from its brief in docs/build-plan/
argument-hint: "<section> - e.g. input, model, starcalc, render, vr, provider-api, editor"
---

Section: $ARGUMENTS

You are building one slice of StarVizion. The plan and the routine are in `docs/build-plan/`, and they replace the
conversation history a long session would have carried, so follow them rather than improvising.

# 1. Before anything else

- Run `git fetch` and `git status -sb`. If the branch is **behind** its remote, stop and ask the user to pull
  (`git pull --ff-only`). Building on a stale branch means a rebase later.
- If the working tree has uncommitted changes, say what they are and ask before going on.
- If `docs/build-plan/README.md` does not exist yet, say so and offer to write it (the routine, the rules every
  section inherits, the list of sections, and "Writing a new brief"). Writing it is then this session's slice:
  stop when it is written.

# 2. Read

1. `docs/build-plan/README.md`: the routine and the rules every section inherits.
2. `docs/build-plan/<section>.md` for the section named above. If none was given, list the sections from the
   README and ask which.
    - **If the brief does not exist yet**, writing it is this session's slice. Follow "Writing a new brief" in the
      README, read the section's design doc end to end to do it, and stop when the brief is written.
3. Take the slice marked **next**. If it is **blocked**, say why and which slice could be done instead, and ask.
4. **Read the design doc sections that slice depends on, end to end** (not a search result, not the line you
   think you need): the relevant file under `docs/`, plus `docs/DECISIONS.md`, which wins where a design doc still
   assumes Elite Intel as the host. Also read `docs/CODING_STANDARD.md`.

# 3. Plan, then wait

Tell the user in a few lines which slice this is and what it will produce. Then show the **sources table**: every
field, behaviour or file the slice touches, and where each comes from: an existing crate, module or type (with its
path), a design doc section, an external crate (new dependencies called out), or "to build". Flag anything the docs
disagree about, and anything the slice would need from another crate or would add to the public
`starvizion-provider-api`.

**Wait for the user's approval.** They approve the sources, not the code.

# 4. Build

- That slice and nothing else. Something out of scope goes into the brief's hand-off notes, not the code.
- A decision made while building is recorded in the **design doc** it belongs to, with a date (and in
  `docs/DECISIONS.md` if it is architectural, with who decided it), not only in the brief.
- When the code and its tests are written, run:
    - `cargo fmt --all`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test --workspace`

  Fix what they report. Anything that needs hardware (a controller, a headset) or the other OS, ask the user to
  check and report back.

# 5. When the checks pass (and the user confirms anything that needed hardware)

1. Update `docs/build-plan/<section>.md`: mark the slice **done** with the date, add a hand-off note at the top of
   that list (what was decided and where it is recorded, what was found, what the next slice needs), and mark the
   following slice **next**. Fix the brief's "Built" table if it changed.
2. Check that no doc the slice touched now names a crate, module, type or file that does not exist.
3. Give the user a commit message: a one-line summary, then short bullet points, lines not hard-wrapped, ending
   with the attribution line your instructions specify.
4. **Never commit, push or merge.** The user does all of that.

# 6. Stop

Say the slice is finished and name the next one. **Do not start it in this session**, even with room to spare:
the next session starts fresh, and that is the point of working this way.
