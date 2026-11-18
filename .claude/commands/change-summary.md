---
description: Draft a change summary (commit/PR text plus a plain-English note for users) for the uncommitted work, written to change-summary.txt
argument-hint: "[issue key e.g. SV-12] [--staged]"
---

Optional input: $ARGUMENTS

Generate a change-summary draft for the current uncommitted work. The draft is for the user to work with: they copy
the parts where they need to go. It has two parts: the change summary for the commit or pull request, and a
plain-English note for users that goes into the release notes at the next version tag.

Do NOT stage or commit anything; just write the draft to a file and print it. The user commits manually.

# 1. Gather the change set

The change set is what differs from the last commit on the current branch:

- Default: all uncommitted changes vs `HEAD` - staged and unstaged tracked changes plus new (untracked) files. Use
  `git status --short`, `git diff HEAD`, and read untracked files' contents.
- If `--staged` is in the arguments: scope to staged changes only (`git diff --cached`).

Read the actual diffs and new-file contents so you describe behaviour, not filenames. If the diff is large, read
enough of each hunk to understand the intent; do not guess.

# 2. Understand what changed (functionally)

Work out, from the diffs, what actually changed in terms of:

- Functionality / behaviour (new features, changed behaviour, bug fixes, removed behaviour).
- Notable implementation decisions (new types or traits, data flow, threading, error handling, unsafe/FFI code).
- Public API: any change to `starvizion-provider-api`, and whether it is additive or breaking.
- User data: any change to the Vizlet or HoloFrame file format.
- Dependencies added, removed or upgraded.
- Tests added or changed.
- Anything user-visible (overlays, the Vizlet editor, VR behaviour, devices, settings, installers).

Group related changes into themes. A theme spans whatever files implement it; do not organise by file.

# 3. Write the change summary

This message is a record of work already done, not a plan. Every bullet describes something that now exists in the
diff, never something that should happen next. If a bullet could be mistaken for a to-do item or a code review
comment, rewrite it.

Format:

- **Summary line**: past tense, concise (aim <= 72 chars). If an issue key was passed in the arguments, prefix it:
  `SV-12 <summary>`.
- Blank line.
- **Body**: bullet points grouped by theme, past tense throughout (e.g. "Added", "Fixed", "Reworked", not "Add",
  "Fix", "Rework"). Each bullet states WHAT changed and, where it aids understanding, WHY or the key
  implementation choice. Describe functionality and implementation, not a list of files. It is fine to name a
  crate or area (e.g. "StarCalc parser", "evdev input", "provider API") when it helps the reader, but do not
  enumerate changed files.
- Call out a provider API change or a file-format change in its own bullet, saying whether it is compatible.
- If tests were added or updated, include a short `Tests:` line or bullet, past tense.
- Use plain hyphens, never em dashes.
- Do not hard-wrap lines.
- Keep it factual and reviewable; no filler.

# 4. Write the note for users

Plain English for pilots, not developers: what they can now do, what works differently, what was fixed. Leave out
internal changes users cannot notice. If nothing in the change set is user-visible, say so in one line instead.

```
---

- Item
- Item
...
- Item

---
```

# 5. Output

Write both parts to `change-summary.txt` in the repo root (always overwrite it) and print it inline. Do not run
`git add` or `git commit`. Mention that the draft is ready for the user to review and commit.
