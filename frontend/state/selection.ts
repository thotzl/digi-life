// One selection for the whole app (TCK-160): what the inspector panels show, tagged with where it comes from, so an
// id of one view is never read as an id of another (a trainer sandbox id is not an Ocean creature id, a catalogue id
// is a string). Static data (`selection`) and live data (`vitals`, `activations`) are separate signals, so the
// genome view and brain layout rebuild only when the selected creature changes, not on every telemetry tick.
import { signal } from "@preact/signals-core";

import type { CatalogueCreature } from "../../bindings/CatalogueCreature";
import type { CreatureAgent } from "../../bindings/CreatureAgent";
import type { CreaturePhenotype } from "../../bindings/CreaturePhenotype";
import type { SpeciesEntry } from "../../bindings/SpeciesEntry";
import type { TrainerTelemetrySandbox } from "../../bindings/TrainerTelemetrySandbox";
import { dietLabel } from "../shared/creatureLabels";

/** Where a selection comes from; the id type follows the source. */
export type SelectionRef =
  | { source: "ocean"; creatureId: number }
  | { source: "fossil"; speciesId: string }
  | { source: "trainer"; sandboxId: number }
  | { source: "catalogue"; catalogueId: string };

export type SelectionSource = SelectionRef["source"];

export interface Selection {
  ref: SelectionRef;
  name: string;
  taxa: string;
  genome: string;
  methylations: number[];
  /** `null` while the backend has not answered `get_fossil_phenotype` */
  phenotype: CreaturePhenotype | null;
  /** Fitness to store with a catalogue save; `null` where the source has none (Ocean) */
  fitness: number | null;
  /** The creature's own synapse weights; empty where the source does not send them (trainer telemetry) */
  synapseWeights: number[];
}

export interface Vitals {
  status: string;
  energy: number;
  maxEnergy: number;
  /** `null` where an age has no meaning (fossil, catalogue) */
  age: number | null;
}

export const NO_VITALS: Vitals = { status: "", energy: 0, maxEnergy: 100, age: null };

export interface SelectionStore {
  readonly current: Selection | null;
  select(selection: Selection, vitals?: Vitals): void;
  /** Replaces the phenotype of the current selection if it is still `ref` (async phenotype answers). */
  setPhenotype(ref: SelectionRef, phenotype: CreaturePhenotype): void;
  /** Live data of the current selection; ignored if `ref` is no longer selected. */
  updateVitals(ref: SelectionRef, vitals: Vitals): void;
  updateActivations(ref: SelectionRef, activations: number[]): void;
  rename(ref: SelectionRef, name: string): void;
  clear(): void;
}

export const selection = signal<Selection | null>(null);
export const vitals = signal<Vitals>(NO_VITALS);
export const activations = signal<number[]>([]);

export function sameRef(a: SelectionRef | null | undefined, b: SelectionRef | null | undefined): boolean {
  if (!a || !b || a.source !== b.source) return false;
  switch (a.source) {
    case "ocean": return a.creatureId === (b as typeof a).creatureId;
    case "fossil": return a.speciesId === (b as typeof a).speciesId;
    case "trainer": return a.sandboxId === (b as typeof a).sandboxId;
    case "catalogue": return a.catalogueId === (b as typeof a).catalogueId;
  }
}

function isCurrent(ref: SelectionRef): boolean {
  return sameRef(selection.peek()?.ref, ref);
}

export const selectionStore: SelectionStore = {
  get current() {
    return selection.peek();
  },
  select(next, nextVitals = NO_VITALS) {
    if (!isCurrent(next.ref)) activations.value = [];
    selection.value = next;
    vitals.value = nextVitals;
  },
  setPhenotype(ref, phenotype) {
    const cur = selection.peek();
    if (cur && sameRef(cur.ref, ref)) selection.value = { ...cur, phenotype, methylations: cur.methylations.length ? cur.methylations : phenotype.methylations };
  },
  updateVitals(ref, next) {
    if (isCurrent(ref)) vitals.value = next;
  },
  updateActivations(ref, next) {
    if (isCurrent(ref)) activations.value = next;
  },
  rename(ref, name) {
    const cur = selection.peek();
    if (cur && sameRef(cur.ref, ref)) selection.value = { ...cur, name };
  },
  clear() {
    selection.value = null;
    vitals.value = NO_VITALS;
    activations.value = [];
  },
};

