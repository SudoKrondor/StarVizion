# CODING_STANDARD.md

Coding standard for StarVizion (Rust 2024, cargo workspace, Windows and Linux, a real-time render loop for the
desktop and VR). The hard rules in `CLAUDE.md` (read-only towards games, standalone first, the public provider API,
no sandboxed packaging) and the decisions in `docs/DECISIONS.md` take precedence where they overlap.

## Design Principles

### Design Priorities

- Clarity > Cleverness
- Simplicity > Flexibility
- Current needs > Future possibilities
- Explicit > Implicit
- Low cognitive load > Clever or messy solutions

### Design Rules

- Single responsibility per unit (function, type, module, crate).
- Keep concerns separated; do not mix unrelated responsibilities in one module. The crate split is the first line
  of this: `model` knows nothing about rendering, devices or VR; `render` evaluates nothing itself; `starcalc`
  knows nothing about windows. A change that makes one crate reach into another's concern is a design change.
- Favour extension without modification (Open/Closed) where it does not add speculative abstraction.
- Depend on traits at crate seams where there is more than one implementation (per platform, per VR runtime, real
  versus test). Do not introduce a trait for a single implementation "for testability" when a plain function or
  struct can be tested directly.
- Implementations of a trait must honour its documented contract (Liskov). A trait's doc comment states that
  contract.
- Prefer small, specific traits over broad ones (Interface Segregation).
- Weigh long-term maintainability in design decisions.

### Design Anti-Patterns

DO NOT:

- Mix multiple responsibilities in one unit.
- Create tight coupling between crates. Dependencies point one way, towards `model` and `provider-api`; never add
  a dependency cycle or a dependency that runs against the layering in `README.md`.
- Introduce hidden dependencies, implicit behaviour, or implicit ordering between components (global mutable state,
  initialisation that must happen "first" without the type system enforcing it).
- Leak implementation details of a library or platform through a public interface. Callers must not need to know
  whether input came from evdev, XInput or SDL, or whether frames go to OpenVR or OpenXR. Do not expose a
  dependency's types in a crate's public API unless that is a deliberate, documented choice.
- Handle the same concept (value type, missing-value representation, units, naming) differently across code
  paths. New code matches how the rest of the system already handles it.
- Add a second way to do something the codebase already does one way. Established patterns in neighbouring code
  (how errors are defined, how state is shared between threads, how files are read and written, how platform code
  is split, how logging is done, how tests are laid out) are binding even where this document does not name them.
  Check the sibling modules and crates before inventing a variant.

## Robustness

This project favours **graceful degradation for intentional, documented cases** and **fail-fast for genuinely
unexpected states**. The two are not in tension when applied deliberately.

- **Fail fast on the unexpected.** When an invariant is violated or input is impossible per the design, return an
  error with a clear, actionable message, or panic if it can only be a bug in our own code. Do not limp along with a
  guessed default.
- **Graceful degradation is allowed only as a deliberate product decision, and must be documented with a
  `// WHY:` comment.** Expected examples: StarVizion runs without a VR runtime and shows desktop overlays only; a
  controller unplugged mid-session shows as disconnected rather than stopping the app; a provider that disappears
  leaves its bound fields empty; a Vizlet's missing font falls back to the bundled font. These are features, not
  masked bugs.
- **Never use a fallback to hide a real failure.** `unwrap_or_default()`, `.ok()`, `let _ =` or `if let Ok(..)`
  that throws an error away to make a symptom disappear is a bug, not degradation.
- **Make illegal states unrepresentable:** enums with data over flag soup, newtypes over bare primitives where
  units or identity matter (a device identity is not just a `u32`), private fields with constructors that reject
  invalid input, `NonZero*`, and parsing into typed structs at the boundary rather than validating later.
- Deserialize into typed structs (serde) rather than passing `serde_json::Value` or `HashMap<String, _>` around.
  Direct construction surfaces typos and type errors at compile time.
- Removing a validation guard requires replacing it with an equivalent (a constructor check, a type constraint, a
  `TryFrom` impl).

## Simplicity

### Quality Standards

