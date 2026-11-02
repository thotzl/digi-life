import type { BrainTopology } from "../../bindings/BrainTopology";
import type { CTRNNSynapse } from "../../bindings/CTRNNSynapse";
import type { GeneRef } from "../../bindings/GeneRef";
import type { GeneSpan } from "../../bindings/GeneSpan";
import type { Motor } from "../../bindings/Motor";
import type { NeuronRole } from "../../bindings/NeuronRole";
import type { Organ } from "../../bindings/Organ";
import type { OrganKind } from "../../bindings/OrganKind";
import { ORGAN_KINDS } from "../../bindings/organRegistry";
import { PROMOTERS } from "../../bindings/promoterRegistry";
import type { SystemicInput } from "../../bindings/SystemicInput";

/**
 * Display names for backend gene spans and neuron roles (TCK-187). The backend decides which genes exist and what
 * every neuron is; this module only maps promoter and enum values to text. Keep it data: one table per concept.
 */

/** Meaning of each promoter `parse_genome` expresses, from the backend promoter registry (`PROMOTERS` in
 * `src/biology/genome/promoters.rs`, generated into `bindings/promoterRegistry.ts`).
 * `firstOnly`: the compiler reads only the first gene of this promoter; later copies are silent. */
export const PROMOTER_MEANINGS: Record<string, { meaning: string; firstOnly?: boolean }> = Object.fromEntries(
  PROMOTERS.map(p => [p.motif, p.expression === "firstOnly" ? { meaning: p.meaning, firstOnly: true } : { meaning: p.meaning }]),
);

export const SYSTEMIC_NAMES: Record<SystemicInput, string> = {
  clock: "Hunger clock",
  linearSpeed: "Linear speed",
  rotationalSpeed: "Rotational speed",
  energy: "Internal energy",
  pain: "Pain",
  searchArousal: "Search arousal",
};

/** Display names per organ kind and its spectral channels 0..4 (channel 5 is the temporal change ΔI), from the
 * backend organ registry (`OrganSpec` in `src/biology/organs/`, generated into `bindings/organRegistry.ts`). */
export const ORGAN_KIND_NAMES = Object.fromEntries(
  Object.entries(ORGAN_KINDS).map(([kind, info]) => [kind, info.name]),
) as Record<OrganKind, string>;
export const CHANNEL_NAMES = Object.fromEntries(
  Object.entries(ORGAN_KINDS).map(([kind, info]) => [kind, info.channels]),
) as Record<OrganKind, string[]>;
/** Index of the ΔI channel (`DELTA_CHANNEL` in `src/biology/organs/mod.rs`). */
export const DELTA_CHANNEL = 5;

export const MOTOR_NAMES: Record<Motor, string> = {
  thrust: "Thrust (fwd/bwd)",
  bend: "Bend (left/right)",
};

export function channelName(kind: OrganKind | undefined, channel: number): string {
  if (channel === DELTA_CHANNEL) return "ΔI change";
  return (kind && CHANNEL_NAMES[kind]?.[channel]) || `Ch ${channel + 1}`;
}

export function geneMeaning(promoter: string): string {
  return PROMOTER_MEANINGS[promoter]?.meaning ?? `Unknown gene (${promoter})`;
}

export function sameGene(a: GeneRef, b: GeneRef): boolean {
  return a.promoter === b.promoter && a.spelling === b.spelling && a.occurrence === b.occurrence;
}

/** 1-based ordinal of gene `index` among the genes with the same promoter (genome order). */
export function promoterOrdinal(genes: GeneSpan[], index: number): number {
  const promoter = genes[index].gene.promoter;
  let n = 0;
  for (let i = 0; i <= index; i++) if (genes[i].gene.promoter === promoter) n++;
  return n;
}

/** Short gene name, e.g. "EYE 2". */
export function geneName(genes: GeneSpan[], index: number): string {
  return `${genes[index].gene.promoter} ${promoterOrdinal(genes, index)}`;
}

/** True for a later copy of a promoter the compiler reads only once. */
export function isSilentCopy(genes: GeneSpan[], index: number): boolean {
  return !!PROMOTER_MEANINGS[genes[index].gene.promoter]?.firstOnly && promoterOrdinal(genes, index) > 1;
}

