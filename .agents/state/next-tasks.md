# Next tasks

Last updated: 2026-11-03. Planned tasks and findings not yet ticketed; remove an entry once it is ticketed or becomes the active task.

## Retroactive ADRs (user, 2026-11-03; not started — the agent hit the monthly spend limit)
Record missing decisions from before 2026-11-02 as ADRs (ADR-008 upward), strictly from clear facts in `tickets/closed/`, `CHANGELOG.md` and `wiki/history/`: only explicit user decisions/GOs or choices between named alternatives; status `accepted (recorded retroactively on <date>)`, decision date from the source, evidence tickets in `sources:`; no invented reasons ("Reason not recorded in the sources"); states of affairs (e.g. measured baselines) go to `wiki/history/`, not into ADRs. Candidates to check: trainer evaluation and selection (TCK-174), fitness in catch units and minimal criterion (TCK-175), curriculum (TCK-176), energy model v2 (TCK-151/178), locomotion v2 (TCK-150/180), Oja learning and Lamarck by synapse key (TCK-152; overlap with ADR-002), population independent of the grid (TCK-192), tanh thalamus sum nodes (TCK-190), trainer run state (TCK-191), process decisions (tickets/wiki/`.agents/`) if recorded. Already covered: ADR-001..007.

## Feature epics on the TCK-196 seams (user goal 2026-10-26; Phase 2 of roadmap TCK-169; planning only, no tickets yet)
Goal: the game-dynamics part of the whitepaper (`wiki/overview/whitepaper.md` §3–§4): players train creatures and later send them into arenas against others. The Ocean is secondary (one more training mode, e.g. reproduction dynamics). The plugin architecture (epic TCK-196, closed, v1.12.44) provides the seams; each item below should only add modules (guides in `wiki/guides/`, contracts in `wiki/contracts/`, skill `plugin-authoring`).
- **Active traits from DNA**, next to sensing: claws, teeth, armour, limbs, spikes, ... as new organ kinds (`wiki/guides/add-organ-kind.md`); active organs need motor outputs (`MotorOutputs`, contract organ-kind) and effects in creature systems.
- **Dynamics as plugins:** attacking (active), digestion (more efficient use of food), richer reproduction; each a creature system (`wiki/guides/add-dynamic.md`).
- **Rooms and modes:** arena (creature vs creature: N-entity chambers and the reserved `PAIRING` stream exist; needs a pairing/tournament `Evaluator`, `wiki/guides/add-evaluator.md`), Creator (edit DNA by hand, whitepaper §3.1; TCK-111 must be rewritten on the `ViewModule` seam first), further training rooms.
- Existing feature tickets to place in these epics: TCK-111 (Creator), TCK-123 (multi-scenario/arena, re-scope on `Room`/`Evaluator`), TCK-148 (species model), TCK-118 (diploid genetics), TCK-165..168 (research organs/brain), TCK-113 (mobile).
- **Training escalations** (from TCK-175; only if larger populations and the curriculum are not enough, e.g. arenas with few creatures): novelty blend (Cuccu/Gomez 2011 or minimal-criterion novelty), multi-objective selection (Mouret/Doncieux), connection cost (Clune/Mouret/Lipson 2013). Measure on unseen layouts (`--eval-final`). At pop 16 the large room stayed at 0.227 (TCK-171 wrap-up).

## Before the beta (user, 2026-11-02)
- Identifier `de.pixlich.broodbattle` is alpha only. When the team declares the beta (others will use it), choose the final domain and switch the identifier once: dev = `<id>.dev`, one-time copy of the alpha save folder on first start (`migrate_legacy_db` pattern), beta notes ask to uninstall the alpha by hand. Details: TCK-195 section Identifier. Also a trademark search for "BroodBattle" before a public release.
- Linux/macOS builds (existing `release.yml`) and code signing: TCK-195 section Later.
- Remaining product naming (crate, package name, DB file, whitepaper, repo): TCK-193.

## Findings (not yet ticketed)
From the R7 contract probes (TCK-204, 2026-11-03; documented as current limits in `wiki/contracts/room.md` and `wiki/contracts/frontend-view.md`). Likely part of the arena/rooms epic:
- **Polygon/rectangle obstacles:** `ProceduralObstacle` is simulated as a circle (`x`, `y`, `radius`; `vertices` ignored) in movement collision, contact sense and food drift; walls/rectangles are only chains of circles. A real polygon collision is a host change (physics, contact sense, food drift).
- **Room-specific respawn and per-trial obstacles:** `RespawnPolicy` is a host type (a room cannot keep respawned food behind its course; `spawn_at_min_distance` also ignores obstacles); `Room::obstacles` has no chamber/RNG, so obstacles are fixed per room.
- **Obstacles in trainer telemetry:** `TrainerTelemetrySandbox` carries no obstacles, so the trainer UI cannot draw a room's obstacles.
- **Public test helper for room layout tests:** a small `#[cfg(test)]` helper (e.g. `TrainerSandbox::for_room(room, seed, config)`) would replace the copied 9-argument `TrainerSandbox::spawn` call in every room's tests.
- **Optional `backendMode: null` for views without a simulation:** today such views use `"ocean"`, so entering them sends `set_mode` and suspends the other views (training pauses); a `null` mode would keep the backend as it is.
- **CI on GitHub not verified from the agent environment** (`gh` not authenticated): first runs of the new jobs (Windows job, Ocean reference, fmt/clippy/lint) should be checked once; the Ocean reference is platform-independent since TCK-202 (`fmath`).
