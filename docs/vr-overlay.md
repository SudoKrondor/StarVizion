# StarVizion — Vizlet Windows, VR and Streaming

Every Vizlet is drawn into ordinary windows on the desktop. StarVizion has no VR code of its own: VR players use
Desktop+ to bring Vizlet windows into the headset, and streamers capture them in OBS
([Decisions — D18](DECISIONS.md#d18-vizlets-are-windows-an-overlay-for-the-desktop-a-capture-window-for-vr-and-obs)).
A plan to render through Elite Intel's native overlay or Swing windows, from when StarVizion was to be an Elite
Intel feature, is superseded by [Decisions — D1 and D2](DECISIONS.md).

(This file keeps its old name, `vr-overlay.md`, so links to it still work.)

## Two Kinds of Vizlet Window

| Kind | For | What it is |
|---|---|---|
| **Overlay** | Pilots playing on a monitor | Transparent, frameless, click-through, always on top. Placed anywhere on the desktop. |
| **Capture window** | VR through Desktop+, and OBS | Frameless. Transparent background, or a solid colour for keying. Captured by other software as a window of its own. |

A Vizlet can show as an overlay, as a capture window, or as both at once. Each is a per-Vizlet switch on the
[Vizlet tab](ui-layout.md#vizlet-tab). A [HoloFrame can set them for all its members at once](holoframes-and-persistence.md#window-kind-override).

Overlays draw over a game in borderless windowed mode, not exclusive fullscreen, on every OS. Pilots already expect
that.

### Overlay

- **Transparent and click-through.** Clicks pass through to the game, so the overlay never takes the mouse away
  from it.
- **Always on top**, over the game.
- **Placed by the pilot** anywhere on the desktop. The position is saved in the Vizlet's file.
- **Linux:** a Wayland layer-shell surface, tested on KDE Plasma; GNOME is best effort
  ([D16](DECISIONS.md#d16-linux-desktop-overlays-target-wayland-on-kde-plasma-gnome-is-best-effort)).
- **Cannot be captured** by OBS or Desktop+ as a window. Capturing the whole desktop does show it.

### Capture Window

- **A normal window without decorations.** Not click-through, not always on top. It stays open, usually behind the
  game, while Desktop+ or OBS captures it.
- **Background:** transparent, or a solid colour chosen by the pilot, black by default. Desktop+ keeps a
  transparent background (shown by Alan's prototype). For OBS, streamers usually key out black: green leaves a fringe
  around text. Dark theme colours close to the key colour can be keyed out too; the pilot tunes the background and
  the key until it looks right, and StarVizion does not warn about it.
- **Its size is the Vizlet's own size.** Where it sits on the desktop does not matter: Desktop+ places it in VR, and
  OBS places it in the scene.

## VR

**Windows:** the pilot runs [Desktop+](https://github.com/elvissteinjr/DesktopPlus), captures a Vizlet's capture
window and positions it in the headset with Desktop+'s own tools. Desktop+ keeps the position between sessions.
StarVizion does not talk to the VR runtime.

**Linux:** not in the first release. It waits until Krondor has a headset to try what works: StarVizion's own
solution, or a third-party tool StarVizion points pilots to (D18).

## Streaming

OBS captures each capture window as a source of its own, and the streamer arranges them in their scene and keys out
the background. Streaming 180° or 360° VR video is not supported: few viewers can watch it.

## Rendering Pipeline

1. StarCalc evaluates every NeuroNode's expressions for the frame, once.
2. The Vizlet is drawn into each of its open windows: the overlay, the capture window, or both.
3. Each window presents its frame.

A Vizlet that works as an overlay works the same as a capture window, with no extra authoring. Showing both costs a
second draw, not a second evaluation.

## Open

- **Covered windows.** A capture window sits behind the game. Some compositors stop redrawing a window that is fully
  covered, which would freeze it in Desktop+ and OBS. Windows usually keeps drawing; KDE on Wayland may not. To be
  tested in the window prototype.
- **Arranging click-through overlays.** A click-through overlay cannot be dragged or right-clicked, so the
  [live Vizlet context menu](ui-layout.md#live-vizlet-context-menu) and left-click drag need another way in. Proposed:
  Lock/Unlock becomes click-through on/off. An unlocked overlay can be dragged and right-clicked. Unlocking is done
  from StarVizion's main window or tray, since a locked overlay cannot be clicked.
- **Both backgrounds at once.** A pilot who wants the same Vizlet transparent in Desktop+ and on black in OBS would
  need two capture windows. Not in the first release unless it proves cheap.
- **Whether OBS keeps transparency** when capturing a window. If it does, streamers may not need a key colour at all.
- **Linux VR**, as above.
