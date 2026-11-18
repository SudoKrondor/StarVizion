---
description: Review a change set (working tree, branch, or GitHub PR) against docs/CODING_STANDARD.md
argument-hint: "[PR number/URL | branch/ref | nothing for local changes] [--inline]"
---

Change set to review: $ARGUMENTS

Review the code for integrity and compliance with `@docs/CODING_STANDARD.md`. The hard rules in `@CLAUDE.md` and
the decisions in `@docs/DECISIONS.md` take precedence where they overlap.

# 1. Determine the change set and scope

**Base branch.** `main` is the trunk: work branches off `main` and merges back into it through a pull request.

Pick the mode from the argument:

- **No argument** (default): review local work.
    - If on a feature branch: the change set is the uncommitted working tree plus `git diff main...HEAD`.
    - If on `main` itself: review the uncommitted working tree (and any local commits not yet pushed, i.e.
      `git diff origin/main...HEAD`).
    - Use `git status` and `git diff` for the working tree, and read untracked files in full.
- **A branch or ref**: the change set is `git diff main...<ref>`.
- **A PR number or URL**: review the GitHub PR.
    - Confirm the PR reference resolves. Read its base branch with `gh pr view <pr> --json baseRefName` and use
      THAT as the base (do not assume `main`).
    - Check CI status with `gh pr checks`.
    - Check whether the base branch is merged into the PR branch; if not, update it via
      `gh api repos/{owner}/{repo}/pulls/{pull_number}/update-branch -X PUT` (this preserves approvals, unlike a
      CLI merge). If the update fails (conflicts, permissions), STOP and report.
    - **If CI is failing, the PR does not resolve, or the branch cannot be updated: STOP, report the issue, and do
      not proceed.**
    - Use `gh pr diff` for the diff. Check out the PR branch so reported line numbers match the files.

Scope to the workspace crates (`crates/`, `plugins/`) and their manifests (`Cargo.toml`, `Cargo.lock`). If the
change set includes docs (`docs/`, `README.md`, `CLAUDE.md`), review them for internal contradiction, ambiguity,
stale names, and divergence from the actual implementation. Verify new files sit in the crate and module that
match their responsibility.

# 2. Mechanical checks

Run these on the checked-out change set and report the result of each (pass, or the failures):

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

A failure here is a finding. Do not fix anything; this command reviews.

# 3. Overview

- If reviewing commits or a PR, list the authors of the changes in the set.
- Give a brief big-picture summary of what changed and the common theme(s).

# 4. Analysis criteria

Enforce every applicable rule in `docs/CODING_STANDARD.md`. Before judging, sample neighbouring code (same
module, sibling modules and crates) and infer the de facto conventions of this codebase (how errors are defined,
how state crosses threads, how platform code is split, how files are read and written, how logging is done, how
tests are structured). Flag any change that contradicts these patterns, and say explicitly when the rule is
implicit, citing 1-2 example files where the existing pattern is visible. While the codebase is young and has few
examples, say so rather than inventing a convention.

Additionally review for:

- Logic, coherence, and flow.
- Style, clarity, conciseness, readability.
- Duplication and long-term maintainability; obsolete or dead code left behind.
- Names that are unclear or ambiguous without surrounding context, or that drift from the product vocabulary
  (Vizlet, NeuroNode, HoloFrame, StarCalc, provider).
- Crate layering: a dependency that runs against the layering in `README.md`, a domain crate (`model`,
  `starcalc`, `provider-api`) that gains platform, vendor or game-specific knowledge, a dependency's types leaking
  into a public API.
- Hard rules from `CLAUDE.md`: anything that sends, intercepts or remaps input, writes a game file, reads game
  memory or touches a game process; anything a pilot needs for controller visualization that now depends on a
  plugin or on Elite Intel; sandboxed packaging.
- Public API: any change to `starvizion-provider-api` that is not semver-compatible, a missing version bump, a
  new public type that should be `#[non_exhaustive]`, a public item without a doc comment.
- User data: a change to the `.vizlet.json` / `.holoframe.json` format that breaks loading of files an earlier
  release wrote, or lacks a version and migration.
- Security: keyboard capture not opt-in, key events reaching logs or providers, provider input trusted without
  validation or bounds.
- Unsafe code: `unsafe` outside an FFI or platform module, a missing `// SAFETY:` comment or `# Safety` section, an
  unsafe block larger than the call that needs it, a domain crate without `#![forbid(unsafe_code)]`.
- Performance: blocking work, locks held across calls, per-frame allocation or parsing on the render thread;
  premature optimization elsewhere.
- Error handling: `unwrap()`/`expect()` on data the outside world controls, discarded `Result`s, `String` or
  `Box<dyn Error>` returned from a library crate, missing context, fallbacks that hide real bugs versus
  intentional `// WHY:`-documented degradation.
- Robustness: fail-fast on unexpected states; illegal states that the types could rule out but do not.
- Abstraction quality: traits with one implementation and no second in sight, unused params, fields, generics or
  cargo features, visibility wider than needed.
- Consistency: the same concept handled identically across all code paths that touch it.
- Testing: missing tests for new logic; disabled, `#[ignore]`d or removed still-relevant tests; an assertion
  deleted rather than moved when a responsibility moved; tests for impossible states; tests that need hardware.
- Dependencies: state every added or changed dependency in any `Cargo.toml`, whether it goes through
  `[workspace.dependencies]`, and what it pulls in.
- Linting: list any `#[allow(..)]`, `#[expect(..)]`, crate-level lint change or CI/config change used to bypass
  rather than fix.
- Docs: a change that makes `CLAUDE.md`, `README.md`, `docs/DECISIONS.md` or a design doc wrong without updating
  it; a new architectural decision not recorded in `docs/DECISIONS.md`.

# 5. Reporting

Use a warm, constructive tone that fosters trust and psychological safety, focused on shared goals (reliability,
maintainability) rather than personal preference. Tone must not reduce thoroughness; findings must be precise and
complete.

- Do not use em dashes; use a plain hyphen or rephrase.
- Start with the result of the mechanical checks.
- Number findings starting from 1. For each: a brief plain-English explanation of the problem, the implication,
  and the simplest fix, prefixed with `file:line`.
- Tag each finding with one or more labels: `logic`, `design`, `clarity`, `readability`, `duplication`,
  `convention` (implicit/undocumented), `robustness`, `error-handling`, `testing`, `security`, `performance`,
  `api`, `unsafe`, `docs`. Extend as needed.
- Attribution: report issues introduced by this change set first; list pre-existing related issues separately
  afterward.

Output: by default, write the full report to `code-integrity-review.md` in the repo root and print a short
summary inline. If `--inline` is in the arguments, print the full findings inline and do NOT write a file (use
this for quick iterations on small local change sets).
