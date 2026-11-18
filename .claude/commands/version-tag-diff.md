---
description: Draft plain-English release notes for users from the changes since the last release tag, written to version-diff.txt
argument-hint: "[previous tag, e.g. v0.1.0 - defaults to the latest tag]"
---

Optional input: $ARGUMENTS

Compare the current code line to the previous release tag and draft release notes for users. The draft is for the
user to work with: they copy the parts where they need to go.

Do NOT stage, commit or tag anything; just write the draft to a file and print it. The user commits and tags
manually.

# 1. Gather the change set

The version lives in the root `Cargo.toml` under `[workspace.package] version`, and every crate inherits it.
Between releases it reads `X.Y.Z-dev` (the version being worked towards); a release is tagged `vX.Y.Z`.

- The previous release is the tag given in the arguments, or else the latest release tag:
  `git describe --tags --abbrev=0 --match 'v*'`.
- If there is no release tag yet, say so, and use the first commit as the base.
- Check that the workspace version is ahead of the previous tag. If it is not (for example it still reads the
  released version, or lacks `-dev` after a release), point it out.
- The change set is `git log <tag>..HEAD` and `git diff <tag>...HEAD`. Read the diffs, not just the commit
  messages: commit messages can be incomplete.

# 2. Understand what changed (functionally)

Work out, from the diffs, what actually changed in terms of:

- Functionality / behaviour (new features, changed behaviour, bug fixes, removed behaviour).
- Anything user-visible (overlays, the Vizlet editor, VR behaviour, devices, settings, installers, supported
  platforms).
- Changes that ask something of the user: a new permission (for example evdev access on Linux), a settings or file
  format migration, a dropped platform or VR runtime.
- For plugin developers: changes to `starvizion-provider-api` and its version.

Group related changes into themes. A theme spans whatever files implement it; do not organise by file. Leave out
internal changes users cannot notice (refactors, tests, CI) unless they changed behaviour.

# 3. Write the release notes

This is a record of work already done, not a plan. Every bullet describes something that now exists in the diff,
never something that should happen next.

Format:

- **Summary line**: past tense, concise (aim <= 72 chars), naming the new version.
- Blank line.
- **Body**: bullet points grouped by theme, past tense throughout (e.g. "Added", "Fixed", "Reworked"), in plain
  English for pilots. Say what they can now do or what works differently, not how it was built.
- If anything needs the user to act after upgrading, put it first under its own heading.
- If the provider API changed, add a short separate section for plugin developers, saying whether the change is
  compatible.
- Use plain hyphens, never em dashes.
- Do not hard-wrap lines.
- Keep it factual and end-user friendly.

Example shape (illustrative, not a template to copy verbatim):

```
---

- Item
- Item
...
- Item

---
```

# 4. Output

- Write the notes to `version-diff.txt` in the repo root (always overwrite it) and print them inline.
- Do not run `git add`, `git commit` or `git tag`. Mention that the draft is ready for the user to review.
