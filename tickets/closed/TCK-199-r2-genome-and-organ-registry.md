---
id: TCK-199
title: "R2 Creature Model as Modules: Genome Split, Promoter Registry, Organ Spec, Motor Outputs, Typed Enums"
status: closed
priority: high
assigned: agent
created_at: 2026-11-02
epic_phase: TCK-196 R2
depends_on: TCK-198
---

# TCK-199: Creature Model as Modules

Absorbs TCK-155 (typed model enums). Goal: a new organ kind (e.g. claws) = one file in `organs/` + one enum variant; the compiler (exhaustive `match`) finds the rest.

## Findings (2026-11-02 review)
- `dna.rs` (3,149 lines) holds types/enums (36-488), alphabet (490-534), gene scan (536-594), generator (596-828), mutation (830-1183), ecology + Latin name (1185-1378), payload reader/chromatin (1380-1456), gene table (1459-1544), brain compiler (1546-1830), `parse_genome` (1832-2146), tests.
- Adding an organ kind touches today: promoter lists `EXPRESSED_PROMOTERS` 1459, `GENE_MOTIFS` 653, `ORGAN_PROMOTERS` 1464, chromatin list 1414, golden-master motifs 2691, generator plan 761 + `GenomeComposition` 608 + config; `OrganKind::classify` 40-71, `is_distance` 73, `live_channels` 84, `affinity_letters` 751, `sensor_label` 1588; `SensoryPatch` 258 (sensor-only); decode loop 1966-2015; fixed 6 sensor slots per organ 1636, thalamus 1652; motors (`Motor` 176, OUT loci 1717, positional output indexing in `engine.rs:986`, scenarios, `probes.rs:241`); sensing if-chain `brain.rs:270-410`; effects `physics.rs:115/135`, `metabolism.rs:65`, `engine.rs:1041-1130`, `trainer_engine.rs:370-400`; naming/ecology 1198-1378; probes `probes.rs:30/259-266/856-907`; frontend `genome.ts:17/48/57/70`, `creatureRenderer.ts:71-100/401-530`.
- Promoter set written 5× in dna.rs (648, 653, 1414, 1459, 1464) + TS `PROMOTER_MEANINGS` + golden-master motifs; trait ranges repeated in `physics.rs:101-105`; organ thresholds repeated (classify, ecology 1286-1294, renderer, probes 906); `first_payload` 1475 vs `GeneTable::first_payload` 1510.
- Implicit layout contract: input slot = organ index · 6 + channel; nearest plant/meat as `foods[0]/[1]`.
- Dead phenotype payload computed and sent over IPC, read nowhere: `hydraulic_pressure`, `repair_fidelity`, `rotational_inertia`, `segments` (dummy JSON → `any[]` in TS), `chromatin_state`, `epigenetic_logs`, `survival_*`, `preferred_habitat`, `diet_class` strings, `sensory_*`.
- String-typed model values (TCK-155 rest): activation (`dna.rs:353`, `"tanh"` literal 1665), `symmetry` 369, diet/habitat strings.
- `parse_genome` uses std `sin`/`cos` (1326, 2046) instead of `fmath` (cross-platform determinism leak; switching is a model change → TCK-202).

## Scope (behaviour-preserving)
1. Split into `genome/{alphabet,scan,generate,mutate,payload,table}.rs`, `phenotype/{body,organs,naming}.rs`, `brain/compile.rs`; display labels move out of the model.
2. `genome/promoters.rs`: one `static PROMOTERS: &[PromoterSpec { motif, channel, expression, essential, wobble, meaning }]`; every promoter list derives from it; exported via ts-rs so the frontend drops its copy.
3. `Payload` reader (`h(locus)`, `range(locus, lo, hi)`, `index(locus)`); body traits as a table of (promoter, decode fn).
4. Organ spec: `organs/{eye,nose,thermal,lateral_line,...}.rs`, `OrganKind::spec() -> &'static OrganSpec` with `decode`, `classify`, `live_channels`, `sense(&SenseCtx, &Organ, &mut [f32; CHANNELS])`, `motors`, `cost_load`, `locomotion_share`, `ballast`, `channel_names`, `render_hint`. `SensoryPatch` generalises to `Organ { common, kind, params }`. Sensing in `brain.rs` dispatches through the spec. Parity test per organ: `live_channels` vs `sense`.
5. `MotorOutputs` struct instead of positional `[f32; 2]`.
6. Typed enums for activation, symmetry, diet class, habitat (TCK-155).
7. Remove dead phenotype fields (Rust, bindings, frontend types).