- KISS, YAGNI, DRY.

### Simplicity Rules

- Less code is better than more code.
- Reach for the simplest solution that fully solves the current problem.
- Every type, field, parameter, generic and feature flag must earn its place. Remove unused ones rather than
  leaving them for a later caller that may never arrive.
- Prefer concrete types and plain functions. Reach for generics, trait objects, macros or lifetimes on structs
  when the problem needs them, not to anticipate one.

### Simplicity Anti-Patterns

DO NOT:

- Add "just in case" features, cargo features or speculative configuration (YAGNI).
- Implement future requirements that were not asked for (YAGNI).
- Create abstractions with a single caller and no second use in sight (YAGNI).
- Add helper functions, parameters or `pub` items that were not requested or needed.
- Introduce code, crates or approaches that were not asked for.
- Change existing design unless that is the task.
- Leave obsolete code behind. Remove it; do not comment it out or hide it behind `#[allow(dead_code)]`.

## Code Structure

### Structure Rules

- Use guard clauses and early returns to reduce nesting: handle invalid cases first (`let ... else`, `?`,
  `return`, `continue`), keep the happy path unindented.
- **Aim for functions under ~40 lines.** This is a guideline, not a hard gate: a single cohesive operation (a tight
  polling loop, one large `match` over an enum) may run longer when splitting it would obscure the flow, but it must
  carry a `// WHY:` explaining the cohesion. Default to smaller.
- Bind variables once with their final value (`let x = if .. { a } else { b };`, `match` as an expression) rather
  than declaring `let mut` and assigning across branches.
- Keep cyclomatic complexity low; prefer several small functions over one branching giant.
- Names of variables, types, functions and modules must be self-explanatory without reading the surrounding code.
  Follow Rust naming conventions (RFC 430): `snake_case` items, `UpperCamelCase` types, `SCREAMING_SNAKE_CASE`
  constants, no `get_` prefix on getters.
- Use the product's own vocabulary as it is defined in the docs: Vizlet, NeuroNode, HoloFrame, StarCalc, provider.
  Do not invent synonyms.
- Domain-layer names (`model`, `starcalc`, `provider-api`) must not reference a specific game, platform, VR
  runtime or vendor. Game-specific code lives in a plugin.
- Keep visibility as narrow as possible: private by default, `pub(crate)` before `pub`. A `pub` item in a library
  crate is API.
- Platform-specific code lives behind `#[cfg(target_os = "...")]` in its own module per platform, behind one
  shared interface. Do not scatter `cfg` attributes through shared logic.
- Code is read far more than written. Optimize for the reader.
- Place new files in the crate and module that match their responsibility; see Testing for where tests go.

Guard-clause pattern:

```rust
// BAD - nested
fn handle(item: &Item, store: &mut Store) {
    if !item.is_ignored() {
        if !store.contains(item) {
            process(item);
            store.save(item);
        }
    }
}

// GOOD - guard clauses, happy path unindented
fn handle(item: &Item, store: &mut Store) {
    if item.is_ignored() {
        return;
    }
    if store.contains(item) {
        return;
    }
    process(item);
    store.save(item);
}
```

### Top-Down Organization

- In a module, the public entry points and highest-level functions appear first.
- Callers are ordered before callees so the file reads top to bottom as a flow.
- Orchestration before implementation detail. A type's definition comes before its `impl` blocks; private helpers
  and `#[cfg(test)] mod tests` come last.

### Comments

- Most code should be self-explanatory through good names. Refactor for clarity before reaching for a comment.
- Only these comment kinds are encouraged:
    - Doc comments (`///` on items, `//!` on modules and crates): explain purpose and contract when not obvious from
      the signature. Every `pub` item of `starvizion-provider-api` has one, with `# Errors` and `# Panics`
      sections where they apply.
    - `// WHY: <reason>`: explain the rationale when it is not obvious from the code.
    - `// SAFETY: <reason>`: required on every `unsafe` block, stating why its invariants hold (see Unsafe Code).
- Comments that restate WHAT the code does are discouraged.
- Preserve existing `// WHY:` and `// SAFETY:` comments unless they become irrelevant after a change.
- When you change behaviour, update or delete any doc comments describing the old behaviour.

