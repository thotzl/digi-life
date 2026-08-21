---
id: TCK-114
title: Tauri v2 GitHub Actions Workflow (Desktop Releases)
status: closed
assigned: Gemini-CLI
created_at: 2026-08-21
---

# TCK-114: Tauri v2 GitHub Actions Workflow (Desktop Releases)

This ticket tracks the design and integration of a parallel multi-platform Cloud Build Pipeline on GitHub Actions that compiles ready-to-run `.dmg`, `.exe`, `.msi`, `.deb`, and `.AppImage` bundle installers on release tags.

## Definition of Done

### 1. Multi-Platform Build Configuration
- Create `.github/workflows/release.yml` with a standard GitHub Actions schema.
- Define a build matrix running on `macos-latest` (macOS), `windows-latest` (Windows), and `ubuntu-22.04` (Linux).

### 2. Dependency Toolchains
- Set up Node.js, npm, Rust, Cargo, and native C dependencies (such as `libsoup` and `webkit2gtk` on Linux Ubuntu).
- Build the web bundle using `npm run build` prior to Tauri compilation.

### 3. Automatic Asset Packaging
- Utilize the official `tauri-apps/tauri-action` step to compile release bundles.
- Automatically draft a new GitHub Release with the bundled installers (`.dmg`, `.exe`, `.msi`, `.deb`, `.AppImage`) attached as download assets when a tag like `v*` is pushed.