/** Owning gene span index per genome position (-1 = non-coding); the earlier gene wins on overlap. */
export function geneIndexByPosition(genes: GeneSpan[], length: number): Int32Array {
  const owner = new Int32Array(length).fill(-1);
  genes.forEach((span, idx) => {
    for (let i = span.start; i < Math.min(span.end, length); i++) if (owner[i] === -1) owner[i] = idx;
  });
  return owner;
}

/** Organs a gene produced (matched by gene start). */
function organsOf(span: GeneSpan, organelles: Organ[]): Organ[] {
  return organelles.filter(o => o.geneStartIndex === span.start);
}

/** Full description of one gene for tooltips, e.g. "EYE 1: Sensory organ (EYE) - Eye pair". */
export function describeGene(genes: GeneSpan[], index: number, organelles: Organ[] = []): string {
  const span = genes[index];
  let text = `${geneName(genes, index)}: ${geneMeaning(span.gene.promoter)}`;
  const organs = organsOf(span, organelles);
  if (organs.length > 0) text += ` - ${ORGAN_KIND_NAMES[organs[0].kind] ?? organs[0].kind}${organs.length === 2 ? " pair" : ""}`;
  if (isSilentCopy(genes, index)) text += " [silent copy: only the first gene is read]";
  return text;
}

export interface RoleContext {
  genes: GeneSpan[];
  organelles: Organ[];
}

function geneIndexOf(ctx: RoleContext, gene: GeneRef): number {
  return ctx.genes.findIndex(s => sameGene(s.gene, gene));
}

function geneLabel(ctx: RoleContext, gene: GeneRef): string {
  const idx = geneIndexOf(ctx, gene);
  return idx >= 0 ? geneName(ctx.genes, idx) : gene.promoter;
}

function organKindOf(ctx: RoleContext, gene: GeneRef): OrganKind | undefined {
  const idx = geneIndexOf(ctx, gene);
  return idx >= 0 ? organsOf(ctx.genes[idx], ctx.organelles)[0]?.kind : undefined;
}

/** Display name of a neuron from its backend role. */
export function describeRole(role: NeuronRole, ctx: RoleContext): string {
  switch (role.role) {
    case "systemic":
      return SYSTEMIC_NAMES[role.input] ?? role.input;
    case "sensor": {
      const side = role.side === "single" ? "" : role.side === "left" ? " L" : " R";
      return `${ORGAN_KIND_NAMES[role.kind] ?? role.kind}${side} (${geneLabel(ctx, role.gene)}) - ${channelName(role.kind, role.channel)}`;
    }
    case "thalamus": {
      const kind = organKindOf(ctx, role.gene);
      const node = role.node === "sum" ? "Σ sum" : "Δ L-R diff";
      const organ = kind ? ` ${ORGAN_KIND_NAMES[kind]}` : "";
      return `Thalamus ${node}${organ} (${geneLabel(ctx, role.gene)}) - ${channelName(kind, role.channel)}`;
    }
    case "hidden":
      return `Hidden neuron (${geneLabel(ctx, role.gene)})`;
    case "motor":
      return `Motor: ${MOTOR_NAMES[role.motor] ?? role.motor}`;
  }
}

/** Synapse tooltip: source, target, weight, owner gene, and whether it is a fixed (non-learning) reflex. */
export function describeSynapse(syn: CTRNNSynapse, brain: BrainTopology, ctx: RoleContext): string {
  const name = (id: number) => {
    const n = brain.neurons[id];
    return n ? describeRole(n.role, ctx) : `#${id}`;
  };
  const owner = ctx.genes[syn.owner] ? describeGene(ctx.genes, syn.owner, ctx.organelles) : `gene #${syn.owner}`;
  const fixed = syn.plastic ? "" : "\nfixed reflex (does not learn)";
  return `${name(syn.fromNode)} → ${name(syn.toNode)}\nweight ${syn.weight.toFixed(3)}\nowner ${owner}${fixed}`;
}
