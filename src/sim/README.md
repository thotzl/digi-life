# sim — shared creature kernel

`World`, `WorldSettings`, `TickCtx` and `step_creature`: the one per-creature tick (sense → think → metabolize → death → move → eat) used by the Ocean, the trainer rooms and the probes. One module per dynamic in `systems/`; the IO-free Ocean in `ocean/`.

- Guide: add a dynamic: [`wiki/guides/add-dynamic.md`](../../wiki/guides/add-dynamic.md)
- Contract: creature system: [`wiki/contracts/creature-system.md`](../../wiki/contracts/creature-system.md)
- Rules: [`wiki/architecture/invariants.md`](../../wiki/architecture/invariants.md); module list: [`wiki/architecture/module-map.md`](../../wiki/architecture/module-map.md)
