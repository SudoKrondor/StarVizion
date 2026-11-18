# Decisions

Decisions about StarVizion as a standalone project. Newest at the bottom. Each records who decided and when, and why.

## D1. StarVizion is its own project, not an Elite Intel feature

*Krondor, 2026-11-17, following a preliminary agreement with Alan.*

StarVizion leaves Elite Intel and becomes its own application and repository (`SudoKrondor/StarVizion`).
Elite Intel and StarVizion are independent: either installs and works without the other.

**Why:** pilots of other simulators share the same problem (springless sticks in VR), and would never find
StarVizion inside an Elite Dangerous voice assistant. A 90-144 Hz render loop, VR submission every frame and global
input hooks are also poor work for the JVM: Elite Intel's own HUD overlay is already a native child process for
that reason.

**What it supersedes in the Elite Intel spec:** anything that makes Elite Intel the host: Swing UI, `AppPaths`,
`elite.intel.devices` as the input source, and the Device Service owning frame-rate polling. StarVizion owns its
own input, paths, and UI.

## D2. Rust

*Krondor, 2026-11-17.*

Native code for the render loop, VR and input hooks, with memory safety and one cargo build for Windows and Linux.
Python was ruled out (thread model, installers). C++ works, but Rust removes the class of bugs that manual memory
management brings to Win32, X11 and OpenVR code.

## D3. Usable on its own; extended through providers

*Krondor, 2026-11-17.*

Straight after install, with no other software, StarVizion shows live controller state on the desktop and in VR.
Everything else arrives through providers (`starvizion-provider-api`), which any developer can write.

**Why:** EDO StellarCore stalled because a plugin host gives players nothing until its plugins exist. StarVizion's
first release must not need a plugin to be worth installing.

## D4. Elite Intel is a bundled plugin; Elite Intel 2.0 integrates

*Krondor, 2026-11-17.*

The Elite Intel integration ships with StarVizion and stays idle until it finds Elite Intel running, then needs no
setup. StarVizion is a candidate to display Elite Intel's overlays in VR. The Elite Intel side of the integration
lands in Elite Intel 2.0.

## D5. Keyboard capture is opt-in

*Krondor, 2026-11-17.*

Global keyboard capture is off until the user turns it on. On Linux it needs evdev access the user must grant, and
it is keylogger-shaped access that should be explained, not assumed. Many users will only want controller state
and game data in VR, where a keyboard is of little use anyway.

## D6. Native installers only

*Krondor, 2026-11-17.*

A setup `.exe` on Windows; `.deb` and `.rpm` on Linux (gamers mostly run Fedora- or Debian-derived distributions).
No Flatpak, no Snap, no storefront: a sandbox breaks global input hooks, desktop overlays and talking to other
local apps such as Elite Intel. Leading candidate tool: `cargo-packager` (NSIS on Windows), built per OS in GitHub
Actions.