### Documentation

- Project docs (`CLAUDE.md`, `README.md`, this file, `docs/DECISIONS.md`, the design docs under `docs/`) are part
  of the code. A change that makes one of them wrong is an incomplete change: update the doc in the same change
  set.
- A rule or description that names a crate, module, type, file or setting must name one that exists. A stale name
  is worse than no name, because it sends the reader looking for something that is not there.
- Docs must not contradict each other or the implementation. Where two docs overlap, `CLAUDE.md` and
  `docs/DECISIONS.md` win, and the other is corrected to match. Design docs carried over from Elite Intel that
  still assume Elite Intel as the host are corrected when touched.
- A new architectural decision goes into `docs/DECISIONS.md`, with who decided it and when.

## Error Handling

- Never silently skip or ignore errors, duplicates, or constraint violations. A discarded `Result` (`let _ = ..`,
  `.ok()`) needs a `// WHY:`.
- Library crates return typed errors: an error enum per crate or per module, implementing `std::error::Error`,
  that callers can match on. Do not return `String` or `Box<dyn Error>` from a library crate's public API.
- The application binary may aggregate errors at its top level, adding context about what it was doing when the
  error happened.
- Errors carry enough context to act on: which file, which device, which provider, which expression.
- `unwrap()` and `expect()` are for invariants that can only fail through a bug in our own code, and the
  `expect` message says what invariant was broken. They are never for I/O, parsing, user input, device or provider
  data, or anything else the outside world controls. `unwrap()` is fine in tests.
- Do not let a panic in one component take the app down where the design says it should degrade: a provider
  thread or a plugin is a deliberate boundary where a failure is caught, logged and contained. Comment why the
  boundary is broad.
- Log through the workspace's chosen logging facade, never `println!`/`eprintln!` outside `main` and tests.
  Choosing that facade is an architectural decision for `docs/DECISIONS.md`.
- Never log or transmit secrets, and never log keystrokes: keyboard capture is opt-in and keylogger-shaped
  (D5), so key events must not reach a log, a support bundle or a provider unless the user bound them on purpose.

## Unsafe Code

StarVizion talks to Win32, X11/Wayland, evdev and VR runtimes, so some `unsafe` is unavoidable. Keep it small and
auditable.

- No `unsafe` outside the modules that wrap FFI or platform APIs. Shared logic and the domain crates (`model`,
  `starcalc`, `provider-api`) contain none; mark them `#![forbid(unsafe_code)]`.
- Wrap each unsafe API in a safe function or type whose signature makes misuse impossible, and keep the `unsafe`
  block as small as the call that needs it.
- Every `unsafe` block carries a `// SAFETY:` comment; every `unsafe fn` documents its contract under `# Safety`.
- Prefer a maintained safe wrapper crate over hand-written bindings, subject to Dependencies & Libraries.

## Concurrency and the Render Loop

The render loop runs at the headset's refresh rate (90-144 Hz) and VR submission happens every frame. A missed
frame is visible to the pilot.

- Never block the render thread: no file I/O, no network or IPC waits, no lock held by a thread that does any of
  those. Input polling, provider connections and file loading happen elsewhere and hand over results.
- Hand data between threads through channels or a clearly owned shared state; document who owns it. Do not hold a
  lock across a call into another component.
- Avoid per-frame heap allocation and per-frame parsing on the hot path. StarCalc expressions are parsed once and
  evaluated every frame; keep it that way.
- Measure before optimizing anything else.

## Testing

Tests run with `cargo test --workspace`. Unit tests live next to the code in a `#[cfg(test)] mod tests` at the
bottom of the file; tests that exercise a crate only through its public API live in that crate's `tests/`
directory.

- Add tests for new logic.
- Unit tests sit in the module they test; integration tests in `tests/` are named after the behaviour they cover.
- Tests follow the same standards as production code.
- Avoid test duplication: if a case is a strict subset of a broader test, drop the redundant one.
- Design for testability: pure functions, plain data in and out, and platform, device and VR access behind a seam
  so logic can be tested without hardware. Tests must not need a controller, a headset or a running game.
