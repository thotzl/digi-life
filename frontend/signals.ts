import { signal } from "@preact/signals-core";
import { SpeciesRecord } from "./shared/types";

// REACTIVE HUD STATE OVERLAYS (Signals)
export const selectedId = signal<number | null>(null);
export const selectedName = signal("Unnamed Creature");
export const selectedTaxa = signal("Clonal strain - Gen. 1");
export const selectedStatus = signal("Alive");
export const selectedEnergy = signal(0);
export const selectedMaxEnergy = signal(100);
export const selectedAge = signal(0);

export const selectedGenome = signal("");
export const selectedMethylations = signal<number[]>([]);
export const selectedPhenotype = signal<any>(null);
export const selectedBrainActivations = signal<number[]>([]);

export const speciesRosterSignal = signal<SpeciesRecord[]>([]);

// Accordion Toggles
export const isAliveExpanded = signal(true);
export const isExtinctExpanded = signal(false);
