# Next tasks

Last updated: 2026-10-26. Planned tasks and findings not yet ticketed; remove an entry once it is ticketed or becomes the active task.

## Planned after epic TCK-171 (user goal, 2026-10-26; planning only, no tickets yet)
Goal: with this epic and follow-up epics, build the setup that fulfils the game-dynamics part of the whitepaper (`wiki/overview/whitepaper.md` §3–§4): players train creatures and later send them into arenas against others. The Ocean is secondary (one more training mode, e.g. reproduction dynamics).
- **Active traits from DNA**, next to sensing: claws, teeth, armour, limbs, spikes, ... as organ/trait kinds expressed by the genome compiler (extends `OrganKind`, ADR-002).
- **Dynamics as plugins:** reproduction (Ocean), attacking (active), digestion (more efficient use of food), ... each addable as a self-contained module on shared foundations (metabolism function, scenario plugins, creature factory), not wired into the engine loop by hand.
- **Rooms and modes as plugins:** arena (creature vs creature), Creator (edit DNA by hand, whitepaper §3.1), further training rooms; built on the existing scenario/plugin and runner foundations.
- **Code pass before features:** review the whole code base, fix bugs, make it DRY (utils, helpers, reusable classes/modules), so the above only adds modules. Relation to roadmap TCK-169 Phase 1 (targeted rebuild) and Phase 2 (features, incl. TCK-111/113/118/123) to be decided when ticketing.
- Next step after P8: propose an epic structure and tickets to the user (needs GO before creating tickets).
- **Training escalations** (from TCK-175, not part of epic TCK-171; only if larger populations and the curriculum are not enough): novelty blend (fitness plus a novelty term on end position / visited sectors, Cuccu/Gomez 2011, or minimal-criterion novelty), multi-objective selection (eating, path and energy as separate Pareto objectives, Mouret/Doncieux), connection cost as a selection objective (Clune/Mouret/Lipson 2013). Measure on unseen layouts (`--eval-final`).
- **Dependency security alerts:** ticketed as TCK-189.
- Remaining gap from TCK-171 (`## Wrap-up`): targets met at the app default population 100 and at 300; at pop 16 the large room stays at 0.227. Only relevant if small populations matter (e.g. arenas with few creatures); options: the training escalations below.

## Findings (not yet ticketed)
- None open.
