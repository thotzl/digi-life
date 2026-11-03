---
id: TCK-113
title: Tauri v2 Mobile Platform Builds (Android & iOS)
status: open
assigned: unassigned
created_at: 2026-08-21
epic_phase: Roadmap TCK-169 Phase 2 (feature epics, to be planned)
---

# TCK-113: Tauri v2 Mobile Platform Builds (Android & iOS)

## Update (2026-11-03, after TCK-196)
- Desktop release builds exist since TCK-195 (`npm run release:win`, NSIS + portable ZIP; bundle targets are a platform-neutral list; Linux/macOS via `release.yml` still open). Mobile builds would add Tauri mobile targets and a touch-capable UI on the `ViewModule` seam (`wiki/contracts/frontend-view.md`). Re-scope before starting.

This ticket outlines the configuration, initialization, and build requirements for compile-testing the biomorphic sandbox and HUD interfaces on mobile devices.

## Requirements & Scope

### 1. Mobile Project Initialization
- Run `npx tauri android init` and `npx tauri ios init` to configure Android Studio and Xcode directories.
- Configure safe-area padding offsets in `beta.css` to prevent notches and taskbars from overlapping interactive controls.

### 2. Touch Interactivity Fallbacks
- Adapt pointer-event handlers in canvas renderers to support multitouch pinching (zoom) and double-tap gestures for focus selection.
- Build simulated joystick overlay or virtual thumb-steering pad on mobile viewports.

### 3. Build & Bundling
- Compile and sign testing APKs for Android using Android Studio.
- Compile and test iOS emulator bundles using Xcode.