// --- typed id accessors: each view reads only ids of its own source ---

export function oceanCreatureId(sel: Selection | null): number | null {
  return sel?.ref.source === "ocean" ? sel.ref.creatureId : null;
}

export function trainerSandboxId(sel: Selection | null): number | null {
  return sel?.ref.source === "trainer" ? sel.ref.sandboxId : null;
}

export function catalogueId(sel: Selection | null): string | null {
  return sel?.ref.source === "catalogue" ? sel.ref.catalogueId : null;
}

// --- builders: the one place each source fills a selection (formerly six copies of the signal fill) ---

export function fromOceanCreature(agent: CreatureAgent): { selection: Selection; vitals: Vitals } {
  const p = agent.phenotype;
  return {
    selection: {
      ref: { source: "ocean", creatureId: agent.id },
      name: p.latinName,
      taxa: `${p.latinName.substring(0, 16)} (Strain: #${agent.id}, Gen: ${agent.generation})`,
      genome: agent.genome,
      methylations: p.methylations,
      phenotype: p,
      fitness: null,
      synapseWeights: agent.synapseWeights,
    },
    vitals: { status: "Alive", energy: agent.energy, maxEnergy: p.stomachCapacity, age: agent.age },
  };
}

export const FOSSIL_VITALS: Vitals = { status: "Extinct (Fossil)", energy: 0, maxEnergy: 100, age: null };

export function fromFossil(record: SpeciesEntry, phenotype: CreaturePhenotype | null): Selection {
  return {
    ref: { source: "fossil", speciesId: record.id },
    name: record.name,
    taxa: `${record.name} [FOSSIL - EXTINCT]`,
    genome: record.genome,
    methylations: phenotype?.methylations ?? [],
    phenotype,
    fitness: null,
    synapseWeights: [],
  };
}

export function sandboxVitals(tele: TrainerTelemetrySandbox, phenotype: CreaturePhenotype): Vitals {
  return { status: tele.finished ? "🏁 SUCCESS" : "Active", energy: tele.energy, maxEnergy: phenotype.stomachCapacity, age: tele.age };
}

export function fromSandbox(tele: TrainerTelemetrySandbox, phenotype: CreaturePhenotype, generation: number): { selection: Selection; vitals: Vitals } {
  return {
    selection: {
      ref: { source: "trainer", sandboxId: tele.id },
      name: phenotype.latinName,
      taxa: `${phenotype.latinName} (${dietLabel(phenotype.carnivory)}) - Sandbox #${tele.id} (Gen. ${generation})`,
      genome: tele.genome,
      methylations: phenotype.methylations,
      phenotype,
      fitness: tele.current_fitness,
      synapseWeights: [],
    },
    vitals: sandboxVitals(tele, phenotype),
  };
}

export const CATALOGUE_VITALS: Vitals = { status: "Cryo-Frozen", energy: 100, maxEnergy: 100, age: null };

export function fromCatalogue(creature: CatalogueCreature, phenotype: CreaturePhenotype | null): Selection {
  return {
    ref: { source: "catalogue", catalogueId: creature.id },
    name: creature.name,
    taxa: `Provenance: ${creature.source}`,
    genome: creature.genome,
    methylations: creature.methylations,
    phenotype,
    fitness: creature.fitness,
    synapseWeights: creature.synapseWeights,
  };
}

/** Arguments of `save_to_catalogue` for the selected creature (without `id`). */
export function catalogueEntry(sel: Selection, name: string): { name: string; genome: string; source: string; fitness: number; methylations: number[]; synapseWeights: number[] } {
  return {
    name,
    genome: sel.genome,
    source: sel.ref.source === "trainer" ? "Evolutionary Trainer" : "Ocean Substrate",
    fitness: sel.fitness ?? 0,
    methylations: sel.methylations,
    synapseWeights: sel.synapseWeights,
  };
}
