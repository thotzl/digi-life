# server — engine, IPC and commands

`engine/` runs the simulations on their own thread; `ipc.rs`, `ocean_ipc.rs`, `trainer_ipc.rs` hold the typed messages; `api/` the Tauri commands.

- Guide: add an IPC message: [`wiki/guides/add-ipc-message.md`](../../wiki/guides/add-ipc-message.md)
- Contract: IPC message: [`wiki/contracts/ipc-message.md`](../../wiki/contracts/ipc-message.md)
- Rules: [`wiki/architecture/invariants.md`](../../wiki/architecture/invariants.md); module list: [`wiki/architecture/module-map.md`](../../wiki/architecture/module-map.md)
