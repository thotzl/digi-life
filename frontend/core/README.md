# frontend/core — router, lifecycle, IPC client

`router.ts` (view routing, owns the backend mode), `view.ts` and `scope.ts` (view lifecycle and teardown), `ipc.ts` (the only Tauri caller), plus camera, `styled`, escaping and toasts.

- Contract: frontend view: [`wiki/contracts/frontend-view.md`](../../wiki/contracts/frontend-view.md)
- Contract: IPC message: [`wiki/contracts/ipc-message.md`](../../wiki/contracts/ipc-message.md)
- Rules: [`wiki/architecture/invariants.md`](../../wiki/architecture/invariants.md); module list: [`wiki/architecture/module-map.md`](../../wiki/architecture/module-map.md)
