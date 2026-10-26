# Next tasks

Last updated: 2026-10-26. Planned tasks and findings not yet ticketed; remove an entry once it is ticketed or becomes the active task.

## Planned after epic TCK-171 (user goal, 2026-10-26; planning only, no tickets yet)
Goal: with this epic and follow-up epics, build the setup that fulfils the game-dynamics part of the whitepaper (`wiki/overview/whitepaper.md` §3–§4): players train creatures and later send them into arenas against others. The Ocean is secondary (one more training mode, e.g. reproduction dynamics).
- **Active traits from DNA**, next to sensing: claws, teeth, armour, limbs, spikes, ... as organ/trait kinds expressed by the genome compiler (extends `OrganKind`, ADR-002).
- **Dynamics as plugins:** reproduction (Ocean), attacking (active), digestion (more efficient use of food), ... each addable as a self-contained module on shared foundations (metabolism function, scenario plugins, creature factory), not wired into the engine loop by hand.
- **Rooms and modes as plugins:** arena (creature vs creature), Creator (edit DNA by hand, whitepaper §3.1), further training rooms; built on the existing scenario/plugin and runner foundations.
- **Code pass before features:** review the whole code base, fix bugs, make it DRY (utils, helpers, reusable classes/modules), so the above only adds modules. Relation to roadmap TCK-169 Phase 1 (targeted rebuild) and Phase 2 (features, incl. TCK-111/113/118/123) to be decided when ticketing.
- Next step after P8: propose an epic structure and tickets to the user (needs GO before creating tickets).
- Open gap carried over from TCK-171 (`## Result`): at pop 16 the success targets are not met on unseen layouts (small 0.383, large 0.096); pop 300 meets both. Options: headless training with large populations, TCK-175 escalations (novelty blend, multi-objective, connection cost).

## Findings (not yet ticketed)
- **Thalamus sum nodes give a constant offset when silent** (found in P3, existed in the old model as well): sum nodes use `sigmoid` with bias 0, so a silent channel outputs 0.5 and feeds a constant drive into hidden neurons and motors (`src/biology/dna.rs`, thalamus `ThalamusKind::Sum` → sigmoid). Acts like an implicit bias per live channel. Check in P5/P7 with the steering probe whether a centred activation (e.g. `tanh`, or sigmoid − 0.5) improves directed steering; it is a model change (golden master, CI reference, `MODEL_VERSION`).
