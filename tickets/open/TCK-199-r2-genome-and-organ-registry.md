---
id: TCK-199
title: "R2 Creature Model as Modules: Genome Split, Promoter Registry, Organ Spec, Motor Outputs, Typed Enums"
status: open
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