## Gate
Golden master (model-field hash from TCK-197) and trainer references unchanged. Removing dead fields does not change the model-field hash by construction.

## Acceptance
- `dna.rs` gone; no module > ~600 lines; promoter, organ, motor data each in one place.
- Contract `wiki/contracts/organ-kind.md` and guide `wiki/guides/add-organ-kind.md` written in this ticket; `wiki/biology/genome-compiler.md` and `sensing.md` describe the new structure.
- TCK-155 moved to closed as absorbed.

## Verification (2026-11-02, v1.12.32)
- `dna.rs` (3,149 lines) removed. New modules in `src/biology/`: `genome/` (`alphabet`, `promoters` with `PROMOTERS: &[PromoterSpec]`, `scan`, `table`, `payload` with `Payload::{h, range, index}`, `generate`, `mutate`, tests), `phenotype/` (`parse_genome`, `CreaturePhenotype`, `Symmetry`, `body.rs` with the `BODY_GENES` decode table, `naming.rs`, tests), `organs/` (`OrganKind`, `spec()`, `classify`, `OrganSpec`, `Organ` (was `SensoryPatch`), `SenseCtx`, `decode_organs`, shared `spectral.rs` / `contact.rs`, one file per kind), `brain/` (roles, `Motor`, `MotorOutputs`, `Activation`, `compile.rs`). Largest file 483 lines (tests).
- A new organ kind now touches 2 files: `organs/<kind>.rs` and `organs/mod.rs` (mod line, enum variant + `ALL`, exhaustive `spec()` arm, its `classify` region); bindings `OrganKind.ts` / `organRegistry.ts` carry it to the frontend (before: ~10 places incl. sensing chain, physics, metabolism, naming, probes thresholds, frontend name tables). Parity test `organs::tests::test_live_channels_match_sense` over `OrganKind::ALL`.
- Promoter registry exported (`bindings/promoterRegistry.ts`), `frontend/shared/genome.ts` builds `PROMOTER_MEANINGS`, `ORGAN_KIND_NAMES`, `CHANNEL_NAMES` from the registries (copies removed). `MotorOutputs { thrust, bend }` replaces `[f32; 2]`. Typed `Activation`, `Symmetry` (TCK-155); diet/habitat strings were dead and are removed.
- Dead phenotype fields removed (Rust, bindings, frontend): `segments`, `hydraulic_pressure`, `rotational_inertia`, `repair_fidelity`, `chromatin_state`, `epigenetic_logs`, `survival_expectation`, `survival_analysis`, `preferred_habitat`, `diet_class`, `sensory_visus/olfaction/tactility/biolum`; `CTRNNNeuron.label`. The golden master still hashes `hydraulic_pressure` / `repair_fidelity`, decoded in the test from the genome exactly as before, so the fingerprint is unchanged.
- Behaviour bugs left for TCK-202 and marked `// TCK-202` (thermal `-5.0f32.max`, std `cos`, eyes as limbs, pain and predator thresholds).
- Gate on master (`.agents/artifacts/gate.sh`): `cargo test` 212 passed, 1 ignored; golden master `0x6b78c31a4d19b0e8` unchanged; standard, curriculum, rehearsal references match; `npm test` 13 passed; build ok; doc links OK (84 files); no bindings drift after regeneration. UI smoke OK (0 console errors); selection check (TCK-194 script): inspector shows the clicked sandbox (#63) with name and diet, two resets give different creatures.
- Docs: `wiki/contracts/organ-kind.md`, `wiki/guides/add-organ-kind.md`; `genome-compiler.md` and `sensing.md` rewritten to the module structure; paths fixed in ctrnn-brain, configuration, simulation-engine, config-section, adr-002, glossary, repository-map, testing, trainer, skill `biology-dna`.
