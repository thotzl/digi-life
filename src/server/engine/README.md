# server/engine — plugin host

`Engine` + `run_loop` (`mod.rs`), the `Mode` trait and warp cap (`mode.rs`), the modes `OceanMode` and `TrainerMode`, the `EventSink` (`sink.rs`) and the store ports (`store.rs`). Runs headless in `tests.rs`; only `spawn_simulation_thread` needs Tauri.

- Guide: add a room or mode: [`wiki/guides/add-room-or-mode.md`](../../../wiki/guides/add-room-or-mode.md)
- Contract: engine mode: [`wiki/contracts/mode.md`](../../../wiki/contracts/mode.md)
- Guide: add an event sink: [`wiki/guides/add-event-sink.md`](../../../wiki/guides/add-event-sink.md)
- Contract: event sink: [`wiki/contracts/event-sink.md`](../../../wiki/contracts/event-sink.md)
- Rules: [`wiki/architecture/invariants.md`](../../../wiki/architecture/invariants.md); module list: [`wiki/architecture/module-map.md`](../../../wiki/architecture/module-map.md)
