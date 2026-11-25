# StarVizion — StarCalc Expression Engine

StarCalc is StarVizion's built-in expression evaluation system. It lets users write simple formulas that combine input values and constants to produce derived outputs — average two axes into a composite thrust level, compute a ratio, apply conditional logic, normalise or scale an axis range.

## Expression Syntax

StarCalc expressions are evaluated per frame. Supported constructs:

- **Arithmetic:** `+`, `-`, `*`, `/`, `%`, `^` (power)
- **Comparison:** `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logic:** `&&`, `||`, `!`
- **Ternary:** `condition ? valueIfTrue : valueIfFalse`
- **Built-in functions:** `sin()`, `cos()`, `tan()`, `abs()`, `min()`, `max()`, `clamp()`, `round()`, `floor()`, `ceiling()`, `sqrt()`, `ln()`, `log10()`, `atan2()`
- **Input references:** user-named variables bound to device inputs (axes as floats −1.0 to 1.0, buttons as booleans)
- **Colors:** written bare, `#RRGGBB` or `#RRGGBBAA` (see [Color Values](#color-values-in-expressions))
- **Text:** string literals in double quotes, `"Speed: "` (see [Text Values](#text-values))

## Language Rules

*Settled by Krondor, 2026-11-24.*

**Style.** StarCalc follows the C family (C, Java, Kotlin, JavaScript, C#) wherever the syntax above does not say
otherwise, so people who already write code in one of those find nothing new. Names are case-sensitive: `Throttle`
and `throttle` are different variables.

**Lexical rules** (*settled by Krondor, 2026-11-24*):
- **Numbers:** `1`, `1.5`, `.5`, `1e3`, `1.5e-3`. Not `1.`, `1_000`, `1.5.2` or a number run into a name
  (`2pi`). A minus sign is an operator, not part of the number. A number too large to represent is an error.
- **Names:** an ASCII letter or `_`, then letters, digits or `_`. `true` and `false` are the boolean literals.
  `True` is just a name.
- **Colors:** `#` and exactly 6 or 8 hex digits, in either case.
- **Text:** double quotes, with the escapes `\"`, `\\` and `\n`. A raw line break inside quotes is an error that
  says to use `\n` instead, so a Windows `\r\n` pasted into text never hides in it.
- **Whitespace:** spaces, tabs and line breaks (`\n` and `\r`) separate tokens and are otherwise ignored, so
  expressions pasted from Windows parse the same.
- **Comments:** none yet. `//` and `/*` give a "comments are not supported" error, which keeps that syntax free
  for C-style comments if they are added.
- **Nesting:** expressions nest at most 64 levels deep (parentheses, calls, conditionals, unary operators and
  exponents each add one). Deeper nesting is an error, not a crash, because Vizlet files can be imported from
  other people.
- Lone `&`, `|` and `=` are errors that suggest `&&`, `||` and `==`.

**Precedence**, tightest first:

| Level | Operators | Associativity |
|---|---|---|
| 1 | `^` (power) | right: `2^3^2` is `2^9` |
| 2 | unary `-`, `!` | right |
| 3 | `*`, `/`, `%` | left |
| 4 | `+`, `-` | left |
| 5 | `<`, `<=`, `>`, `>=` | left |
| 6 | `==`, `!=` | left |
| 7 | `&&` | left |
| 8 | `\|\|` | left |
| 9 | `? :` | right |

`^` binds tighter than unary minus, as in mathematics and Python: `-2^2` is `-4`. Parentheses override everything.

**Types must match.** StarCalc has four value types: number, boolean, color and text. Nothing converts
implicitly:
- `true + 1` is an error, and so is `"5" == 5`.
- Both branches of `? :` must have the same type, and the condition must be a boolean.
- Arithmetic takes numbers. `&&`, `||` and `!` take booleans. `<`, `<=`, `>` and `>=` compare numbers.
- `==` and `!=` compare two values of the same type.
- `+` also joins two texts. Converting a number to text is explicit: see [Text Values](#text-values).

Type errors are found as the expression is typed, not while it runs.

**Names are strict.**
- A binding or local cannot reuse a reserved name, a function name, or the name of another binding or local in
  the same NeuroNode.
- `t` exists only inside a Parametric primitive's `x` and `y` expressions.

These rules are the strict starting point and may be relaxed before 1.0 (see
[Decisions — D9](DECISIONS.md#d9-no-compatibility-obligations-before-10)).

**Runtime failures.**
- Division by zero, an infinite result or a NaN result (for example `sqrt(-1)` or `ln(0)`) fails the expression.
  Nothing infinite or NaN ever reaches a NeuroNode property.
- `%` keeps the sign of the left operand, as in the C family: `-7 % 3` is `-1`. `x % 0` fails, like division
  by zero.
- `round` sends halves away from zero: `round(2.5)` is `3`, `round(-2.5)` is `-3`.

## Local (Intermediate) Expressions

A NeuroNode can define **local expressions** — named computed values evaluated before the node's property expressions, and available within them exactly like variables. Locals let a complex calculation be written once and referenced multiple times across different properties, instead of repeating it in every property expression that needs it.

**Evaluation order within a frame:**
1. Input variables are read from the device
2. Window and time variables are updated
3. Locals are evaluated top to bottom — each local can reference locals defined above it
4. Property expressions are evaluated using all variables and locals

**Scope:** locals are per-NeuroNode. A local defined in one node is not visible to other nodes in the same Vizlet. Vizlet-wide shared locals are a planned future direction — see [Roadmap](roadmap.md#vizlet-wide-shared-locals).

**In the editor:** locals appear as an editable list above the property expressions in the property panel. Each row is a name and an expression field; rows can be added, removed, and reordered.

## Parametric Evaluation

For Parametric primitives, StarCalc is evaluated in a loop — the sweep parameter is injected at each step alongside all standard variables, while all input, window, and time variables remain constant across the steps within a single frame. This means a Parametric node's shape can be both mathematically defined and reactive to input simultaneously.

## Error Handling

An invalid expression is flagged in the editor with a visual indicator on the node. A failed expression on one node does not affect other nodes.

**At runtime** (*settled by Krondor, 2026-11-24*), a failure is handled once, where a NeuroNode's expressions are evaluated. Whatever draws a node always receives a plain value and never checks for failure itself:
- A property whose expression fails keeps its **last good value**, so the overlay does not flicker.
- If the expression has never succeeded, the property takes its fixed default value. For a text property that is `""`.
- A local that fails fails every expression that uses it, and each of those falls back the same way.
- The NeuroNode is flagged for the editor, and the error is logged when it starts, not on every frame.

## Telemetry Expressions (Future)

StarCalc expressions can reference fields announced by providers the same way they reference a controller axis — for example ship status, cargo load or current game mode from the Elite Intel plugin, or any field a third-party provider offers for another game. Which fields exist depends entirely on which providers are running; StarCalc itself knows no game.

---

## Variable Contract

Every StarCalc expression evaluates inside a context — a set of named variables and functions available at evaluation time. The editor exposes all of these to the user; expressions can reference any variable or function listed here.

### Input Variables

Input variables are per-NeuroNode and user-named — the user defines the name and maps it to a physical device input in the node's input-bindings panel. The expression can then reference that chosen name; the naming is entirely up to the user, nothing is fixed.

| Input Kind | Value Type | Range |
|---|---|---|
| Axis | float | −1.0 to 1.0 |
| Button | boolean | true / false |
| Hat switch | boolean | true / false — each hat direction is a separate button |

Hat switches are treated as individual buttons regardless of how the hardware reports them — a 4-way hat exposes four boolean bindings, an 8-way hat exposes eight. StarVizion does not treat hat switches as axes or POV values.

**"Regardless of how the hardware reports them" is doing real work here.** Confirmed 2026-09-06: the hardware
reports a hat as a single 4-bit value (`0–7`, plus null for centred), never as booleans. The expansion into
per-direction booleans is StarVizion's translation to perform.

**StarVizion and BindForge deliberately use different hat models, and that is not a defect to reconcile.**
StarVizion shows what the hardware does, so an 8-way hat gets eight booleans and a diagonal is its own
state. BindForge edits what Elite binds, and `.binds` names only four POV tokens per hat, so a diagonal is
two tokens held together. Same hardware, two correct answers, because the questions differ.

If an expression references a variable that has not been bound, it fails validation. An unbound variable is a hard error, not a silent default.

### Window Variables

Always available to every NeuroNode with no setup required — they reflect the Vizlet's current window geometry: width, height, centre X (width/2), centre Y (height/2), and aspect ratio (width/height). These let an expression like "axis value times half the window width, offset from centre" work correctly regardless of the Vizlet's actual size.

### Time Variables

Time variables enable animation — behaviour that changes over time with no input required: total elapsed seconds since the Vizlet started, and elapsed seconds since the previous frame. Both are read-only and monotonic. Example: a dot that pulses in size using a sine wave of elapsed time.

### Helper Functions

In addition to the standard math functions above, StarCalc provides:

| Function | Description |
|---|---|
| `clamp(x, min, max)` | Restrict a value to a range |
| `lerp(a, b, factor)` | Linear interpolation from a to b by the given factor |
| `map(v, inMin, inMax, outMin, outMax)` | Remap a value from one range to another — for example, translating a raw axis range into a pixel range in one call |
| `smooth(value, speed)` | Exponential smoothing that damps sudden changes over time; `speed` is a unitless strength value from 0.0 (no smoothing) to 10.0 (heavy, slow, floaty smoothing) |
| `format(number, decimals)` | Convert a number to text with a fixed number of decimal places: `format(0.756, 1)` is `"0.8"` (see [Text Values](#text-values)) |

**`smooth()` behaviour** (*settled by Krondor, 2026-11-24, leaning to performance*):
- Each `smooth()` call in an expression keeps its own state, in a slot fixed when the expression is compiled. Two calls in one expression never share it, and a frame costs no lookup.
- On its first frame it returns its input unchanged, so nothing slides in from 0.
- `speed` 0.0 returns the input unchanged. Higher values damp harder.
- The damping uses `deltaTime`, so it feels the same at 90 Hz and 144 Hz. The exact mapping from `speed` to a time constant is set when `smooth()` is built and recorded here.

### Reserved Names

`width`, `height`, `centerX`, `centerY`, `aspectRatio`, `time`, `deltaTime`, `t`, `pi`, `e`, `true`, `false` cannot be used as input-binding variable names. `t` is reserved specifically because it is injected automatically as the sweep parameter during Parametric primitive evaluation.

### Built-in Constants

`pi` and `e` are available in all expressions with no setup required.

### Color Values in Expressions

Colors are their own value type, written bare with no quotes (*settled by Krondor, 2026-11-24*): `#FF8800` or `#FF880080`. Expressions can produce a color conditionally, for example `fire ? #FF0000 : #00FF00` for a color that switches with a button state. Colors can be compared with `==` and `!=` and chosen with `? :`. They take part in no arithmetic.

### Text Values

*Settled by Krondor, 2026-11-24.*

Text properties, such as a Text primitive's content, take text values:
- **Literals** are written in double quotes: `"Speed: "`.
- **`+` joins** two texts: `"Speed: " + format(speed, 1)`.
- **Numbers become text only through `format()`.** `"Speed: " + speed` is a type error.
- **Numbers are formatted US-style** whatever the operating system's locale: `.` for decimals and no thousands separator, so `format(1234.5, 2)` is `"1234.50"`. The first release is not localized (see [Decisions — D10](DECISIONS.md#d10-the-first-release-is-not-localized)).
- **A failed text expression** shows its last good text, or `""` if it has never had one (see [Error Handling](#error-handling)). StarCalc has no null.
