---
id: TCK-109
title: Implementation of Tauri v2 Desktop Shell & Native IPC Protocol
status: closed
assigned: Gemini-CLI
created_at: 2026-08-21
---

# TCK-109: Implementation of Tauri v2 Desktop Shell & Native IPC Protocol

## Description
Establish a single-dependency, portless desktop application using Tauri v2. The UI layer (Vite, TypeScript, Preact Signals) must be bundled into a native borderless desktop window, communicating directly with a headless native background process via Tauri's IPC channels.

## Requirements & Scope
- **Tauri v2 Shell Configuration:**
  - Initialize Tauri v2 in the project root directory.
  - Configure the application window to be frameless with custom window controls rendered in HTML/CSS.
- **Portless Communication Channel:**
  - Eliminate standard HTTP/WebSocket interfaces for local application state.
  - Implement Tauri IPC command handlers (`invoke`) to send control parameters and state requests to the background process.
  - Set up a high-frequency event stream from Rust to JavaScript for spatial and neural telemetry.
- **Asynchronous Execution Architecture:**
  - Establish a pipeline where the client UI manages session profiles, local file configurations, and database lookups, passing only operational task specifications (IDs and parameters) to the background core.
- **Zero-Dependency Compilation:**
  - Create building pipelines where `npm run tauri build` outputs a single standalone executable package for Windows, macOS, and Linux without local Node.js or Git prerequisites.