- Doc examples on public API (`provider-api` especially) compile and run as doctests.

### Testing Anti-Patterns

DO NOT:

- Remove, `#[ignore]` or comment out still-relevant tests instead of fixing them. When a responsibility moves,
  move its test to follow it (do not just delete the assertion).
- Write tests for states the type system makes impossible.
- Write tests that depend on timing, real devices or the machine they run on without marking them `#[ignore]`
  with a reason.

## Dependencies & Libraries

- Keep dependencies minimal; each one is compile time, audit surface and a licence to check (the project is MIT).
- Declare shared dependency versions once in the root `[workspace.dependencies]` and reference them with
  `.workspace = true` from member crates.
- Investigate a crate's intended usage before abandoning it over an initial snag.
- When adding a dependency, state it explicitly and call it out for review, with why it is needed and what it
  pulls in. Disable default features that are not used.
- `Cargo.lock` is committed.

## API & Compatibility

- `starvizion-provider-api` is public: third-party developers build against it. Change it only compatibly
  (semver: additive changes in a minor version, breaking changes need explicit approval and a major version), and
  version it. Mark public enums and structs that may grow `#[non_exhaustive]`.
- The provider wire protocol, once it exists, follows the same rule: an additive field changes every message that
  carries it, so call it out and check the consumers.
- Vizlet and HoloFrame files (`.vizlet.json`, `.holoframe.json`) are user data. Never break loading of a file an
  earlier release wrote; a format change carries a version and a migration from the previous one. This applies
  from 1.0 on: before it, formats change freely (D9 in `docs/DECISIONS.md`).
- Work branches off `main` and merges back into it through a pull request.

## Security

- Consider security implications of changes.
- Honour the hard constraint: StarVizion is read-only towards games. Never send, intercept or remap input, never
  write a game file, never read game memory or inject into a process. Game data comes only from providers.
- Global input capture (keyboard especially) is privileged access: keep it opt-in, off by default, and explain it
  to the user where it is turned on.
- Treat everything a provider sends as untrusted input: validate it at the boundary, bound its size and rate, and
  never let it crash the app or block the render loop.

## Linting and Formatting

- Code is formatted with `cargo fmt --all` and passes `cargo clippy --workspace --all-targets -- -D warnings`, as
  CI runs it.
- Fix the cause of a warning. An `#[allow(..)]` (prefer `#[expect(..)]`) is acceptable only when the lint is wrong
  for that spot, scoped to the smallest item, with a `// WHY:`.

## AI Assistant Behavior

- You may suggest new design or crates, but get confirmation before applying them.

### Starting Tasks

- Seek clarity on requirements before coding; ask rather than assume.
- Understand the current architecture and identify the files to change. Read `README.md`, `docs/DECISIONS.md`
  and the design doc for the area first.
- Break large or vague tasks into smaller steps, or ask for clarification.
- Read entire files before editing to understand context and avoid duplication.

### Working with Libraries

- Look up current syntax and usage when unsure (docs.rs, the crate's own docs).
- You may suggest a better crate, but if the user keeps theirs, use it. Do not swap a crate without approval.

### Handling Violations

1. Name the specific principle breached.
2. Explain the violation in plain English.
3. Offer the simplest correction.

### Continuous Focus

- Watch for and correct scope creep and unnecessary complexity.

### Quality Assurance

- Implement real solutions, not stubs. No `todo!()` or `unimplemented!()` left in a finished change.
- Do not perform large refactors unless explicitly instructed.
- Find the root cause of repeated issues instead of trying random fixes.
- Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`
  after a change, not just the tests you added.

### Linting Anti-Patterns

DO NOT:

- Suppress warnings with `#[allow(..)]`, crate-level lint overrides or by loosening project settings instead of
  fixing the cause.
- Formally bypass a problem rather than fixing it (e.g. discarding a `Result` to silence `unused_must_use`, adding
  `unwrap()` to get past a type error, cloning to silence the borrow checker without understanding why it
  complained, or shortening a function by extracting code without regard to responsibility).
