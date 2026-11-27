# StarVizion — VR Overlay

StarVizion supports rendering a Vizlet to the desktop, to VR, to both at once, or to the desktop with an external tool handling VR placement. All four are v1 requirements. For the first three, StarVizion owns both output paths directly, in Rust — the desktop overlay window (`crates/render`) and the VR compositor integration (`crates/vr`). It does not render through any other application's overlay. A plan to render through Elite Intel's native overlay or Swing windows, from when StarVizion was to be an Elite Intel feature, is superseded by [Decisions — D1 and D2](DECISIONS.md).

**Why a direct VR compositor integration, rather than capturing an existing desktop overlay window into VR, for the first three modes:** a desktop-window-capture approach is workable on Windows but has no equivalent that works the same way on Linux, which would break platform parity for a Windows+Linux product. The VR industry-standard overlay API that StarVizion targets is itself genuinely cross-platform, so a direct integration gives identical behaviour on both platforms once built.

**Why a fourth, hybrid mode exists anyway:** Desktop+ is a real, existing tool some VR users already run to grab a desktop window and pin/position it precisely in VR space. Relying on it for VR placement — rather than StarVizion's own direct compositor integration — means StarVizion only ever has to render a plain desktop window for that Vizlet, with no VR-side code of its own involved at all. This is confirmed useful enough to support directly rather than leaving to chance, precedented by Elite Intel, which already shipped exactly this four-mode overlay model successfully. It's a genuine hybrid, not a replacement: the three direct-integration modes remain the only cross-platform option and stay as designed above; **Desktop+ mode is Windows-only**, since the Desktop+ tool itself is, and exists purely as an added convenience for players already using it.

## Rendering Modes

| Mode | Output Target | Platform |
|---|---|---|
| Desktop | A transparent, frameless, always-on-top window on the desktop | Windows + Linux |
| VR | An overlay texture submitted directly to the VR compositor | Windows + Linux |
| Desktop + VR | Both of the above, simultaneously, from the same Vizlet | Windows + Linux |
| Desktop+ | A plain desktop window only; VR placement is handled entirely by the external Desktop+ tool, outside StarVizion | Windows only |

