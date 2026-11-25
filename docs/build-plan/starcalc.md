# StarCalc

## Goal

`crates/starcalc` parses a NeuroNode's expressions once and evaluates them every frame against input bindings, window
variables, time variables, locals and, for a Parametric primitive, the sweep parameter `t`. It reports invalid
expressions precisely enough for the editor to flag them inline. A failure in one expression never affects another.
StarCalc knows nothing about windows, devices, files or any game.

## Sources

- `docs/starcalc-expression-engine.md`, all of it: syntax, Language Rules (precedence, types, names, runtime
  failures), locals and evaluation order, Parametric evaluation, error handling, the variable contract, helper
  functions and `smooth()` behaviour, reserved names, constants, color values, text values.
- `docs/vizlet-and-node-system.md`: NeuroNode modes (Live and Static), Binding a NeuroNode, Parametric Primitive.
- `docs/ui-layout.md`, Vizlet Editor: every field is live, locals are listed above properties, and the expanded
  editor shows the available variables, functions and a live value.
- `docs/CODING_STANDARD.md`, Concurrency and the Render Loop: parse once, evaluate every frame, no per-frame heap
  allocation or parsing.
- `docs/DECISIONS.md` D7 (StarCalc knows no game; provider fields are bound like any other input), D9 (no
  compatibility obligations before 1.0), D10 (no localization; numbers are formatted US-style).

## Built

| Path | What it is | Slice |
|---|---|---|
| `crates/starcalc/Cargo.toml` | Crate manifest, no dependencies | scaffold |
| `crates/starcalc/src/lib.rs` | Crate doc comment only | scaffold |

## Open questions

Gaps the design doc still leaves. Each is settled in the slice named, and the answer goes into
`docs/starcalc-expression-engine.md` with its date before the question is removed here.

1. **Lexical details.** Number literal forms (`1`, `1.5`, `.5`, `1e3`?), identifier characters, and whether
   comments exist. Follow the C family (Language Rules). Settled in slice 1.
2. **`smooth()` time constant.** The exact mapping from `speed` 0.0-10.0 to a time constant. The rest of its
   behaviour is settled in the design doc. Settled in slice 4.

Settled 2026-11-24 and recorded in the design doc: precedence and associativity, case sensitivity, bare color
literals, strict typing, strict name rules, runtime failures, `%` and `round`, `smooth()` state, the fallback for
failed expressions, and text values.

## Slices

Hand-off notes, newest first:

- 2026-11-24: the user settled the open questions on precedence, types, names, failures, `smooth()`, fallback and
  text. All are recorded in `docs/starcalc-expression-engine.md` (Language Rules, Error Handling, Helper
  Functions, Color Values, Text Values), along with D9 and D10. Slice 7 is no longer blocked.

### 1. Lexer and parser: **next**

Produces: source text to a syntax tree. Covers number and boolean literals, bare color literals, quoted text
literals, identifiers, all operators from the doc, the ternary, parentheses and function calls, with the
precedence table in Language Rules. Parse errors carry a byte span and a message the editor can show inline.
Typed errors, `#![forbid(unsafe_code)]`.

Done when: unit tests cover every row of the precedence table, `-2^2`, every literal form and each kind of parse
error with its span. Open question 1 is settled in the design doc.

### 2. Validation and compilation: todo

Produces: a syntax tree, checked against a context description, becomes a compiled expression with a known result
type (number, boolean, color, text). The context is the node's bindings with their kinds, the locals defined so
far, window and time variables, and `t` when the expression is a Parametric `x`/`y`. Errors with spans for the
type and name rules in Language Rules: unknown names, unbound variables, reserved or function names used as
binding names, duplicate names, wrong function arity and type mismatches.

Done when: tests cover each error kind, the reserved-name list and every built-in's arity.

### 3. Evaluation: todo

Produces: a compiled expression evaluated against a frame of variable values, with every built-in function and
constant from the doc except `smooth()` and `format()`. Evaluation does no heap allocation and no parsing. The
runtime failures in Language Rules (division by zero, `% 0`, infinite or NaN results) are a typed error, not a
panic.

Done when: tests cover every function and operator against known values, including `%` with negative operands and
`round` on halves, and each runtime failure case. A test or benchmark shows repeated evaluation does not allocate.

### 4. `smooth()`: todo

Produces: `smooth(value, speed)` as the design doc's Helper Functions describe it: one state slot per call site,
fixed at compile time, driven by `deltaTime`.

Done when: tests feed known values over simulated frames and check speed 0.0 (no smoothing), a middle value and
10.0, the first-frame value, that 90 Hz and 144 Hz converge alike, and that call sites are independent. Open
question 2 is settled in the design doc.

### 5. NeuroNode evaluation: todo

Produces: evaluating one NeuroNode's expression set in the documented order: locals top to bottom (each sees the
ones above it), then property expressions. Failures follow Error Handling: last good value, else the fixed
default, dependents of a failed local fail too, the node is flagged, and the error is logged when it starts, not
on every frame. The caller always receives plain values.

Done when: tests cover ordering, a local referring to a later local (an error), a failing local and its
dependents, last-good-value and default fallback, and failure isolation. Needs the workspace logging facade: if
`docs/DECISIONS.md` has none yet, this slice proposes one there for the user to decide.

### 6. Parametric sweep: todo

Produces: evaluating a Parametric primitive's `x(t)` and `y(t)` across a start value, an end value and a number
of evenly spaced steps, with all other variables held constant within the frame. Output goes into a caller-owned
point buffer, so the sweep does not allocate every frame.

Done when: tests check a known shape (a circle), the step count including both ends, and that input variables
stay fixed across steps.

### 7. Text values: todo

Produces: text values as the design doc's Text Values describes them: quoted literals, `+` joining two texts, and
`format(number, decimals)` with US formatting. Kept off the per-frame allocation path where possible, for example
by writing into a buffer the NeuroNode owns.

Done when: tests cover `format()` rounding and decimal places, joining, a text type mismatch (`"a" + 1`), and
the `""` fallback for a text property that has never succeeded.

## Out of scope

- **Storing expressions in Vizlet files.** That belongs to the Model and files section. StarCalc takes source text
  and returns compiled expressions and errors. It reads no files.
- **Provider fields as variables.** They arrive through bindings like any input (D7). Their value types come from
  the Plugins section. StarCalc handles them once they map to number, boolean, color or text.
- **Vizlet-wide shared locals.** On the roadmap, undecided (`docs/roadmap.md`).
- **Editor surfaces.** The inline error indicator, variable and function lists and live value readout belong to
  the Render and editor section. StarCalc gives them spans, errors and values. A catalogue of functions for the
  expanded editor is added when the editor needs it.
- **Hat expansion.** The input layer turns a hat into per-direction booleans. StarCalc only sees booleans.
- **Localization.** Not in the first release (D10).
