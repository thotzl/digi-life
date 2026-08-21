import { signal } from "@preact/signals-core";
import { SpeciesRecord, GeneSpan } from "../shared/types";

// REACTIVE HUD STATE OVERLAYS (Signals)
export const selectedId = signal<number | null>(null);
export const selectedName = signal("Unnamed Creature");
export const selectedTaxa = signal("Clonal strain - Gen. 1");
export const selectedStatus = signal("Alive");
export const selectedEnergy = signal(0);
export const selectedMaxEnergy = signal(100);
export const selectedAdrenaline = signal(1.0);
export const selectedAge = signal(0);

export const selectedGenome = signal("");
export const selectedMethylations = signal<number[]>([]);
export const selectedPhenotype = signal<any>(null);

export const speciesRosterSignal = signal<SpeciesRecord[]>([]);

// Accordion Toggles
export const isAliveExpanded = signal(true);
export const isExtinctExpanded = signal(false);

export function getLocusDescription(i: number): string {
  if (i === 0) return "Symmetry Profile (Quad vs Vertical)";
  if (i === 1) return "Muscle Strength / Primary Color Hue / Head Flattening";
  if (i === 2) return "Size Regulator / Primary Color Saturation / Parapodia Freq";
  if (i === 3) return "Sensory Acuity / Primary Color Lightness";
  if (i === 4) return "Secondary Color Hue";
  if (i === 5) return "Neural Tau (Integration Speed) / Secondary Color Saturation / Thermal Center";
  if (i === 6) return "Asymmetry Level / Secondary Color Lightness / Thermal Width";
  if (i === 7) return "Body Thickness (Mean Radius)";
  if (i === 8) return "Body Length (Base Length)";
  if (i >= 9 && i <= 11) return `Spinal Curve Amplitude Harmonic #${i - 8}`;
  if (i === 12) return "Biomorphic Stiffness (Elasticity)";
  if (i === 13) return "Pulse Speed / Stomach Capacity / Mitosis Threshold";
  if (i === 14) return "Spinal Curve Phase Shift / Sexual Maturity Age";
  if (i === 15) return "Wiggle Amplitude / Mitosis Energy Loss / Hydraulic Pressure";
  if (i === 16) return "Hidden Neurons Count (Brain Topology)";
  if (i >= 17 && i <= 20) return `Motor Output #${i - 16} Bias / Time Constant / Activation Style`;
  return `Synaptic Pathway Codon #${i - 20}`;
}

// Helper to compute active gene spans locally from chromatinState and genome
export function computeActiveGeneSpans(genome: string, chromatinState: boolean[]): GeneSpan[] {
  const spans: GeneSpan[] = [];
  if (!genome || !chromatinState) return spans;
  
  const currentLength = genome.length;
  let scanIdx = 16;
  while (scanIdx < currentLength - 1) {
    if (chromatinState[scanIdx] && chromatinState[scanIdx + 1]) {
      const charA = genome[scanIdx];
      const charB = genome[scanIdx + 1];
      const isStart = (charA === "S" && charB === "T") || (charA === "G" && charB === "O");

      if (isStart) {
        const geneStart = scanIdx;
        const payloadStart = scanIdx + 2;
        let payloadEnd = -1;
        let stopFoundAt = -1;

        for (let j = payloadStart; j < currentLength - 1; j++) {
          const cA = genome[j];
          const cB = genome[j + 1];
          const isStop = (cA === "S" && cB === "P") || (cA === "E" && cB === "N");

          if (isStop) {
            payloadEnd = j;
            stopFoundAt = j;
            break;
          }
        }

        if (payloadEnd === -1) {
          payloadEnd = currentLength;
          stopFoundAt = currentLength - 1;
        }

        const payloadLength = payloadEnd - payloadStart;
        if (payloadLength > 0) {
          spans.push({
            start: geneStart,
            end: Math.min(currentLength - 1, stopFoundAt + 1)
          });
        }
        scanIdx = Math.min(currentLength, stopFoundAt + 2);
      } else {
        scanIdx++;
      }
    } else {
      scanIdx++;
    }
  }
  return spans;
}