A Vizlet's render mode is a per-Vizlet setting, configured on the [Vizlet Editor's Vizlet tab](ui-layout.md#vizlet-tab) under Render Mode. Different Vizlets can independently use any of the four modes — a Desktop-only Vizlet, a VR-only Vizlet, a Desktop + VR Vizlet, and a Desktop+ Vizlet can all be active at once. A [HoloFrame can optionally override Render Mode for all its members at once](holoframes-and-persistence.md#render-mode-override) — whole-group, never per-member, the same precedence pattern already used for HoloFrame visibility.

**Desktop + VR is not two separate configurations layered together** — it is one Vizlet with both a desktop window (position, size, always-on-top, transparency) and a VR world-space transform (position, width, world/HMD-locked) active and persisted at the same time. Both sets of properties are shown and editable together on the Vizlet tab whenever this mode is selected, rather than the Window and VR fields being mutually exclusive the way they are for the single-mode options.

## Rendering Pipeline

All Vizlets, regardless of output mode, use the same rendering pipeline:

1. StarCalc evaluates all node expressions for the current frame.
2. Each NeuroNode's resolved visual output is rendered to a shared render target.
3. The composed output is sent to whichever destination(s) the Vizlet's render mode specifies — presented via the transparent overlay window for Desktop, submitted to the VR compositor for VR, or both in the same frame for Desktop + VR.

The draw calls are identical regardless of target; only the final output submission differs. This means a Vizlet that works correctly in desktop mode also works correctly in VR mode — and in both at once — with no additional authoring required. Rendering to both destinations simultaneously costs a second submission call per frame, not a second render pass; the expensive part (evaluating expressions and composing the frame) happens once regardless of how many destinations receive it.

## VR Compositor Integration

StarVizion talks to the VR compositor directly — there is no intermediary shell service. Per active Vizlet with VR as part of its render mode (VR, or Desktop + VR), StarVizion:

- Creates an overlay handle for the Vizlet
- Submits the rendered texture to that handle every frame
- Sets the overlay's transform (world-locked or HMD-locked, per a per-Vizlet setting)
- Destroys the overlay handle when the Vizlet deactivates

**Requirement:** the VR runtime must be active for the VR side of rendering to work. StarVizion checks whether the VR runtime is running and an HMD is connected before activating the VR side. A Vizlet set to VR-only gracefully falls back to desktop-only rendering if the check fails, and sets a warning badge on the nav button. A Vizlet set to Desktop + VR is unaffected on its desktop side when the check fails — it simply runs desktop-only until the VR runtime becomes available, still with the same warning badge, rather than losing its output entirely the way a VR-only Vizlet's fallback does.

## VR Overlay Positioning and Persistence

Each Vizlet in VR mode has a world-space transform — position in meters relative to the HMD or world origin, and a width in meters (aspect ratio is preserved from the Vizlet's pixel dimensions). This is stored alongside the desktop window properties in the Vizlet's saved data.

**Activation restores the saved position automatically.** When a Vizlet activates in VR mode, StarVizion creates the overlay and immediately applies the transform already saved in that Vizlet's file — no per-session setup required.

**The pilot places VR Vizlets from the desktop, in the VR layout view** *(settled by Krondor, 2026-11-26)*. Flight
and space-sim pilots fly with a HOTAS, so their VR motion controllers are usually put away, and a pilot setting up
a cockpit may not have the headset on at all. Placement is therefore done with the mouse:

- **What it shows.** The view around a seated pilot, unwrapped flat: 180° wide, as far as the head turns without
  the shoulders moving, and 120° tall, 60° up and 60° down. That makes a 3:2 view with 60° × 40° squares. Nothing
  goes behind the pilot. The vertical range may be widened later if pilots need more.
- **The centre** is where the pilot looks when facing forward: the VR runtime's seated forward direction, which the
  pilot sets with its "reset seated position" or "recenter" command.
- **Guides, not snap zones.** The view is divided into a 3×3 grid: the centre square is straight ahead, the ones
  around it are left, right, up, down and the diagonals. The grid only helps the pilot find directions; a Vizlet can
  go anywhere.
- **Mouse.** Drag moves a Vizlet around the pilot. The mouse wheel moves it closer or farther, and the view draws it
  bigger or smaller to match, like a layer in an image editor. Resize handles change its width. Vizlets can overlap:
  the closer one is in front.
- **Live in the headset.** While the headset is on, every change shows in VR as it is made, so a pilot can wear the
  headset and arrange the cockpit with the mouse.
- **Stored as before:** a direction, a distance and a width, in the transform described above.

A Vizlet that follows the headset (HMD-locked) is placed the same way, with the centre of the view being wherever
the pilot looks.

**Later: moving overlays with VR controllers.** Some pilots do keep their motion controllers charged, and grabbing
an overlay in the headset would be nice for them. It is not in the first release. OpenVR does not move another
application's overlays for it, so StarVizion would build it:

- **Detecting the controllers.** OpenVR reports which tracked devices are connected and which are controllers.
  With none connected, nothing changes.
- **Which button does what.** StarVizion would not pick buttons per controller model. It declares actions
  (grab, move closer or farther) with default bindings for common controllers through SteamVR Input. Pilots rebind
  them in SteamVR's own binding screen, like any VR application.
- **Read-only towards games.** Taking controller input away from the game would break the rule in `CLAUDE.md`. So
  grabbing works only in a layout mode the pilot turns on from StarVizion, never while they are just flying.
- **Persisting.** A move made in the headset is written back to the Vizlet's file, the same as one made with the
  mouse.

HMD-locked positioning — an overlay that follows the headset like a HUD element, rather than staying fixed in world space — is also supported as a per-Vizlet setting.

## Stream Canvas

*Settled by Krondor, 2026-11-26 ([Decisions — D17](DECISIONS.md#d17-a-stream-canvas-for-obs-separate-from-what-the-pilot-sees)).*

The desktop overlay is click-through, so OBS cannot capture it as a window. Streamers could only capture the whole
desktop, and then the stream shows the Vizlets exactly where the pilot put them for flying. The **Stream Canvas**
is a second output arranged for the stream instead.

- **A normal window, not an overlay.** It is not click-through and not always on top, so OBS captures it with an
  ordinary window source. It stays open, usually behind the game.
- **An opaque background, black by default.** Streamers remove it with a colour-key filter in OBS. Black keys cleanly
  around text; green leaves a green fringe. The pilot can pick another background colour.
- **A fixed resolution, chosen by the pilot** to match their OBS canvas, from presets (1920×1080, 2560×1440,
  3440×1440, 3840×2160) or a custom size. There is no standard aspect ratio, so 16:9 and 21:9 are both first-class.
- **Its own Vizlet positions.** A Vizlet on the Stream Canvas has a position and size there, saved in its file,
  separate from its desktop window and its VR transform. Moving it on the stream does not move it for the pilot.
- **Independent of render mode.** "Show on Stream Canvas" is a per-Vizlet switch alongside the
  [render mode](#rendering-modes), not a fifth mode, so a Vizlet can be Desktop + VR and on the stream at once.
- **For VR streamers too.** The stream shows the VR view, which moves with the pilot's head. Vizlets on the Stream
  Canvas, layered over that feed in OBS, stay still on screen.
- **Same pipeline.** The Stream Canvas is one more output of the [Rendering Pipeline](#rendering-pipeline):
  expressions are evaluated once per frame, and only the drawing and presenting are repeated for it.

**Cost.** Vizlets are flat 2D shapes, and most of the canvas is empty background. Drawing them at 3440×1440 (about
5 megapixels) or 4K (about 8) is small work for any GPU that runs the game. The real cost of a large canvas is OBS's
capture and encoding, which the streamer pays anyway. To keep it low, the Stream Canvas renders at the frame rate
the pilot sets for it (OBS usually runs at 30 or 60), not the monitor's rate, and not at all when it is closed.

**Arranging.** The Stream Canvas has its own layout view in the editor, drawn at the canvas's aspect ratio. The
pilot drags and drops Vizlets onto it and sizes them there.

**Key colour.** Dark theme colours close to the key colour can be keyed out with the background. The pilot's answer
is to change the background colour and tune the key in OBS until it looks right for their stream. StarVizion does not
warn about it.

**Not in the first release:** a separate theme for the Stream Canvas
([D11](DECISIONS.md#d11-starvizion-owns-colour-plugins-name-colour-roles)). Nice to have. If it is added, the Stream
Canvas follows the pilot's cockpit theme by default and the pilot can choose another.

**Open:**
- What happens to positions when the pilot changes the canvas resolution: scaled with it, or kept in pixels.
- Whether a Stream Canvas hidden behind the game keeps rendering. Compositors can stop redrawing a window that is
  fully covered, which would freeze it in OBS. To be tested on Windows and KDE in the overlay prototype.
- Whether a HoloFrame can switch its members onto or off the Stream Canvas as a group. Likely yes, not decided.
