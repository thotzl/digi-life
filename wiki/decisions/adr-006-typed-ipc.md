---
sources: [tickets/closed/TCK-157-typed-ipc-contract.md, tickets/closed/TCK-160-frontend-structure.md, tickets/closed/TCK-161-test-suite-csp-lint.md, tickets/closed/TCK-197-r0-safety-net.md, src/server/ipc.rs, wiki/contracts/ipc-message.md, wiki/contracts/frontend-view.md]
---

# ADR-006: Typed IPC End to End, Backend as Source of Truth, Partial Updates

- Status: accepted
- Date: 2026-11-03
- Tickets: TCK-157, TCK-160, TCK-161, TCK-197

## Context

Only the trainer messages were typed (TCK-177). Ocean actions and events were JSON built by hand, command results were `serde_json::Value`, and the frontend kept hand-written copies of backend types that had drifted. `safeInvoke` swallowed errors, so failures showed nowhere. Two defects in this session came from the UI sending whole state snapshots: a delayed reset event and, later, the boot gate both overwrote or dropped a speed change the user had just made. A catalogue name could inject HTML with full IPC access (no CSP).

## Decision

1. **Every message is a Rust type exported with ts-rs.** `ClientAction` (engine, Ocean, trainer actions), `UiEvent` (engine, Ocean, trainer events), command results as DTOs and `IpcError`; the frontend imports types only from `bindings/` (invariant 13). Unknown or malformed actions are rejected with a logged, user-visible error.
2. **One IPC client in the frontend** (`frontend/core/ipc.ts`): typed `call`, `send`, `setMode`, one event listener, errors shown as a notice.
3. **The mode belongs to the router.** The mode changes only through the `set_mode` command, called by the router (invariant 1).
4. **The backend is the source of truth; controls send partial updates.** A UI control sends only the field it changed (e.g. `UPDATE_TRAINER_HYPERPARAMS { warpSpeed }`); the UI never pushes a full settings snapshot back, so no stale value can overwrite a newer one and a change before the first backend state is not lost.
5. **The client renders and never interprets untrusted strings as markup.** Text children are always text; markup only through an explicit `html()` marker for built-in icons; a restrictive CSP is set (invariants 14, 15).

## Consequences

- Adding a message is a Rust type, a dispatch arm and a regenerated binding (`wiki/guides/add-ipc-message.md`); `tsc` finds every consumer.
- Errors reach the user instead of disappearing; the engine logs trainer start, pause and speed changes for diagnosis.
- Bindings must be committed after a type change (invariant 7, checked in CI).
