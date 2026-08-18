import { 
  generateRandomGenome, 
  parseGenome, 
  CreaturePhenotype,
  ALPHABET,
  SensoryPatch,
  classifySensoryPatch,
  getComplementaryChar,
  getComplementaryString,
  mutateGenome
} from "./biology/dna";
import { CreatureRenderer } from "./render/creatureRenderer";
import { 
  initDb, 
  saveSpecies, 
  getAliveSpecies, 
  getAllSpecies,
  markSpeciesAsExtinct, 
  getSpeciesById, 
  clearDb,
  getSavedSimulationState,
  saveSimulationState,
  clearSimulationState,
  SpeciesRecord 
} from "./biology/speciesDb";

// Simulation Modes
type SimMode = "sandbox" | "evo";
let currentMode: SimMode = "sandbox";

// ==========================================
// 1. Single Sandbox Creature State
// ==========================================
let sandboxGenome = generateRandomGenome(256);
let sandboxAntisense = getComplementaryString(sandboxGenome);
let sandboxPhenotype: CreaturePhenotype;
let sandboxNeuronStates: number[] = [];
let sandboxNeuronActivations: number[] = [];

// Single-Creature Sandbox Physics
let spx = 256;            
let spy = 256;            
let svx = 0;              
let svy = 0;              
let sheadingAngle = -Math.PI / 2; 
let somegaRot = 0;        

// Bioluminescent Target (Sandbox manual food or fallback orbit)
let tx = 256;
let ty = 120;
let isMouseInsideCanvas = false;

// Active muscle contraction key tracking (Manual override)
const keys = {
  w: false,
  a: false,
  s: false,
  d: false,
  q: false,
  e: false
};

// ==========================================
// 2. Multi-Agent Evolution Mode State
// ==========================================
interface CreatureAgent {
  id: number;
  speciesId: string;  // Defined by its unique Sense DNA string
  genome: string;
  antisense: string;
  phenotype: CreaturePhenotype;
  px: number;
  py: number;
  vx: number;
  vy: number;
  headingAngle: number;
  omegaRot: number;
  energy: number;     // Current food reserves
  age: number;        // Age in frames
  generation: number; // Gen lineage
  adrenaline: number; // Endocrine hormonal adrenaline multiplier [1.0 to 1.8]
  hasEaten: boolean;  // Physiological feeding guarantee (must consume at least 1 spore/prey to reproduce!)
  
  // NATIVE RECURRENT CTRNN BRAIN STATES (Euler integration state holders):
  neuronStates: number[];
  neuronActivations: number[];
}

interface FoodSpore {
  x: number;
  y: number;
  vx: number;
  vy: number;
}

let creatures: CreatureAgent[] = [];
let foodPellets: FoodSpore[] = [];
let selectedAgentId: number | null = null;
let nextAgentId = 1;
let highestGeneration = 1;

// Fine-Grained Reactive DOM Elements cache (Eradicates 60fps document.getElementById layouts-thrashing!)
const svgElementCache = new Map<string, SVGElement>();

// High-Speed Synchronous In-Memory Cache (Eradicates 60fps database file-thrashing!)
let cachedAliveSpecies: SpeciesRecord[] = [];

// Tracking active species sizes on canvas to monitor Extinction events
let activeSpeciesOnCanvas = new Set<string>();

// Bite impact flash tracking [x, y, age]
let biteImpacts: { x: number; y: number; age: number }[] = [];

// Automatic periodic state save throttling
let lastStateSaveTime = 0;

// Global animation frame state
let lastTime = 0;        
let renderer: CreatureRenderer;

// DOM Elements
const canvas = document.getElementById("creature-canvas") as HTMLCanvasElement;
const shuffleBtn = document.getElementById("shuffle-btn") as HTMLButtonElement;
const mutateBtn = document.getElementById("mutate-btn") as HTMLButtonElement;
const genomeStringEl = document.getElementById("genome-string") as HTMLDivElement;
const genomeGridEl = document.getElementById("genome-grid") as HTMLDivElement;
const consoleLogsEl = document.getElementById("console-logs") as HTMLDivElement;

const coreTraitsList = document.getElementById("core-traits-list") as HTMLDivElement;
const organelleCountEl = document.getElementById("organelle-count") as HTMLSpanElement;
const organellesContainer = document.getElementById("emergent-organelles-container") as HTMLDivElement;

// Modes Switch & Stats Dashboard Widgets
const modeSandboxBtn = document.getElementById("mode-sandbox-btn") as HTMLButtonElement;
const modeEvoBtn = document.getElementById("mode-evo-btn") as HTMLButtonElement;
const evoStatsCard = document.getElementById("evo-stats-card") as HTMLDivElement;
const brainControlCard = document.getElementById("brain-control-card") as HTMLDivElement;
const sandboxButtonGroup = document.getElementById("sandbox-button-group") as HTMLDivElement;
const traitsHeaderTitle = document.getElementById("traits-header-title") as HTMLHeadingElement;

const evoPopCountEl = document.getElementById("evo-pop-count") as HTMLSpanElement;
const evoGenMaxEl = document.getElementById("evo-gen-max") as HTMLSpanElement;
const evoFoodCountEl = document.getElementById("evo-food-count") as HTMLSpanElement;

// Floating Inspect HUD Widgets (Top-Right)
const inspectOverlay = document.getElementById("inspect-overlay") as HTMLDivElement;
const closeInspectBtn = document.getElementById("close-inspect-btn") as HTMLButtonElement;
const inspectTraitsList = document.getElementById("inspect-traits-list") as HTMLDivElement;
const inspectLatinName = document.getElementById("inspect-latin-name") as HTMLHeadingElement;
const inspectDietClass = document.getElementById("inspect-diet-class") as HTMLParagraphElement;
const inspectGenerationTag = document.getElementById("inspect-generation-tag") as HTMLSpanElement;

// Evo Mode action buttons
const injectUrzelleBtn = document.getElementById("inject-urzelle-btn") as HTMLButtonElement;
const resetEvolutionBtn = document.getElementById("reset-evolution-btn") as HTMLButtonElement;

// Near-Fullscreen Diagnostics Modal Widgets (Stammbaum & Arten-Archiv)
const openDiagnosticsBtn = document.getElementById("open-diagnostics-btn") as HTMLButtonElement;
const diagnosticsModal = document.getElementById("diagnostics-modal") as HTMLDivElement;
const closeModalBtn = document.getElementById("close-modal-btn") as HTMLButtonElement;
const modalSpeciesList = document.getElementById("modal-species-list") as HTMLDivElement;
const modalStammbaumTreeContainer = document.getElementById("modal-stammbaum-tree-container") as HTMLDivElement;

// Genetic Specimen Drawer Widgets
const modalSpeciesDetails = document.getElementById("modal-species-details") as HTMLDivElement;
const closeDrawerBtn = document.getElementById("close-drawer-btn") as HTMLButtonElement;
const drawerLatinName = document.getElementById("drawer-latin-name") as HTMLHeadingElement;
const drawerSpeciesId = document.getElementById("drawer-species-id") as HTMLParagraphElement;
const drawerBrainVisualizer = document.getElementById("drawer-brain-visualizer") as HTMLDivElement;
const drawerGenomeGrid = document.getElementById("drawer-genome-grid") as HTMLDivElement;

let isDiagnosticsModalOpen = false;

// Neural AI Toggle Checkbox
const aiToggle = document.getElementById("ai-toggle") as HTMLInputElement;

/**
 * Custom console logging utility
 */
function logToConsole(message: string, type: "system" | "mutation" | "repair" = "system"): void {
  const entry = document.createElement("div");
  entry.className = "console-entry";
  
  if (type === "mutation") {
    entry.innerHTML = `<span class="mutation-log">[Basen-Paar]</span> ${message}`;
  } else if (type === "repair") {
    entry.innerHTML = `<span style="color: #10b981;">[Polymerase]</span> ${message}`;
  } else {
    entry.innerHTML = `<span class="highlight">[Epigenetik]</span> ${message}`;
  }

  consoleLogsEl.appendChild(entry);
  consoleLogsEl.scrollTop = consoleLogsEl.scrollHeight;
}

/**
 * Static Construction of the Evolved Deep Neural Synapse Web SVG.
 * Dynamically plots columns and connections for arbitrary input masks, outputs masks, and hidden layers inside the Sandbox or Inspect HUD!
 */
function renderSynapseWeb(phenotype: CreaturePhenotype, targetEl?: HTMLDivElement): void {
  // Target custom element if provided, otherwise default to context containers!
  const container = targetEl || document.getElementById(
    isDiagnosticsModalOpen 
      ? "modal-brain-visualizer-container" 
      : (currentMode === "sandbox" ? "brain-visualizer-container" : "inspect-brain-visualizer-container")
  ) as HTMLDivElement;
  if (!container) return;

  const b = phenotype.brain;
  const totalNodes = b.neurons.length;

  let svgContent = `<svg class="brain-svg" width="100%" height="135" viewBox="0 0 200 135">`;

  // Compute precise (x, y) coordinates for all N neurons
  const coords: { x: number; y: number }[] = Array(totalNodes);

  const inputs = b.neurons.filter(n => n.type === "input");
  const outputs = b.neurons.filter(n => n.type === "output");
  const hiddens = b.neurons.filter(n => n.type === "hidden");

  // A. Plot Input nodes (left column)
  inputs.forEach((n, idx) => {
    const x = 32;
    const y = 15 + idx * (105 / Math.max(1, inputs.length - 1)) + (inputs.length === 1 ? 52.5 : 0);
    coords[n.id] = { x, y };
  });

  // B. Plot Output nodes (right column)
  outputs.forEach((n, idx) => {
    const x = 168;
    const y = 15 + idx * (105 / Math.max(1, outputs.length - 1)) + (outputs.length === 1 ? 52.5 : 0);
    coords[n.id] = { x, y };
  });

  // C. Plot Hidden nodes (staggered middle column)
  hiddens.forEach((n, idx) => {
    const x = 100;
    const y = 15 + idx * (105 / Math.max(1, hiddens.length - 1)) + (hiddens.length === 1 ? 52.5 : 0);
    coords[n.id] = { x, y };
  });

  // 1. Draw Synaptic Connection lines/curves
  b.synapses.forEach(syn => {
    const pFrom = coords[syn.fromNode];
    const pTo = coords[syn.toNode];
    if (!pFrom || !pTo) return;

    const id = `synapse-${syn.fromNode}-${syn.toNode}`;
    const strokeColor = syn.weight > 0 ? "rgb(16, 185, 129)" : "rgb(239, 68, 68)";

    if (syn.fromNode === syn.toNode) {
      // Self-recurrent loop: draw as a beautiful curved loop path
      const r = 5;
      const x = pFrom.x;
      const y = pFrom.y;
      svgContent += `
        <path class="synapse-line" id="${id}" d="M ${x} ${y} A ${r} ${r} 0 1 1 ${x + 0.1} ${y - r*2}" 
              stroke="${strokeColor}" stroke-opacity="0.18" stroke-width="0.4" fill="none" />
      `;
    } else if (syn.fromNode > syn.toNode) {
      // Backward recurrent feedback loop: draw as a curved quadratic bezier
      const cx = (pFrom.x + pTo.x) / 2;
      const cy = (pFrom.y + pTo.y) / 2 - 25; // curve upwards
      svgContent += `
        <path class="synapse-line" id="${id}" d="M ${pFrom.x} ${pFrom.y} Q ${cx} ${cy} ${pTo.x} ${pTo.y}" 
              stroke="${strokeColor}" stroke-opacity="0.18" stroke-width="0.4" fill="none" />
      `;
    } else {
      // Straight feedforward line
      svgContent += `
        <line class="synapse-line" id="${id}" x1="${pFrom.x}" y1="${pFrom.y}" x2="${pTo.x}" y2="${pTo.y}" 
              stroke="${strokeColor}" stroke-opacity="0.18" stroke-width="0.4" />
      `;
    }
  });

  // 2. Draw Nodes circles & labels
  b.neurons.forEach(n => {
    const p = coords[n.id];
    if (!p) return;

    const id = `neuron-${n.id}`;
    svgContent += `
      <circle class="neuron-node" id="${id}" cx="${p.x}" cy="${p.y}" r="3.5" fill="#1f2937" stroke="rgba(255,255,255,0.12)" stroke-width="1.2" />
    `;

    // Render node labels beautifully
    if (n.type === "input") {
      svgContent += `
        <text x="${p.x - 7}" y="${p.y + 2.5}" fill="#8b9bb4" font-size="5.2" font-weight="bold" text-anchor="end">${n.label}</text>
      `;
    } else if (n.type === "output") {
      svgContent += `
        <text x="${p.x + 7}" y="${p.y + 2.5}" fill="#8b9bb4" font-size="5.2" font-weight="bold" text-anchor="start">${n.label}</text>
      `;
    }
  });

  svgContent += `</svg>`;
  container.innerHTML = svgContent;

  // Fine-Grained Reactivity: Clear previous caches and populate with direct SVG element references
  svgElementCache.clear();

  // Cache all neuron node elements
  b.neurons.forEach(n => {
    const id = `neuron-${n.id}`;
    const el = document.getElementById(id) as any;
    if (el) svgElementCache.set(id, el);
  });

  // Cache all synapse connection elements
  b.synapses.forEach(syn => {
    const id = `synapse-${syn.fromNode}-${syn.toNode}`;
    const el = document.getElementById(id) as any;
    if (el) svgElementCache.set(id, el);
  });
}

/**
 * Generates descriptive tooltips for each of the variable loci based on chromatin state and double-helix pairing.
 */
function getLocusTooltipText(idx: number, phenotype: CreaturePhenotype): string {
  const char = phenotype.genomeString[idx];
  const antiChar = phenotype.antisenseStrand[idx];
  const isComplementary = getComplementaryChar(char) === antiChar;
  const mShift = phenotype.methylations[idx];

  const pairingText = isComplementary 
    ? `Basenpaar: ${char}-${antiChar} (Stabil)` 
    : `MISMATCH: ${char}-${antiChar} (Unstabil)`;

  const mInfo = mShift !== 0 ? ` [Methyliert: ${mShift > 0 ? "+" : ""}${mShift}]` : "";

  if (idx === 0) {
    return `Locus 0: Symmetrie-Regulator\n${pairingText}${mInfo}\nA-M: Bilateral, N-Z: Quad`;
  }
  if (idx >= 1 && idx <= 3) {
    return `Locus ${idx}: Primärfarbe (H, S, L)\n${pairingText}${mInfo}`;
  }
  if (idx >= 4 && idx <= 6) {
    return `Locus ${idx}: Sekundärfarbe (H, S, L)\n${pairingText}${mInfo}`;
  }
  if (idx === 7) {
    return `Locus 7: Körper-Dicke (Notochord)\n${pairingText}${mInfo}\nExprimiert: ${Math.round(phenotype.spinalHarmonics.meanRadius)}px`;
  }
  if (idx === 8) {
    return `Locus 8: Spinal-Länge (Notochord)\n${pairingText}${mInfo}\nExprimiert: ${Math.round(phenotype.spinalHarmonics.baseLength)}px`;
  }
  if (idx === 9) {
    return `Locus 9: [Replikation] Insertion Slippage Rate\n${pairingText}${mInfo}\nExprimiert: ${Math.round(phenotype.insertionRate * 100)}%`;
  }
  if (idx === 10) {
    return `Locus 10: [Replikation] Deletion Slippage Rate\n${pairingText}${mInfo}\nExprimiert: ${Math.round(phenotype.deletionRate * 100)}%`;
  }
  if (idx === 11) {
    return `Locus 11: [Replikation] Reparatur-Fidelity\n${pairingText}${mInfo}\nExprimiert: ${Math.round(phenotype.repairFidelity * 100)}%`;
  }
  if (idx === 12) {
    return `Locus 12: [Biophysik] Starrheit / Stiffness\n${pairingText}${mInfo}\nExprimiert: ${Math.round(phenotype.stiffness * 100)}%`;
  }
  if (idx === 13) {
    return `Locus 13: [Biophysik] Atemfrequenz / Frequenz\n${pairingText}${mInfo}`;
  }
  if (idx === 14) {
    return `Locus 14: [Biophysik] Wellenphase / Delay\n${pairingText}${mInfo}`;
  }
  if (idx === 15) {
    return `Locus 15: [Biophysik] Strömungs-Sway Amplitude\n${pairingText}${mInfo}`;
  }
  if (idx === 16) {
    const hiddenCount = phenotype.brain.neurons.filter(n => n.type === "hidden").length;
    return `Locus 16: [🧠 CTRNN Hidden-Knoten] NEAT Topologie-Größe\n${pairingText}${mInfo}\nExprimiert: ${hiddenCount} Hidden Neuronen im Netzwerk`;
  }
  if (idx === 17 || idx === 18) {
    return `Locus ${idx}: [🧠 CTRNN Bias / Decay] Temporaler Koeffizient\n${pairingText}${mInfo}`;
  }
  if (idx === 19 || idx === 20) {
    return `Locus ${idx}: [🧠 CTRNN Output Konstanten]\n${pairingText}${mInfo}`;
  }

  if (!phenotype.chromatinState[idx]) {
    return `Locus ${idx}: Heterochromatin (CLOSED / Stumm)\n${pairingText}${mInfo}`;
  }

  for (let sIdx = 0; sIdx < phenotype.activeGeneSpans.length; sIdx++) {
    const span = phenotype.activeGeneSpans[sIdx];
    if (idx >= span.start && idx < span.end) {
      const organ = phenotype.organelles[sIdx];
      const identity = classifySensoryPatch(organ);
      if (idx === span.start || idx === span.start + 1) {
        return `Locus ${idx}: START-CODON [Euchromatin OPEN]\n${pairingText}${mInfo}\nOrgan: ${identity.name}`;
      }
      return `Locus ${idx}: Aktives Codon (Euchromatin)\n${pairingText}${mInfo}\nGen #${sIdx+1} (${identity.name})`;
    }
  }

  return `Locus ${idx}: Offenes Euchromatin (Dormantes Segment)\n${pairingText}${mInfo}`;
}

/**
 * Re-builds and updates the variable-character visual genome grid in the DOM,
 * displaying split Watson-Crick basenpaars.
 */
function updateGenomeGrid(phenotype: CreaturePhenotype, highlightIndex: number = -1, targetEl: HTMLDivElement = genomeGridEl): void {
  targetEl.innerHTML = "";
  const currentLength = phenotype.genomeString.length;
  
  // Synchronize raw genome text bar synchronously to prevent visual mismatch!
  if (targetEl === genomeGridEl && genomeStringEl) {
    genomeStringEl.innerText = phenotype.genomeString;
  }

  if (currentLength >= 180) {
    targetEl.style.gridTemplateColumns = "repeat(32, 1fr)";
  } else {
    targetEl.style.gridTemplateColumns = "repeat(16, 1fr)";
  }

  for (let i = 0; i < currentLength; i++) {
    const char = phenotype.genomeString[i];
    const antiChar = phenotype.antisenseStrand[i];
    const isComplementary = getComplementaryChar(char) === antiChar;
    const type = getLocusType(i, phenotype);
    const mShift = phenotype.methylations[i];
    
    const locusDiv = document.createElement("div");
    locusDiv.className = "locus";
    
    locusDiv.style.display = "flex";
    locusDiv.style.flexDirection = "column";
    locusDiv.style.justifyContent = "space-between";
    locusDiv.style.padding = "1px 0";

    if (type === "body-chassis") {
      locusDiv.classList.add("active-gene");
      locusDiv.style.borderLeft = "2px solid #00f2fe";
    } else if (type === "promoter") {
      locusDiv.classList.add("promoter-gene");
    } else if (type === "active") {
      locusDiv.classList.add("active-gene");
    } else {
      locusDiv.classList.add("junk-gene");
    }

    const baseVal = ALPHABET.indexOf(char);
    const mValue = (baseVal + mShift + 26) % 26;
    const hue = (mValue / 26) * 360;
    
    if (phenotype.chromatinState[i]) {
      locusDiv.style.borderColor = `hsl(${hue}, 85%, 55%)`;
    } else {
      locusDiv.style.borderColor = "#2c3345";
    }
    locusDiv.style.borderWidth = "1px";
    locusDiv.style.borderStyle = "solid";

    if (mShift !== 0) {
      locusDiv.style.boxShadow = "inset 0 0 5px #ff4a85";
    }

    // Uncomplementary replication errors glow red
    if (!isComplementary) {
      locusDiv.style.backgroundColor = "rgba(239, 68, 68, 0.18)";
      locusDiv.style.borderColor = "#ef4444";
      locusDiv.style.boxShadow = "0 0 10px #ef4444";
    }

    const senseSpan = document.createElement("span");
    senseSpan.style.fontSize = "0.38rem";
    senseSpan.style.fontWeight = "bold";
    senseSpan.style.lineHeight = "1.1";
    senseSpan.style.color = phenotype.chromatinState[i] ? "rgba(0, 242, 254, 0.95)" : "rgba(255, 255, 255, 0.18)";
    senseSpan.innerText = ALPHABET[mValue];

    const divider = document.createElement("div");
    divider.style.height = "1px";
    divider.style.background = isComplementary ? "rgba(255, 255, 255, 0.06)" : "rgba(239, 68, 68, 0.35)";

    const antiSpan = document.createElement("span");
    antiSpan.style.fontSize = "0.38rem";
    antiSpan.style.fontWeight = "bold";
    antiSpan.style.lineHeight = "1.1";
    antiSpan.style.color = phenotype.chromatinState[i] ? "rgba(192, 132, 252, 0.95)" : "rgba(255, 255, 255, 0.12)";
    antiSpan.innerText = antiChar;

    locusDiv.appendChild(senseSpan);
    locusDiv.appendChild(divider);
    locusDiv.appendChild(antiSpan);
    
    const tooltipText = getLocusTooltipText(i, phenotype);
    const tooltip = document.createElement("span");
    tooltip.className = "locus-tooltip";
    tooltip.innerText = tooltipText;
    locusDiv.appendChild(tooltip);

    if (i === highlightIndex) {
      locusDiv.classList.add("mutated-flash");
      setTimeout(() => {
        locusDiv.classList.remove("mutated-flash");
      }, 1200);
    }

    targetEl.appendChild(locusDiv);
  }
}

/**
 * 60fps Real-Time Neural Signal Propagator (Updates visual glows, line thickness and current flow on active nodes)
 */
function updateLiveNeuralActivity(allLayerActivations: number[][], brain: any): void {
  const activations = allLayerActivations[0]; // flat array of totalNodes

  // 1. Update Layer Nodes using our high-speed direct reference cache!
  brain.neurons.forEach((n: any) => {
    const id = `neuron-${n.id}`;
    const el = svgElementCache.get(id);
    if (el) {
      // Clamp raw activation strictly to [0.0, 1.0] before applying exponent
      const rawAct = Math.max(0.0, Math.min(1.0, Math.abs(activations[n.id])));
      const act = Math.pow(rawAct, 4.0); // Clean 4th-power high-contrast spark
      
      // Clean, tidy scale-limits: radius grows from 3.5px (neutral) to 5.0px (peak flash)
      const radius = 3.5 + act * 1.5;
      const colorGlow = n.type === "input" ? "#00f2fe" : (n.type === "output" ? "#38bdf8" : "#c084fc");
      const glow = act > 0.45 ? `0 0 5px ${colorGlow}` : "none";
      const fill = act > 0.35 ? `rgba(0, 242, 254, 0.9)` : "#111827";
      
      el.setAttribute("r", radius.toString());
      el.style.fill = fill;
      el.style.filter = glow !== "none" ? `drop-shadow(${glow})` : "none";
    }
  });

  // 2. Update Synaptic Connections (Flow currents) using direct reference cache!
  brain.synapses.forEach((syn: any) => {
    const id = `synapse-${syn.fromNode}-${syn.toNode}`;
    const el = svgElementCache.get(id);
    if (el) {
      const weight = syn.weight;
      if (Math.abs(weight) < 0.15) {
        // Faint, almost dead synapses stay very dark gray
        el.setAttribute("stroke", "rgba(255,255,255,0.02)");
        el.setAttribute("stroke-opacity", "0.02");
        el.setAttribute("stroke-width", "0.3");
        el.style.filter = "none";
        return;
      }

      // Query the activation of the source node
      const preAct = activations[syn.fromNode];
      const rawSig = Math.max(0.0, Math.min(1.0, Math.abs(preAct * weight) / 2.0));
      const current = Math.pow(rawSig, 4.0); 

      if (current > 0.12) {
        // ACTIVE SYNAPSE: Flash to vivid biological colors!
        const color = weight > 0 ? "rgb(16, 185, 129)" : "rgb(239, 68, 68)";
        const opacity = Math.min(0.85, 0.25 + current * 0.8);
        // Synapse thickness scales tightly from 0.4px (neutral) to 1.4px (peak flash)
        const thickness = 0.5 + current * 1.0;
        const glow = current > 0.45 ? `drop-shadow(0 0 4px ${color})` : "none";

        el.setAttribute("stroke", color);
        el.setAttribute("stroke-opacity", opacity.toString());
        el.setAttribute("stroke-width", thickness.toString());
        el.style.filter = glow;
      } else {
        // INACTIVE SKELETON: Clean, visible light-gray connection!
        el.setAttribute("stroke", "rgb(139, 155, 180)");
        el.setAttribute("stroke-opacity", "0.18");
        el.setAttribute("stroke-width", "0.4");
        el.style.filter = "none";
      }
    }
  });
}

/**
 * Embodied Cognition: Decodes brain inputs strictly based on the presence, scaling,
 * and body placement of the creature's actually expressed sensory organelles!
 * Returns an array of size K + 1 (the K organelle values + the internal hunger clock).
 */
function computeEmbodiedSensoryInputs(
  phenotype: CreaturePhenotype, 
  px: number,
  py: number,
  headingAngle: number,
  vx: number,
  vy: number,
  omegaRot: number,
  clockVal: number,
  currentAgentId: number
): number[] {
  const K = phenotype.organelles.length;
  const inputs: number[] = Array(K + 1).fill(0.0);

  // Set the final input node to be the Hunger Clock oscillator (internal sense)
  inputs[K] = clockVal;

  const dpr = window.devicePixelRatio || 1;
  const logicalWidth = canvas.width / dpr;
  const logicalHeight = canvas.height / dpr;

  phenotype.organelles.forEach((patch, idx) => {
    const aff = patch.spectralAffinity;
    const organPower = patch.scale * (1.1 - patch.bandwidth);
    const range = patch.scale * 350.0; // Expanded sensory range horizon!
    
    // Calculate organ's looking angle in radians (0 is forward, negative is right, positive is left)
    const alpha = (patch.angle - 90) * (Math.PI / 180);
    const halfCone = Math.max(0.1, patch.bandwidth * 1.5); // view cone width in radians

    let maxStimulus = 0.0;

    // A. Scan all Food Spores
    foodPellets.forEach(pellet => {
      let dx = pellet.x - px;
      let dy = pellet.y - py;
      if (dx > logicalWidth / 2) dx -= logicalWidth;
      if (dx < -logicalWidth / 2) dx += logicalWidth;
      if (dy > logicalHeight / 2) dy -= logicalHeight;
      if (dy < -logicalHeight / 2) dy += logicalHeight;

      const dist = Math.sqrt(dx*dx + dy*dy);
      if (dist <= range) {
        let angleRel = Math.atan2(dy, dx) - headingAngle;
        while (angleRel > Math.PI) angleRel -= Math.PI * 2;
        while (angleRel < -Math.PI) angleRel += Math.PI * 2;

        let deltaBeta = angleRel - alpha;
        while (deltaBeta > Math.PI) deltaBeta -= Math.PI * 2;
        while (deltaBeta < -Math.PI) deltaBeta += Math.PI * 2;

        if (Math.abs(deltaBeta) <= halfCone) {
          // Food spore signatures: visual=green (0.33), smell=sweet (0.15), vibration=still (0.05)
          let match = 0.0;
          if (aff >= 0.8) {
            // Photoreceptor matches Visual Hue
            match = Math.max(0, 1.0 - Math.abs(aff - 0.33) / (patch.bandwidth * 1.8 + 0.12));
          } else if (aff >= 0.25 && aff <= 0.65) {
            // Chemoreceptor matches Olfactory smell
            match = Math.max(0, 1.0 - Math.abs(aff - 0.15) / (patch.bandwidth * 1.8 + 0.12));
          } else if (aff < 0.25) {
            // Mechanoreceptor matches vibration
            match = Math.max(0, 1.0 - Math.abs(aff - 0.05) / Math.max(0.05, patch.bandwidth));
          }

          if (match > 0.05) {
            const strength = match * organPower * (1.0 - dist / range) * Math.cos(deltaBeta);
            maxStimulus = Math.max(maxStimulus, strength);
          }
        }
      }
    });

    // B. Scan all other Creature Agents in the tank (Prey and Predators)
    creatures.forEach(other => {
      if (other.id === currentAgentId) return;

      let dx = other.px - px;
      let dy = other.py - py;
      if (dx > logicalWidth / 2) dx -= logicalWidth;
      if (dx < -logicalWidth / 2) dx += logicalWidth;
      if (dy > logicalHeight / 2) dy -= logicalHeight;
      if (dy < -logicalHeight / 2) dy += logicalHeight;

      const dist = Math.sqrt(dx*dx + dy*dy);
      if (dist <= range) {
        let angleRel = Math.atan2(dy, dx) - headingAngle;
        while (angleRel > Math.PI) angleRel -= Math.PI * 2;
        while (angleRel < -Math.PI) angleRel += Math.PI * 2;

        let deltaBeta = angleRel - alpha;
        while (deltaBeta > Math.PI) deltaBeta -= Math.PI * 2;
        while (deltaBeta < -Math.PI) deltaBeta += Math.PI * 2;

        if (Math.abs(deltaBeta) <= halfCone) {
          // Derive Target Signatures dynamically based on other's expressed genome!
          const targetVisual = other.phenotype.primaryColor.h / 360;
          const targetSmell = (other.phenotype.basalMetabolicRate % 100) / 100;
          const targetVibration = (other.phenotype.pulseSpeed * 1000) % 1.0;
          const targetHeat = other.phenotype.isPredator ? 0.85 * other.adrenaline : 0.15;

          let match = 0.0;
          if (aff >= 0.8) {
            // Photoreceptor matches target visual HSL hue ratio
            match = Math.max(0, 1.0 - Math.abs(aff - targetVisual) / (patch.bandwidth * 1.8 + 0.12));
          } else if (aff >= 0.25 && aff <= 0.65) {
            // Chemoreceptor matches target olfactory smell signature
            match = Math.max(0, 1.0 - Math.abs(aff - targetSmell) / (patch.bandwidth * 1.8 + 0.12));
          } else if (aff < 0.25) {
            // Mechanoreceptor matches target auditory/vibrational frequency signature
            match = Math.max(0, 1.0 - Math.abs(aff - targetVibration) / (patch.bandwidth * 1.8 + 0.12));
          } else {
            // Thermoreceptor matches target metabolic heat signature
            match = Math.max(0, 1.0 - Math.abs(aff - targetHeat) / (patch.bandwidth * 1.8 + 0.12));
          }

          if (match > 0.05) {
            const strength = match * organPower * (1.0 - dist / range) * Math.cos(deltaBeta);
            maxStimulus = Math.max(maxStimulus, strength);
          }
        }
      }
    });

    // C. Scan physical boundary hydrostatic touch pressure (Mechanoreceptors feel the wall warn-zone!)
    if (aff < 0.25) {
      const wallWarningZone = range * 0.5; // warning zone proportional to receptor range
      let boundaryPressure = 0.0;
      
      if (px < wallWarningZone) boundaryPressure = 1.0 - px / wallWarningZone;
      else if (px > logicalWidth - wallWarningZone) boundaryPressure = 1.0 - (logicalWidth - px) / wallWarningZone;

      if (py < wallWarningZone) boundaryPressure = Math.max(boundaryPressure, 1.0 - py / wallWarningZone);
      else if (py > logicalHeight - wallWarningZone) boundaryPressure = Math.max(boundaryPressure, 1.0 - (logicalHeight - py) / wallWarningZone);

      if (boundaryPressure > 0.0) {
        // Wall warning is an omnidirectional warning pressure scaled by organ power
        const strength = boundaryPressure * organPower;
        maxStimulus = Math.max(maxStimulus, strength);
      }
    }

    // D. Scan proprioceptive self-movement (Mechanoreceptors feel water shear flow and rotation!)
    if (aff < 0.25) {
      const speed = Math.sqrt(vx * vx + vy * vy);
      const rotSpeed = Math.abs(omegaRot);
      // Fluid shear flow and centrifugal force perception scaled by organ power
      const proprioceptiveStimulus = Math.min(1.0, speed * 0.15 + rotSpeed * 0.35);
      
      if (proprioceptiveStimulus > 0.0) {
        const strength = proprioceptiveStimulus * organPower;
        maxStimulus = Math.max(maxStimulus, strength);
      }
    }

    inputs[idx] = Math.max(0.0, Math.min(1.0, maxStimulus));
  });

  return inputs;
}

/**
 * Dynamic Deep MLP Feedforward signal execution
 */
function executeBrain(
  brain: any, 
  inputs: number[], 
  neuronStates: number[], 
  neuronActivations: number[]
): { outputs: number[], allLayerActivations: number[][] } {
  const totalNodes = brain.neurons.length;

  // Initialize or resize temporal membrane states in-memory instantly!
  if (neuronStates.length !== totalNodes) {
    neuronStates.length = totalNodes;
    neuronStates.fill(0.0);
  }
  if (neuronActivations.length !== totalNodes) {
    neuronActivations.length = totalNodes;
    neuronActivations.fill(0.0);
  }

  const K = inputs.length - 1; // last input is the clock

  // 1. Direct Sensory Follow: Assign sensory inputs directly to input nodes
  for (let i = 0; i <= K; i++) {
    neuronActivations[i] = inputs[i];
  }

  // 2. Continuous Euler Integration: Update hidden and output neurons
  for (let i = K + 1; i < totalNodes; i++) {
    const neuron = brain.neurons[i];

    // Accumulate inputs from all incoming temporal synapses
    let sum = 0.0;
    brain.synapses.forEach((syn: any) => {
      if (syn.toNode === i) {
        sum += neuronActivations[syn.fromNode] * syn.weight;
      }
    });

    // Euler integration step (dt = 1.0, tau_i is the genetically encoded decay time)
    neuronStates[i] += (1.0 / neuron.tau) * (-neuronStates[i] + sum + neuron.bias);

    // Bounded potential clamping to prevent numeric drift explosions
    neuronStates[i] = Math.max(-4.0, Math.min(4.0, neuronStates[i]));

    // Sigmoidal / Tanh activation function
    neuronActivations[i] = Math.tanh(neuronStates[i]);
  }

  // 3. Map output node activations directly to the 4 motor directions
  // Output nodes are compiled at indices K+1 to K+4
  const outputs = [
    neuronActivations[K + 1],
    neuronActivations[K + 2],
    neuronActivations[K + 3],
    neuronActivations[K + 4]
  ];

  // Return the activations as a flat array for fine-grained direct visualization
  return { outputs, allLayerActivations: [neuronActivations] };
}

/**
 * Returns locus classification (promoter, active, or junk) for visual rendering.
 */
function getLocusType(idx: number, phenotype: CreaturePhenotype): "body-chassis" | "promoter" | "active" | "junk" {
  if (idx < 16) {
    return "body-chassis";
  }

  if (!phenotype.chromatinState[idx]) {
    return "junk";
  }

  for (const span of phenotype.activeGeneSpans) {
    if (idx >= span.start && idx < span.end) {
      if (idx === span.start || idx === span.start + 1) {
        return "promoter";
      }
      if (idx === span.end - 1 || idx === span.end - 2) {
        return "promoter";
      }
      return "active";
    }
  }

  return "junk";
}

/**
 * Calculates biomechanical parameters and updates the traits list with live stats,
 * including Latin Taxonomy binomial nomenclature, Sensory Meter Bars, and Survival Expectation.
 */
function updateTraitsDashboard(phenotype: CreaturePhenotype, speedX = 0, speedY = 0, liveThrust = 0, liveMass = 1, dragFwd = 0, dragLat = 0, headingAngle = -Math.PI / 2): void {
  const breathingHz = Math.round(phenotype.pulseSpeed * 1000 * 10) / 10;
  const speedMag = Math.sqrt(speedX * speedX + speedY * speedY);
  const displayHeadingDeg = Math.round(((headingAngle * 180) / Math.PI + 360) % 360);
  
  const score = phenotype.survivalExpectation;
  let scoreColor = "#ef4444";
  let scoreText = "Kritisch";
  
  if (score >= 80) {
    scoreColor = "#10b981";
    scoreText = "Hervorragend";
  } else if (score >= 60) {
    scoreColor = "#34d399";
    scoreText = "Gut";
  } else if (score >= 40) {
    scoreColor = "#fbbf24";
    scoreText = "Mittelmäßig";
  } else if (score >= 20) {
    scoreColor = "#f97316";
    scoreText = "Gefährdet";
  }

  const limbsCount = phenotype.organelles.filter(node => node.expressionStyle >= 0.72).length;

  coreTraitsList.innerHTML = `
    <!-- Species Taxonomy Card -->
    <div class="species-card">
      <div class="species-latin" style="display: flex; justify-content: space-between; align-items: center;">
        <span>${phenotype.latinName}</span>
        <span style="font-size: 0.65rem; padding: 2px 6px; border-radius: 4px; font-weight: bold; background: ${phenotype.isPredator ? "rgba(239, 68, 68, 0.15)" : "rgba(16, 185, 129, 0.15)"}; color: ${phenotype.isPredator ? "rgb(239, 68, 68)" : "rgb(16, 185, 129)"};">
          ${phenotype.isPredator ? "Raubtier (Karnivor)" : "Beute (Filtrierer)"}
        </span>
      </div>
      <div class="species-class">${phenotype.dietClass}</div>
      <div class="species-class" style="color: #00f2fe; font-size: 0.68rem; margin-top: 2px;">Zone: ${phenotype.preferredHabitat}</div>
    </div>

    <!-- Dynamic Survival Expectation Box -->
    <div class="survival-box">
      <div class="survival-header">
        <span>Überlebenschance</span>
        <span style="color: ${scoreColor}; font-weight: 800;">${scoreText}</span>
      </div>
      <div style="display: flex; align-items: center; gap: 12px; margin-top: 2px;">
        <span class="survival-percentage" style="color: ${scoreColor};">${score}%</span>
        <div class="survival-bar-bg">
          <div class="survival-bar-fill" style="width: ${score}%; background-color: ${scoreColor}; box-shadow: 0 0 8px ${scoreColor};"></div>
        </div>
      </div>
      <div class="survival-summary">${phenotype.survivalAnalysis}</div>
    </div>

    <!-- Biomechanical Traits -->
    <div class="trait-list">
      <div class="trait-item">
        <span class="trait-label font-bold" style="color:var(--color-promoter);">Genom-Größe:</span>
        <span class="trait-val" style="color:var(--color-promoter); font-weight:700;">${phenotype.genomeString.length} Loci</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Replikations-Slippage (Ins / Del):</span>
        <span class="trait-val" style="color:#d946ef;">${Math.round(phenotype.insertionRate*100)}% / ${Math.round(phenotype.deletionRate*100)}%</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">DNA-Reparatur-Fidelity:</span>
        <span class="trait-val" style="color: #10b981; font-weight:700;">${Math.round(phenotype.repairFidelity * 100)}%</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Emergente Gliederung:</span>
        <span class="trait-val" style="color: #38bdf8; font-weight:700;">${phenotype.emergentChambersCount} Segmente</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Exprimierte Gliedmaßen:</span>
        <span class="trait-val" style="color: #a5f3fc;">${limbsCount} Muskelfüße</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Aggression / Karnivorie:</span>
        <span class="trait-val" style="color: #ef4444; font-weight:700;">${Math.round(phenotype.carnivory * 100)}%</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Zell-Masse / Elastizität:</span>
        <span class="trait-val">${Math.round(liveMass)} μg / ${Math.round(phenotype.stiffness * 100)}%</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Atemfrequenz / Schub (F):</span>
        <span class="trait-val">${breathingHz} Hz / ${Math.round(liveThrust * 10) / 10} mN</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Strömungswiderstand (Fwd/Lat):</span>
        <span class="trait-val" style="color: #fda4af;">${Math.round(dragFwd*10)/10} / ${Math.round(dragLat*10)/10} Ns</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Geschwindigkeit (Velocity):</span>
        <span class="trait-val" style="color: var(--color-active);">${Math.round(speedMag * 100) / 100} px/f</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Ausrichtung (Heading):</span>
        <span class="trait-val" style="color: #fca5a5;">${displayHeadingDeg}°</span>
      </div>
    </div>

    <!-- Dynamic Sensory Panel -->
    <div class="sensory-panel">
      <h4 style="font-size: 0.65rem; text-transform: uppercase; color: var(--text-muted); letter-spacing: 1px; margin-bottom: 4px;">Perzeptives Spektrum</h4>
      
      <div class="sensor-row">
        <span class="sensor-name">Visus (Photonen/Licht)</span>
        <div class="sensor-bar">
          <div class="sensor-fill" style="width: ${phenotype.sensoryVisus}%; background-color: #a21caf; box-shadow: 0 0 5px #a21caf;"></div>
        </div>
        <span class="sensor-val">${phenotype.sensoryVisus}%</span>
      </div>

      <div class="sensor-row">
        <span class="sensor-name">Olfaktorik (Moleküle)</span>
        <div class="sensor-bar">
          <div class="sensor-fill" style="width: ${phenotype.sensoryOlfaction}%; background-color: #15803d; box-shadow: 0 0 5px #15803d;"></div>
        </div>
        <span class="sensor-val">${phenotype.sensoryOlfaction}%</span>
      </div>

      <div class="sensor-row">
        <span class="sensor-name">Taktilität (Druckwellen)</span>
        <div class="sensor-bar">
          <div class="sensor-fill" style="width: ${phenotype.sensoryTactility}%; background-color: #b45309; box-shadow: 0 0 5px #b45309;"></div>
        </div>
        <span class="sensor-val">${phenotype.sensoryTactility}%</span>
      </div>

      <div class="sensor-row">
        <span class="sensor-name">Thermotaktik (Infrarot)</span>
        <div class="sensor-bar">
          <div class="sensor-fill" style="width: ${phenotype.sensoryBiolum}%; background-color: #e11d48; box-shadow: 0 0 5px #e11d48;"></div>
        </div>
        <span class="sensor-val">${phenotype.sensoryBiolum}%</span>
      </div>
    </div>
  `;

  // Update Expressed Organelles Counter
  organelleCountEl.innerText = phenotype.organelles.length.toString();

  // Populate Organelles Scrollbox
  organellesContainer.innerHTML = "";
  if (phenotype.organelles.length === 0) {
    organellesContainer.innerHTML = `
      <div style="color: var(--text-muted); font-size: 0.8rem; text-align: center; margin-top: 30px;">
        Keine Rezeptor-Patches exprimiert.<br>Membran ist vollkommen glatt.
      </div>
    `;
    return;
  }

  phenotype.organelles.forEach((node, idx) => {
    const identity = classifySensoryPatch(node);
    
    const spectralPercent = Math.round(node.spectralAffinity * 100);
    const specPercent = Math.round((1.0 - node.bandwidth) * 100);
    const stylePercent = Math.round(node.expressionStyle * 100);

    let spectralTypeName = "Mechanisch";
    if (node.spectralAffinity >= 0.8) spectralTypeName = "Photonen/Licht";
    else if (node.spectralAffinity >= 0.25 && node.spectralAffinity <= 0.65) spectralTypeName = "Moleküle/Chemo";
    else if (node.spectralAffinity > 0.65 && node.spectralAffinity < 0.8) spectralTypeName = "Infrarot/Wärme";

    const card = document.createElement("div");
    card.className = "organelle-card";
    card.innerHTML = `
      <div class="organelle-card-meta">
        <div class="organelle-card-title">
          <span class="organelle-dot" style="background-color: ${identity.color}; box-shadow: 0 0 6px ${identity.color};"></span>
          <span>Gen #${idx + 1}: ${identity.name}</span>
        </div>
        <div class="organelle-card-loci">Loci ${node.geneStartIndex}-${node.geneEndIndex}</div>
        <div style="font-size:0.65rem; color:var(--text-muted); font-style:italic; margin-top:2px;">${identity.desc}</div>
      </div>
      <div>
        <div class="organelle-card-val" style="color: ${identity.color}; font-weight:700;">λ: ${spectralPercent}% (${spectralTypeName})</div>
        <div class="organelle-card-loci">Spezialisierung: ${specPercent}%</div>
        <div class="organelle-card-loci">Form (Haar): ${stylePercent}% | Pos: ${Math.round(node.spinalPos * 100)}%</div>
      </div>
    `;
    organellesContainer.appendChild(card);
  });
}

/**
 * Dedicated renderer inside the Top-Right Inspect HUD Overlay.
 * Renders real-time glowing progress vital bars (energy, senescence, adrenaline, and lineages).
 */
function updateInspectTraitsDashboard(agent: CreatureAgent): void {
  const energyPercent = Math.round((agent.energy / agent.phenotype.stomachCapacity) * 100);
  const agePercent = Math.round((agent.age / 2700) * 100);

  inspectLatinName.innerText = agent.phenotype.latinName;
  inspectDietClass.innerHTML = `
    <span>${agent.phenotype.dietClass}</span>
  `;
  inspectGenerationTag.innerText = `${agent.generation}. Gen`;

  inspectTraitsList.innerHTML = `
    <!-- Energy progress bar -->
    <div class="vital-bar-container">
      <span class="vital-bar-label">⚡ Energie:</span>
      <div class="vital-bar-bg">
        <div class="vital-bar-fill" style="width: ${energyPercent}%; background-color: #10b981; box-shadow: 0 0 8px #10b981;"></div>
      </div>
      <span style="font-weight:bold; color:#10b981; min-width:32px; text-align:right;">${energyPercent}%</span>
    </div>

    <!-- Lifespan progress bar -->
    <div class="vital-bar-container" style="margin-top:6px;">
      <span class="vital-bar-label">⌛ Alterung:</span>
      <div class="vital-bar-bg">
        <div class="vital-bar-fill" style="width: ${agePercent}%; background-color: #38bdf8; box-shadow: 0 0 8px #38bdf8;"></div>
      </div>
      <span style="font-weight:bold; color:#38bdf8; min-width:32px; text-align:right;">${agePercent}%</span>
    </div>

    <!-- Adrenaline progress bar -->
    <div class="vital-bar-container" style="margin-top:6px;">
      <span class="vital-bar-label">🔥 Adrenalin:</span>
      <div class="vital-bar-bg">
        <div class="vital-bar-fill" style="width: ${Math.round(((agent.adrenaline - 1.0)/0.8) * 100)}%; background-color: #f43f5e; box-shadow: 0 0 8px #f43f5e;"></div>
      </div>
      <span style="font-weight:bold; color:#f43f5e; min-width:32px; text-align:right;">${Math.round(agent.adrenaline * 100)}%</span>
    </div>

    <!-- Lineage info -->
    <div class="trait-list" style="margin-top: 10px;">
      <div class="trait-item">
        <span class="trait-label">Biomasse / Elastizität:</span>
        <span class="trait-val">${Math.round(agent.phenotype.spinalHarmonics.baseLength * agent.phenotype.spinalHarmonics.meanRadius)} μg² / ${Math.round(agent.phenotype.stiffness * 100)}%</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Genom-Länge:</span>
        <span class="trait-val" style="color:var(--color-promoter); font-weight:700;">${agent.genome.length} Loci</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Stammbaum-Mutter:</span>
        <span class="trait-val" style="color:#d946ef; font-size:0.6rem; max-width:140px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" title="${agent.generation > 1 ? agent.speciesId : "Gründerzelle"}">
          ${agent.generation > 1 ? agent.speciesId.substring(0, 16) + "..." : "Gründer-Urzelle"}
        </span>
      </div>
    </div>
  `;
}

/**
 * Dedicated renderer inside the Near-Fullscreen Cybernetic Diagnostics Modal Dashboard.
 * Dynamically populates highly detailed real-time clinical readings and scrollable organelle cards.
 */
/**
 * Compiles a single node of the Stammbaum lineage tree recursively, nesting children under parents!
 */
function compileStammbaumNodeHTML(parent: SpeciesRecord, list: SpeciesRecord[]): string {
  const children = list.filter(rec => rec.parentSpeciesId === parent.id);
  const statusColor = parent.status === "alive" ? "#00f2fe" : "#555b70";
  const dotColor = parent.status === "alive" ? "#10b981" : "#ef4444";
  
  let html = `
    <div style="display:flex; flex-direction:column; align-items:flex-start; margin-top: 4px;">
      <div class="stammbaum-node" data-id="${parent.id}" style="border-left: 2px solid ${statusColor};">
        <div style="display:flex; justify-content:space-between; align-items:center; width:100%; gap:12px;">
          <span class="stammbaum-title" style="color:${parent.status === "alive" ? "#fff" : "var(--text-muted); font-style:italic;"}">${parent.name}</span>
          <span style="font-size:0.6rem; color:var(--text-muted); font-weight:bold;">Gen ${parent.generation}</span>
        </div>
        <div class="stammbaum-meta" style="margin-top: 2px;">
          <span style="display:flex; align-items:center;">
            <span class="roster-dot" style="background-color:${dotColor};"></span>
            ${parent.status === "alive" ? "Lebende Spezies" : "Ausgestorben"}
          </span>
          <span>Peak: ${parent.peakPopulation}</span>
        </div>
        <div style="font-size:0.52rem; color:rgba(255,255,255,0.14); font-family:monospace; max-width:260px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-top:2px;" title="${parent.id}">
          ID: ${parent.id.substring(0, 20)}...
        </div>
      </div>
  `;
  
  if (children.length > 0) {
    html += `<div class="stammbaum-branch">`;
    children.forEach(child => {
      html += compileStammbaumNodeHTML(child, list);
    });
    html += `</div>`;
  }
  
  html += `</div>`;
  return html;
}

/**
 * Main renderer of the Widescreen Stammbaum and Species Archive Modal.
 * Pulls all recorded lineages from disk and compiles a structured, nested family tree.
 */
function updateStammbaumAndSpeciesModal(): void {
  getAllSpecies().then(speciesList => {
    // 1. Compile Global Species List (Arten-Archiv)
    let listHTML = "";
    speciesList.forEach(rec => {
      const birthDate = new Date(rec.birthTime).toLocaleTimeString();
      const statusColor = rec.status === "alive" ? "#10b981" : "#4d5974";
      const statusText = rec.status === "alive" ? "Lebend" : "Fossil";
      const statusBg = rec.status === "alive" ? "rgba(16, 185, 129, 0.12)" : "rgba(77, 89, 116, 0.12)";
      
      listHTML += `
        <div class="roster-item" data-id="${rec.id}" style="border-left: 2.5px solid ${statusColor}; margin-bottom: 6px;">
          <div style="display:flex; justify-content:space-between; align-items:center; pointer-events: none;">
            <span class="roster-title" style="font-size:0.75rem; font-weight:bold;">${rec.name}</span>
            <span style="font-size:0.58rem; padding:1.5px 5px; border-radius:3px; font-weight:bold; background:${statusBg}; color:${statusColor};">
              ${statusText}
            </span>
          </div>
          <div class="roster-meta" style="margin-top:2px; pointer-events: none;">
            <span>Gen: ${rec.generation} | Peak-Pop: ${rec.peakPopulation}</span>
            <span style="font-size:0.58rem; color:var(--text-muted);">${birthDate}</span>
          </div>
        </div>
      `;
    });

    if (speciesList.length === 0) {
      listHTML = `<div style="color:var(--text-muted); text-align:center; font-size:0.75rem; margin-top:40px;">Noch keine Datensätze registriert.</div>`;
    }
    modalSpeciesList.innerHTML = listHTML;

    // 2. Compile Hierarchical Stammbaum Lineage Tree (Nesting descendants under ancestors!)
    const activeIds = new Set(speciesList.map(rec => rec.id));
    const roots = speciesList.filter(rec => !rec.parentSpeciesId || !activeIds.has(rec.parentSpeciesId));

    let treeHTML = `<div style="display: flex; flex-direction: column; gap: 14px; align-items: flex-start; padding-bottom: 30px;">`;
    roots.forEach(root => {
      treeHTML += compileStammbaumNodeHTML(root, speciesList);
    });
    treeHTML += `</div>`;

    if (speciesList.length === 0) {
      treeHTML = `<div style="color:var(--text-muted); text-align:center; font-size:0.75rem; margin-top:40px;">Evolutionsgeschichte leer.</div>`;
    }
    modalStammbaumTreeContainer.innerHTML = treeHTML;

  }).catch(err => {
    console.error("Failed to compile Stammbaum tree:", err);
  });
}

/**
 * High-performance, reactive species selector inside the Stammbaum & Archiv modal.
 * Pulls the detailed genome and compiles its Watson-Crick DNA grid and CTRNN directed graph live inside the drawer!
 */
function selectSpeciesForInspection(speciesId: string, itemEl: HTMLElement): void {
  getSpeciesById(speciesId).then(record => {
    if (!record) return;

    // Slide out the genetic drawer
    modalSpeciesDetails.style.display = "flex";
    drawerLatinName.innerText = record.name;
    drawerSpeciesId.innerText = `ID: ${record.id}`;

    // Parse genome phenotype
    const pheno = parseGenome(record.genome, record.antisense);

    // Compile and render its CTRNN brain graph inside our drawer!
    renderSynapseWeb(pheno, drawerBrainVisualizer);

    // Compile and render its Watson-Crick genome grid inside our drawer!
    updateGenomeGrid(pheno, -1, drawerGenomeGrid);

    // Visual highlights: remove .roster-item-active and .stammbaum-node-active from other cards
    document.querySelectorAll(".roster-item-active").forEach(el => el.classList.remove("roster-item-active"));
    document.querySelectorAll(".stammbaum-node-active").forEach(el => el.classList.remove("stammbaum-node-active"));

    // Highlight this selected card
    itemEl.classList.add(itemEl.classList.contains("roster-item") ? "roster-item-active" : "stammbaum-node-active");

    // Try to select a live, swimming agent on the canvas of this species (if any are alive!)
    const liveAgent = creatures.find(c => c.speciesId === speciesId);
    if (liveAgent) {
      selectedAgentId = liveAgent.id;
      updateGenomeGrid(liveAgent.phenotype);
      renderSynapseWeb(liveAgent.phenotype);
    }
    
    logToConsole(`[Archiv] Spezies '${record.name}' ausgewählt. Genom- und Gehirn-Datenblatt geladen.`);
  }).catch(err => {
    console.error("Failed to select species from archive:", err);
  });
}

/* OBSOLETE INSPECT CODE BLOCK WIPE START
    <div class="vital-bar-container">
      <span class="vital-bar-label">⚡ Magenfüllung:</span>
      <div class="vital-bar-bg">
        <div class="vital-bar-fill" style="width: ${energyPercent}%; background-color: #10b981; box-shadow: 0 0 10px #10b981;"></div>
      </div>
      <span style="font-weight:bold; color:#10b981; min-width:35px; text-align:right;">${energyPercent}%</span>
    </div>

    <div class="vital-bar-container" style="margin-top:8px;">
      <span class="vital-bar-label">⌛ Seneszenz (Alter):</span>
      <div class="vital-bar-bg">
        <div class="vital-bar-fill" style="width: ${agePercent}%; background-color: #38bdf8; box-shadow: 0 0 10px #38bdf8;"></div>
      </div>
      <span style="font-weight:bold; color:#38bdf8; min-width:35px; text-align:right;">${agePercent}%</span>
    </div>

    <div class="vital-bar-container" style="margin-top:8px;">
      <span class="vital-bar-label">🔥 Adrenalinspiegel:</span>
      <div class="vital-bar-bg">
        <div class="vital-bar-fill" style="width: ${Math.round(((agent.adrenaline - 1.0)/0.8) * 100)}%; background-color: #f43f5e; box-shadow: 0 0 10px #f43f5e;"></div>
      </div>
      <span style="font-weight:bold; color:#f43f5e; min-width:35px; text-align:right;">${Math.round(agent.adrenaline * 100)}%</span>
    </div>

    <div class="trait-list" style="margin-top: 14px; display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
      <div class="trait-item">
        <span class="trait-label">Stetige Starrheit:</span>
        <span class="trait-val" style="color: #00f2fe; font-weight:bold;">${Math.round(agent.phenotype.stiffness * 100)}%</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Atemtakt (pulseSpeed):</span>
        <span class="trait-val">${Math.round(agent.phenotype.pulseSpeed * 10000) / 10} Hz</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Chassis-Biomasse:</span>
        <span class="trait-val">${Math.round(agent.phenotype.spinalHarmonics.baseLength * agent.phenotype.spinalHarmonics.meanRadius)} μg</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Magen-Maximalvolumen:</span>
        <span class="trait-val" style="color:#fbbf24; font-weight:bold;">${Math.round(agent.phenotype.stomachCapacity)} nJ</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Metabolischer Grundverbrauch (BMR):</span>
        <span class="trait-val" style="color:#f43f5e; font-weight:bold;">${agent.phenotype.basalMetabolicRate} fJ/f</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Thermische Resistenzzone:</span>
        <span class="trait-val" style="color:#34d399;">${agent.phenotype.thermalToleranceMin}°C bis ${agent.phenotype.thermalToleranceMax}°C</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Evolvierte CTRNN Neuronen:</span>
        <span class="trait-val" style="color:#c084fc; font-weight:bold;">${totalNeurons} Knoten</span>
      </div>
      <div class="trait-item">
        <span class="trait-label">Evolvierte Synapsen (NEAT):</span>
        <span class="trait-val" style="color:#a5f3fc; font-weight:bold;">${totalSynapses} Verbindungen</span>
      </div>
      <div class="trait-item" style="grid-column: span 2;">
        <span class="trait-label">DNA-Reparaturtreue (Fidelity):</span>
        <span class="trait-val" style="color:#10b981; font-weight:bold;">${Math.round(agent.phenotype.repairFidelity * 100)}% (Slippage: Ins ${Math.round(agent.phenotype.insertionRate*100)}% / Del ${Math.round(agent.phenotype.deletionRate*100)}%)</span>
      </div>
      <div class="trait-item" style="grid-column: span 2;">
        <span class="trait-label">Ahnenlinie (Stammbaum-Mutter ID):</span>
        <span class="trait-val" style="color:#d946ef; font-family:monospace; font-size:0.6rem; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; max-width:240px;" title="${agent.generation > 1 ? agent.speciesId : "Gründer-Urzelle"}">
          ${agent.generation > 1 ? agent.speciesId : "Gründer-Urzelle (Generation 1)"}
        </span>
      </div>
    </div>
  `;

  // Populate Organelles Scrollbox inside the Modal
  modalOrganellesContainer.innerHTML = "";
  if (agent.phenotype.organelles.length === 0) {
    modalOrganellesContainer.innerHTML = `
      <div style="color: var(--text-muted); font-size: 0.8rem; text-align: center; margin-top: 40px;">
        Keine Rezeptor-Patches exprimiert.<br>Membran ist vollkommen glatt.
      </div>
    `;
    return;
  }

  agent.phenotype.organelles.forEach((node, idx) => {
    const identity = classifySensoryPatch(node);
    
    const spectralPercent = Math.round(node.spectralAffinity * 100);
    const specPercent = Math.round((1.0 - node.bandwidth) * 100);

    let spectralTypeName = "Mechanisch";
    if (node.spectralAffinity >= 0.8) spectralTypeName = "Photonen/Licht";
    else if (node.spectralAffinity >= 0.25 && node.spectralAffinity <= 0.65) spectralTypeName = "Moleküle/Chemo";
    else if (node.spectralAffinity > 0.65 && node.spectralAffinity < 0.8) spectralTypeName = "Infrarot/Wärme";

    const card = document.createElement("div");
    card.className = "organelle-card";
    card.innerHTML = `
      <div class="organelle-card-meta">
        <div class="organelle-card-title">
          <span class="organelle-dot" style="background-color: ${identity.color}; box-shadow: 0 0 6px ${identity.color};"></span>
          <span style="font-weight:bold; color:#fff;">Gen #${idx + 1}: ${identity.name}</span>
        </div>
        <div class="organelle-card-loci">Loci ${node.geneStartIndex}-${node.geneEndIndex}</div>
        <div style="font-size:0.68rem; color:var(--text-muted); font-style:italic; margin-top:2px;">${identity.desc}</div>
      </div>
      <div>
        <div class="organelle-card-val" style="color: ${identity.color}; font-weight:700;">λ: ${spectralPercent}% (${spectralTypeName})</div>
        <div class="organelle-card-loci">Spezial-Schärfe: ${specPercent}%</div>
        <div class="organelle-card-loci">Pos: ${Math.round(node.spinalPos * 100)}% | Winkel: ${Math.round(node.angle)}°</div>
      </div>
    `;
    modalOrganellesContainer.appendChild(card);
  });
}
OBSOLETE INSPECT CODE BLOCK WIPE END */

/**
 * Resizes simulation canvas to fill the complete screen viewport dynamically
 * scaled by the device pixel ratio to deliver high-resolution Retina/4K sharpness!
 */
function resizeCanvas(): void {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = window.innerWidth * dpr;
  canvas.height = window.innerHeight * dpr;
  
  const ctx = canvas.getContext("2d");
  if (ctx) {
    ctx.scale(dpr, dpr);
  }
}

/**
 * Main update routine. Computes phenotype from genome, updates DOM components.
 * Epigenetic Memory Transfer: Passes previous methylations directly to keep learned synapses!
 */
function updateSimulation(highlightLocus: number = -1): void {
  sandboxPhenotype = parseGenome(sandboxGenome, sandboxAntisense, sandboxPhenotype ? sandboxPhenotype.methylations : undefined);

  // Reset recurrent neural memory buffers upon simulation updates
  sandboxNeuronStates = [];
  sandboxNeuronActivations = [];

  // Update genome string display
  genomeStringEl.innerText = sandboxGenome;
  
  const dpr = window.devicePixelRatio || 1;
  spx = (canvas.width / dpr) / 2;
  spy = (canvas.height / dpr) / 2;
  svx = 0;
  svy = 0;
  somegaRot = 0;
  
  // Refresh interactive grid and trait panels once
  updateGenomeGrid(sandboxPhenotype, highlightLocus);
  updateTraitsDashboard(sandboxPhenotype, 0, 0, 0, 1, 0, 0);
  renderSynapseWeb(sandboxPhenotype);

  // Print Cascading Epigenetic Embryology Logs
  sandboxPhenotype.epigeneticLogs.forEach(logLine => {
    logToConsole(logLine);
  });
}

/**
 * Evaluates the biological impact of a mutation in the codon scanning genome.
 */
function explainMutation(index: number, oldVal: string, newVal: string, oldPheno: CreaturePhenotype, newPheno: CreaturePhenotype): string {
  if (index < 16) {
    if (index === 0) {
      const oldSym = oldPheno.symmetry === "vertical" ? "Bilateral" : "Quad";
      const newSym = newPheno.symmetry === "vertical" ? "Bilateral" : "Quad";
      if (oldSym !== newSym) {
        return `Symmetrie mutiert von ${oldSym} zu <strong style="color: #ff4a85">${newSym}</strong>. Die komplette Struktur der Lebensform hat sich umgestellt!`;
      }
    }
    if (index === 8) {
      return `Spinal-Größe (Locus 8) mutiert von '${oldVal}' zu '${newVal}'. Das Notochord hat sich stetig verlängert/verkürzt auf <strong style="color: #ff4a85">${Math.round(newPheno.spinalHarmonics.baseLength)}px</strong>, wodurch nun <strong style="color: #ff4a85">${newPheno.emergentChambersCount} Segmente</strong> emergieren!`;
    }
    if (index === 9) {
      return `Insertions-Rate (Locus 9) mutiert. Das Genom-Verdoppelungsrisiko beträgt nun <strong style="color: #d946ef">${Math.round(newPheno.insertionRate * 100)}%</strong>.`;
    }
    if (index === 10) {
      return `Deletions-Rate (Locus 10) mutiert. Das Genom-Verkürzungsrisiko beträgt nun <strong style="color: #d946ef">${Math.round(newPheno.deletionRate * 100)}%</strong>.`;
    }
    if (index === 11) {
      return `Reparatur-Fidelity (Locus 11) mutiert von '${oldVal}' zu '${newVal}'. Die Replikations-Reparaturquote des Polymerase-Systems beträgt nun <strong style="color: #10b981">${Math.round(newPheno.repairFidelity * 100)}%</strong>.`;
    }
    if (index === 12) {
      return `Wirbelsäulen-Stiffness (Locus 12) mutiert von '${oldVal}' zu '${newVal}'. Die Starrheit/Elastizität beträgt nun <strong style="color: #00f2fe">${Math.round(newPheno.stiffness * 100)}%</strong>. Die Kraftübertragung verschiebt sich.`;
    }
    if (index === 13) {
      return `Atemfrequenz-Locus (Locus 13) mutiert. Kontraktionsgeschwindigkeit hat sich auf <strong style="color: #00f2fe">${Math.round(newPheno.pulseSpeed * 10000) / 10} Hz</strong> verschoben!`;
    }
    if (index === 14) {
      return `Wellenphase (Locus 14) mutiert. Die Wellenverzögerung zwischen den Segmenten (peristaltischer Fluss) beträgt nun <strong style="color: #00f2fe">${Math.round(newPheno.wavePhase * 100) / 100} rad</strong>.`;
    }
    if (index === 15) {
      return `Strömungswiggle (Locus 15) mutiert. Tastfühler schwankan nun um <strong style="color: #00f2fe">${Math.round(newPheno.wiggleAmplitude * 100)}%</strong> intensiver in der virtuellen Strömung.`;
    }

    return `Locus ${index} (Chassis-Modifikator: '${oldVal}' -> '${newVal}'). Anpassung der Fourier-Körperformen.`;
  }

  // Brain synapse connection shift logs
  if (index >= 16 && index <= 31) {
    return `Locus ${index} mutiert inside Loci 16–31. Die genetischen Deep Brain-Topology oder synaptischen Basisgewichtungen verändern sich permanent!`;
  }

  const oldActiveCount = oldPheno.organelles.length;
  const newActiveCount = newPheno.organelles.length;

  if (newActiveCount > oldActiveCount) {
    return `<strong style="color: #ff4a85">[GEN-EMERGENZ]</strong> Locus ${index} mutiert ('${oldVal}' -> '${newVal}'). Es wurde ein neuer <strong style="color: #ff4a85">Start-Codon (Promoter)</strong> synthetisiert! Ein neues Organ ist gesprossen!`;
  }
  if (newActiveCount < oldActiveCount) {
    return `<strong style="color: #ff007f">[GEN-STILLLEGUNG]</strong> Locus ${index} mutiert ('${oldVal}' -> '${newVal}'). Ein <strong style="color: #ff007f">Start/Stop-Promoter</strong> wurde zerstört. Das Organ wurde stillgelegt und liegt nun als stumme DNA (Intron) vor.`;
  }

  let activeGenNum = -1;
  let affectedOrgan: SensoryPatch | null = null;
  for (let sIdx = 0; sIdx < newPheno.activeGeneSpans.length; sIdx++) {
    const span = newPheno.activeGeneSpans[sIdx];
    if (index >= span.start && index < span.end) {
      activeGenNum = sIdx + 1;
      affectedOrgan = newPheno.organelles[sIdx];
      break;
    }
  }

  if (activeGenNum !== -1 && affectedOrgan) {
    const identity = classifySensoryPatch(affectedOrgan);
    const span = newPheno.activeGeneSpans[activeGenNum - 1];
    const offset = index - (span.start + 2);

    if (offset === 0) {
      return `Locus ${index} mutiert inside Gen #${activeGenNum}. Die <strong style="color: var(--color-active)">spektrale Affinität (λ: ${Math.round(affectedOrgan.spectralAffinity * 100)}%)</strong> hat sich verschoben! Der Sensor empfängt jetzt andere Frequenzen.`;
    }
    if (offset === 1) {
      return `Locus ${index} mutiert inside Gen #${activeGenNum}. Die <strong style="color: var(--color-active)">Sensorschärfe / Spezialisierung (${Math.round((1.0 - affectedOrgan.bandwidth) * 100)}%)</strong> wurde modifiziert.`;
    }
    if (offset === 2) {
      return `Locus ${index} mutiert inside Gen #${activeGenNum}. Die <strong style="color: var(--color-active)">morphologische Form (Haar-Expression: ${Math.round(affectedOrgan.expressionStyle * 100)}%)</strong> hat sich verformt!`;
    }
    if (offset === 3) {
      return `Locus ${index} mutiert inside Gen #${activeGenNum} (${identity.name}). Die <strong style="color: var(--color-active)">physische Größe (Skalierung: ${Math.round(affectedOrgan.scale * 100)}%)</strong> wurde verschoben.`;
    }
    if (offset === 4) {
      return `Locus ${index} mutiert inside Gen #${activeGenNum} (${identity.name}). Die stetige <strong style="color: var(--color-active)">Spinal-Position (${Math.round(affectedOrgan.spinalPos * 100)}%)</strong> wurde neu ausgewählt! Das Organ wandert entlang der Körperachse!`;
    }
    if (offset === 5) {
      return `Locus ${index} mutiert inside Gen #${activeGenNum} (${identity.name}). Der <strong style="color: var(--color-active)">Ankerwinkel (${Math.round(affectedOrgan.angle)}°)</strong> am Körperrand wurde verschoben.`;
    }
    if (offset === 6) {
      return `Locus ${index} mutiert inside Gen #${activeGenNum} (${identity.name}). Die <strong style="color: var(--color-active)">Pigmentierungs-Farbe</strong> hat sich verändert.`;
    }

    return `Locus ${index} mutiert inside Gen #${activeGenNum} (${identity.name}). Anpassung stiller Codon-Eigenschaften.`;
  }

  if (!newPheno.chromatinState[index]) {
    return `Locus ${index} mutiert von '${oldVal}' zu '${newVal}'. Dies betrifft ein stummes Segment im <span style="color: var(--text-muted)">Heterochromatin (CLOSED)</span>. Die Mutation verbleibt unbemerkt im stillen Archiv.`;
  }

  return `Locus ${index} mutiert von '${oldVal}' zu '${newVal}'. Die Chromatinstruktur ist offen, aber es wurde kein aktives structural Gen moduliert.`;
}

// Stats UI Refresh Interval
let lastUiUpdate = 0;

// ============================================================================
// 60FPS Continuous Animation & 2D Hydrodynamic Rigid-Body Physics Loop
// ============================================================================
function animate(timestamp: number) {
  const dt = lastTime === 0 ? 0 : Math.min(100, timestamp - lastTime); 
  lastTime = timestamp;

  const dpr = window.devicePixelRatio || 1;
  const logicalWidth = canvas.width / dpr;
  const logicalHeight = canvas.height / dpr;

  // CLEAR CANVAS ONCE per frame (Supports N creatures rendering together!)
  const ctx = canvas.getContext("2d");
  if (ctx) {
    ctx.clearRect(0, 0, logicalWidth, logicalHeight);
  }

  const frameScale = dt / 16.66;

  if (renderer) {
    // Render Background Water Bubbles exactly once per frame (Unified environment backdrop!)
    if (ctx) {
      ctx.save();
      ctx.fillStyle = "rgba(255, 255, 255, 0.08)";
      for (let i = 0; i < 28; i++) {
        const bxInit = ((i * 47) % 100) / 100 * logicalWidth;
        const yInit = ((i * 93) % 100) / 100 * logicalHeight;
        
        const parallaxSpeed = 0.25 + ((i * 17) % 10) / 10 * 0.45;
        const size = 1.5 + ((i * 23) % 4);
        
        // Bubbles slide relative to mouse tracking vectors
        const bx = (bxInit + tx * parallaxSpeed + logicalWidth) % logicalWidth;
        const by = (yInit + ty * parallaxSpeed + logicalHeight) % logicalHeight;

        ctx.beginPath();
        ctx.arc(bx, by, size, 0, Math.PI * 2);
        ctx.fill();

        if (size > 4) {
          ctx.strokeStyle = "rgba(255, 255, 255, 0.03)";
          ctx.lineWidth = 1;
          ctx.stroke();
        }
      }
      ctx.restore();
    }
    if (currentMode === "sandbox") {
      // ========================================================================
      // A. SANDBOX LABOR MODE (Single Specimen manual playground)
      // ========================================================================
      
      // 1. Food Target orbitingFallback
      if (!isMouseInsideCanvas) {
        const centerX = logicalWidth / 2;
        const centerY = logicalHeight / 2;
        tx = centerX + 180 * Math.cos(timestamp * 0.0009);
        ty = centerY + 180 * Math.sin(timestamp * 0.0009);
      }

      let torque = 0;
      let baseThrustMag = sandboxPhenotype.stiffness * (sandboxPhenotype.pulseSpeed * 1000 * sandboxPhenotype.pulseSpeed * 1000) * 6.0;

      // Continuous Biomechanical Swimming Efficiency (Säule 2: Aal- vs. Blob-Physik!)
      // Derived stethically and continuously from Notochord Aspect Ratio (L/R), Wellenphase and stiffness!
      const sL = sandboxPhenotype.spinalHarmonics.baseLength;
      const sR = sandboxPhenotype.spinalHarmonics.meanRadius;
      const sPhase = sandboxPhenotype.wavePhase;
      const sStiffness = sandboxPhenotype.stiffness;
      const etaSwim = Math.max(0.1, Math.min(3.2, (sL / (sR * 3.5)) * Math.max(0.01, Math.sin(sPhase)) * sStiffness));
      baseThrustMag *= etaSwim;

      const activeLimbsCount = sandboxPhenotype.organelles.filter(node => node.expressionStyle >= 0.72).length;
      baseThrustMag *= (1.0 + activeLimbsCount * 0.12);
      baseThrustMag *= (1.0 + sandboxPhenotype.spinalHarmonics.parapodiaAmp * 1.0);

      let fx = 0;
      let fy = 0;

      const isAiControlled = aiToggle ? aiToggle.checked : false;
      const aiToggleText = document.querySelector(".ai-toggle-text") as HTMLSpanElement;
      if (aiToggleText) {
        aiToggleText.innerText = isAiControlled ? "Aktiv (KI-Gehirn)" : "Inaktiv (WASD)";
      }

      let neuralInputs = [0, 0, 0, 0, 0];
      let neuralOutputs = [0, 0, 0, 0];
      const brain = sandboxPhenotype.brain;

      if (isAiControlled) {
        // Feedforward AI brain decision (Direct unified spectral scanning)
        const clockVal = 0.5 + 0.5 * Math.sin(timestamp * 0.0012);
        neuralInputs = computeEmbodiedSensoryInputs(sandboxPhenotype, spx, spy, sheadingAngle, svx, svy, somegaRot, clockVal, -1);

        // Execute Native Recurrent CTRNN Brain (Euler temporal memory integration)
        const brainRes = executeBrain(brain, neuralInputs, sandboxNeuronStates, sandboxNeuronActivations);
        neuralOutputs = brainRes.outputs;

        const outThrust = neuralOutputs[0];
        const outLeft = neuralOutputs[1];
        const outRight = neuralOutputs[2];

        if (outThrust > 0.0) {
          fx += outThrust * baseThrustMag * Math.cos(sheadingAngle);
          fy += outThrust * baseThrustMag * Math.sin(sheadingAngle);
        }
        torque = (outRight - outLeft) * sandboxPhenotype.stiffness * 5.8;

        // Hebbian Recurrent Graph Neuroplasticity Learning (Iterates over arbitrary CTRNN synapses!)
        const learningRate = 0.00015 * (1.0 - sandboxPhenotype.stiffness * 0.85);
        const forgettingDecay = 0.00002;

        brain.synapses.forEach((syn: any) => {
          const preVal = sandboxNeuronActivations[syn.fromNode];
          const postVal = Math.max(0.0, sandboxNeuronActivations[syn.toNode]); // learning from positive excitation
          
          let weight = syn.weight;
          weight += learningRate * (preVal * postVal) - forgettingDecay * weight;
          weight = Math.max(-2.5, Math.min(2.5, weight));
          syn.weight = weight;
        });
        updateLiveNeuralActivity(brainRes.allLayerActivations, brain);

      } else {
        // Manual override WASD controls: construct inputs dynamically of size K + 1
        const clockVal = 0.5 + 0.5 * Math.sin(timestamp * 0.0012);
        const K = sandboxPhenotype.organelles.length;
        neuralInputs = Array(K + 1).fill(0.0);
        neuralInputs[K] = clockVal; // clock at last node
        
        // Feed WASD keystrokes into the first available receptors
        if (K > 0) neuralInputs[0] = keys.w ? 1.0 : (keys.s ? -0.5 : 0.0);
        if (K > 1) neuralInputs[1] = keys.a ? 1.0 : 0.0;
        if (K > 2) neuralInputs[2] = keys.d ? 1.0 : 0.0;

        // Execute Native Recurrent CTRNN Brain (Euler temporal memory integration)
        const brainRes = executeBrain(brain, neuralInputs, sandboxNeuronStates, sandboxNeuronActivations);
        neuralOutputs = brainRes.outputs;

        const outThrust = neuralOutputs[0];
        const outLeft = neuralOutputs[1];
        const outRight = neuralOutputs[2];

        // Apply physical movements driven by real brain outputs!
        if (outThrust > 0.0) {
          fx += outThrust * baseThrustMag * Math.cos(sheadingAngle);
          fy += outThrust * baseThrustMag * Math.sin(sheadingAngle);
        } else if (outThrust < 0.0) {
          fx += outThrust * baseThrustMag * 0.5 * Math.cos(sheadingAngle);
          fy += outThrust * baseThrustMag * 0.5 * Math.sin(sheadingAngle);
        }
        torque = (outRight - outLeft) * sandboxPhenotype.stiffness * 5.8;

        updateLiveNeuralActivity(brainRes.allLayerActivations, brain);
      }

      // Physics Integration
      const L_noto = sandboxPhenotype.spinalHarmonics.baseLength;
      const meanRadius = sandboxPhenotype.spinalHarmonics.meanRadius;
      const mass = Math.pow(meanRadius, 1.5) * (L_noto / 25);
      const momentOfInertia = mass * (1.0 + (L_noto * L_noto) * 0.00015);
      const rotDragCoeff = 0.45 * mass;
      const rotDragTorque = -rotDragCoeff * somegaRot;
      const alphaRot = (torque + rotDragTorque) / momentOfInertia;

      somegaRot += alphaRot * frameScale;
      sheadingAngle += somegaRot * frameScale;
      sheadingAngle = Math.atan2(Math.sin(sheadingAngle), Math.cos(sheadingAngle));

      const vForward = svx * Math.cos(sheadingAngle) + svy * Math.sin(sheadingAngle);
      const vLateral = -svx * Math.sin(sheadingAngle) + svy * Math.cos(sheadingAngle);

      const receptorBallast = sandboxPhenotype.organelles.length * 0.18;
      const dragForward = (meanRadius * 0.015 + receptorBallast) * (1.0 - sandboxPhenotype.stiffness * 0.3);
      const dragLateral = L_noto * 0.045 + receptorBallast;

      const dragForceForward = -dragForward * vForward;
      const dragForceLateral = -dragLateral * vLateral;

      const fxDrag = dragForceForward * Math.cos(sheadingAngle) - dragForceLateral * Math.sin(sheadingAngle);
      const fyDrag = dragForceForward * Math.sin(sheadingAngle) + dragForceLateral * Math.cos(sheadingAngle);

      const ax = (fx + fxDrag) / mass;
      const ay = (fy + fyDrag) / mass;

      svx += ax * frameScale;
      svy += ay * frameScale;

      // Apply fluid friction damping (Viscous decay) to prevent runaway speeds
      svx *= 0.94;
      svy *= 0.94;
      somegaRot *= 0.88;

      spx += svx * frameScale;
      spy += svy * frameScale;

      // Soft damped wall bouncing (No toroidal wrapping!)
      const margin = (meanRadius * 1.5) * 0.5 + 10; // logical margin based on 50% scale-down
      if (spx < margin) {
        spx = margin;
        svx = -svx * 0.45;
        somegaRot = -somegaRot * 0.5;
      } else if (spx > logicalWidth - margin) {
        spx = logicalWidth - margin;
        svx = -svx * 0.45;
        somegaRot = -somegaRot * 0.5;
      }

      if (spy < margin) {
        spy = margin;
        svy = -svy * 0.45;
        somegaRot = -somegaRot * 0.5;
      } else if (spy > logicalHeight - margin) {
        spy = logicalHeight - margin;
        svy = -svy * 0.45;
        somegaRot = -somegaRot * 0.5;
      }

      // Render single specimen
      renderer.render(sandboxPhenotype, timestamp, spx, spy, sheadingAngle, somegaRot);

      if (timestamp - lastUiUpdate > 150) {
        const netThrust = Math.sqrt(fx * fx + fy * fy);
        updateTraitsDashboard(sandboxPhenotype, svx, svy, netThrust, mass, dragForward, dragLateral, sheadingAngle);
        lastUiUpdate = timestamp;
      }

    } else {
      // ========================================================================
      // B. MULTI-AGENT EVOLUTION Mode (Autonomous survival ocean)
      // ========================================================================
      
      // 1. Draw and drift food spores
      if (ctx) {
        ctx.save();
        for (const pellet of foodPellets) {
          pellet.x += pellet.vx;
          pellet.y += pellet.vy;

          // Soft elastic bounce off glass borders so they stay in the tank!
          const pMargin = 6;
          if (pellet.x < pMargin) {
            pellet.x = pMargin;
            pellet.vx = -pellet.vx;
          } else if (pellet.x > logicalWidth - pMargin) {
            pellet.x = logicalWidth - pMargin;
            pellet.vx = -pellet.vx;
          }

          if (pellet.y < pMargin) {
            pellet.y = pMargin;
            pellet.vy = -pellet.vy;
          } else if (pellet.y > logicalHeight - pMargin) {
            pellet.y = logicalHeight - pMargin;
            pellet.vy = -pellet.vy;
          }

          ctx.beginPath();
          ctx.arc(pellet.x, pellet.y, 4, 0, Math.PI * 2);
          ctx.fillStyle = "rgba(16, 185, 129, 0.85)"; // glowing spores
          ctx.shadowColor = "#10b981";
          ctx.shadowBlur = 8;
          ctx.fill();
        }
        ctx.restore();
      }

      // 2. Loop update all living creature agents
      const deadAgentIds: number[] = [];
      const newbornAgents: CreatureAgent[] = [];
      
      // Create a fast lookup set of currently alive genomes (Species) to calculate Extinctions!
      const currentAliveSpeciesThisFrame = new Set<string>();

      creatures.forEach(agent => {
        agent.age += 1;
        currentAliveSpeciesThisFrame.add(agent.speciesId);

        if (agent.adrenaline === undefined) agent.adrenaline = 1.0;

        // 1. Farb-Metabolismus (Photosynthese für grüne Beutetiere in oberer Lichtzone)
        const hue = agent.phenotype.primaryColor.h;
        const isGreenPrey = !agent.phenotype.isPredator && (hue >= 75 && hue <= 175);
        const inLightZone = agent.py < logicalHeight * 0.35;
        if (isGreenPrey && inLightZone) {
          agent.energy = Math.min(agent.phenotype.stomachCapacity, agent.energy + 0.15); // absorbiert +0.15 Sonnenenergie
        }
        
        // 2. Spatial Temperature Stratification and Thermal Limits (Säule 2: Abiotische Gradienten!)
        // Surface is hot (38°C), bottom is freezing cold (12°C)
        const localTemp = 38.0 - (agent.py / logicalHeight) * 26.0;
        const tempMin = agent.phenotype.thermalToleranceMin;
        const tempMax = agent.phenotype.thermalToleranceMax;
        const thermalStress = localTemp < tempMin ? (tempMin - localTemp) : (localTemp > tempMax ? (localTemp - tempMax) : 0.0);
        
        if (thermalStress > 0.1) {
          // Thermal stress causes significant ongoing energy drain scaled by stress delta
          agent.energy -= thermalStress * 0.0012 * frameScale;
        }

        // 3. Adrenalin-Surcharge metabolic tax (Sprints verdoppeln Energieverbrauch!)
        const metabolicSurcharge = 1.0 + (agent.adrenaline - 1.0) * 1.5;
        agent.energy -= agent.phenotype.basalMetabolicRate * 0.005 * metabolicSurcharge;

        // Check death conditions: starvation or senescence (Tod / Kadaver-Zersetzung)
        if (agent.energy <= 0 || agent.age >= 2700) {
          deadAgentIds.push(agent.id);
          
          // 3. Organischer Kadaver-Nährstoffkreislauf:
          // Körpergröße (Masse) bestimmt die Zahl der freigesetzten Nahrungsporen (1 bis 5)
          const L_dead = agent.phenotype.spinalHarmonics.baseLength;
          const r_dead = agent.phenotype.spinalHarmonics.meanRadius;
          const biomass = L_dead * r_dead;
          const numPellets = Math.max(1, Math.min(5, Math.floor(biomass / 1200)));

          // Wir verlagern bestehende Nahrungsporen an den Todesort (KISS-konstant ohne Memory-Leaks!)
          for (let p = 0; p < numPellets; p++) {
            const pIdx = Math.floor(Math.random() * foodPellets.length);
            if (foodPellets[pIdx]) {
              foodPellets[pIdx].x = agent.px + Math.random() * 32 - 16;
              foodPellets[pIdx].y = agent.py + Math.random() * 32 - 16;
              // Setze langsame Ausbreitungs-Drift
              foodPellets[pIdx].vx = (Math.random() * 0.4 - 0.2);
              foodPellets[pIdx].vy = (Math.random() * 0.4 - 0.2);
            }
          }

          logToConsole(`Kadaver-Zersetzung! Spezies #${agent.id} ist ${agent.energy <= 0 ? "verhungert" : "an Altersschwäche gestorben"}. ${numPellets} Nährstoff-Sporen freigesetzt.`, "mutation");
          return;
        }

        // ==========================================
        // Continuous Brain Decision (Sensory to Motor)
        // ==========================================
        let fx = 0;
        let fy = 0;
        let torque = 0;

        const isPred = agent.phenotype.isPredator;
        const meanRadius = agent.phenotype.spinalHarmonics.meanRadius;
        const baseLength = agent.phenotype.spinalHarmonics.baseLength;
        const stiffness = agent.phenotype.stiffness;
        const pulse = agent.phenotype.pulseSpeed;

        let thrustMag = stiffness * (pulse * 1000 * pulse * 1000) * 6.0;

        // Continuous Biomechanical Swimming Efficiency (Säule 2: Aal- vs. Blob-Physik!)
        // Derived stethically and continuously from Notochord Aspect Ratio (L/R), Wellenphase and stiffness!
        const wavePhase = agent.phenotype.wavePhase;
        const etaSwim = Math.max(0.1, Math.min(3.2, (baseLength / (meanRadius * 3.5)) * Math.max(0.01, Math.sin(wavePhase)) * stiffness));
        thrustMag *= etaSwim;

        const limbsCount = agent.phenotype.organelles.filter(n => n.expressionStyle >= 0.72).length;
        thrustMag *= (1.0 + limbsCount * 0.12);
        thrustMag *= (1.0 + agent.phenotype.spinalHarmonics.parapodiaAmp * 1.0);

        // Feed dynamic inputs to Compiled Deep Perceptron scaled by physical expressed organelles (Embodied Cognition!)
        const clockVal = 0.5 + 0.5 * Math.sin(timestamp * 0.0012 + agent.id);
        const inputs = computeEmbodiedSensoryInputs(agent.phenotype, agent.px, agent.py, agent.headingAngle, agent.vx, agent.vy, agent.omegaRot, clockVal, agent.id);

        // Execute Native Recurrent CTRNN Brain (Euler temporal memory integration)
        const brainRes = executeBrain(agent.phenotype.brain, inputs, agent.neuronStates, agent.neuronActivations);
        const outputs = brainRes.outputs;

        const outThrust = outputs[0];
        const outLeft = outputs[1];
        const outRight = outputs[2];

        // Sum up perceived threat levels from all danger-receptors (Säule 4: Embodied Physiology!)
        let threatPerception = 0.0;
        agent.phenotype.organelles.forEach((patch, idx) => {
          const sensesDanger = (patch.spectralAffinity < 0.25) || (patch.spectralAffinity > 0.65 && patch.spectralAffinity < 0.8);
          if (sensesDanger) {
            threatPerception = Math.max(threatPerception, inputs[idx]);
          }
        });

        // Update endocrine Adrenaline based on actual threat perception
        if (threatPerception > 0.1) {
          agent.adrenaline = Math.min(1.8, agent.adrenaline + 0.06 * threatPerception);
        } else {
          agent.adrenaline = Math.max(1.0, agent.adrenaline - 0.015);
        }

        if (outThrust > 0.0) {
          // Predators get an aggressive bite speed boost!
          const predatorSavageMultiplier = isPred ? 1.45 : 1.0;
          // Scale muscular force directly by adrenaline level (the panic sprint!)
          fx += outThrust * thrustMag * predatorSavageMultiplier * agent.adrenaline * Math.cos(agent.headingAngle);
          fy += outThrust * thrustMag * predatorSavageMultiplier * agent.adrenaline * Math.sin(agent.headingAngle);
        }
        torque = (outRight - outLeft) * stiffness * 5.8 * agent.adrenaline; // boost torque during sprints

        // Hebbian Recurrent Graph Neuroplasticity Learning (Iterates over arbitrary CTRNN synapses!)
        const learningRate = 0.00015 * (1.0 - stiffness * 0.85);
        const forgettingDecay = 0.0000032; // Set to exactly 0.0000032 for a perfect 1-hour synaptic half-life at 60fps!
        const b = agent.phenotype.brain;

        b.synapses.forEach((syn: any) => {
          const preVal = agent.neuronActivations[syn.fromNode];
          const postVal = Math.max(0.0, agent.neuronActivations[syn.toNode]); // learning from positive excitation
          
          let weight = syn.weight;
          weight += learningRate * (preVal * postVal) - forgettingDecay * weight;
          weight = Math.max(-2.5, Math.min(2.5, weight));
          syn.weight = weight;
        });
        
        if (agent.id === selectedAgentId) {
          updateLiveNeuralActivity(brainRes.allLayerActivations, b);
        }

        // Biomechanical Physics integration
        const mass = Math.pow(meanRadius, 1.5) * (baseLength / 25);
        const momentOfInertia = mass * (1.0 + (baseLength * baseLength) * 0.00015);
        const rotDragCoeff = 0.45 * mass;
        const rotDragTorque = -rotDragCoeff * agent.omegaRot;
        const alphaRot = (torque + rotDragTorque) / momentOfInertia;

        agent.omegaRot += alphaRot * frameScale;
        agent.headingAngle += agent.omegaRot * frameScale;
        agent.headingAngle = Math.atan2(Math.sin(agent.headingAngle), Math.cos(agent.headingAngle));

        const vForward = agent.vx * Math.cos(agent.headingAngle) + agent.vy * Math.sin(agent.headingAngle);
        const vLateral = -agent.vx * Math.sin(agent.headingAngle) + agent.vy * Math.cos(agent.headingAngle);

        const receptorBallast = agent.phenotype.organelles.length * 0.18;
        const dragForward = (meanRadius * 0.015 + receptorBallast) * (1.0 - stiffness * 0.3);
        const dragLateral = baseLength * 0.045 + receptorBallast;

        const dragForceForward = -dragForward * vForward;
        const dragForceLateral = -dragLateral * vLateral;

        const fxDrag = dragForceForward * Math.cos(agent.headingAngle) - dragForceLateral * Math.sin(agent.headingAngle);
        const fyDrag = dragForceForward * Math.sin(agent.headingAngle) + dragForceLateral * Math.cos(agent.headingAngle);

        const ax = (fx + fxDrag) / mass;
        const ay = (fy + fyDrag) / mass;

        agent.vx += ax * frameScale;
        agent.vy += ay * frameScale;

        // Apply fluid friction damping (Viscous decay) to prevent runaway speeds
        agent.vx *= 0.94;
        agent.vy *= 0.94;
        agent.omegaRot *= 0.88;

        agent.px += agent.vx * frameScale;
        agent.py += agent.vy * frameScale;

        // Soft damped wall bouncing (No toroidal wrapping!)
        const margin = (meanRadius * 1.5) * 0.5 + 10; // logical margin based on 50% scale-down
        if (agent.px < margin) {
          agent.px = margin;
          agent.vx = -agent.vx * 0.45;
          agent.omegaRot = -agent.omegaRot * 0.5;
        } else if (agent.px > logicalWidth - margin) {
          agent.px = logicalWidth - margin;
          agent.vx = -agent.vx * 0.45;
          agent.omegaRot = -agent.omegaRot * 0.5;
        }

        if (agent.py < margin) {
          agent.py = margin;
          agent.vy = -agent.vy * 0.45;
          agent.omegaRot = -agent.omegaRot * 0.5;
        } else if (agent.py > logicalHeight - margin) {
          agent.py = logicalHeight - margin;
          agent.vy = -agent.vy * 0.45;
          agent.omegaRot = -agent.omegaRot * 0.5;
        }

        // ==========================================
        // COLLISION ATTACK COMBAT & SPORE FEEDING (CONTINUOUS SPECTRUM!)
        // ==========================================
        // A. Spore Consumption (All creatures can graze, but energy gained is scaled by 1.0 - carnivory!)
        const eatRadius = meanRadius * 1.5 * 0.5 + 4;
        for (const pellet of foodPellets) {
          let dx = pellet.x - agent.px;
          let dy = pellet.y - agent.py;
          if (dx > logicalWidth / 2) dx -= logicalWidth;
          if (dx < -logicalWidth / 2) dx += logicalWidth;
          if (dy > logicalHeight / 2) dy -= logicalHeight;
          if (dy < -logicalHeight / 2) dy += logicalHeight;

          const d = Math.sqrt(dx*dx + dy*dy);
          if (d <= eatRadius) {
            // Consume Spore!
            pellet.x = Math.random() * logicalWidth;
            pellet.y = Math.random() * logicalHeight; // respawn randomly

            const herbivoreEfficiency = 1.0 - agent.phenotype.carnivory;
            if (herbivoreEfficiency > 0.05) {
              const energyGain = 45.0 * herbivoreEfficiency * 1.25;
              agent.energy = Math.min(agent.phenotype.stomachCapacity, agent.energy + energyGain);
              agent.hasEaten = true; // Physiological feeding guarantee met!
            }
          }
        }

        // B. Predatory Biting (Any creature with carnivory >= 0.35 (Omnivores & Predators) can bite others!)
        if (agent.phenotype.carnivory >= 0.35) {
          const biteRange = meanRadius * 1.6 * 0.5 + 5.0;
          creatures.forEach(victim => {
            if (victim.id === agent.id) return;
            // Prevent suicide of the lineage: don't attack your own species/clones!
            if (victim.speciesId === agent.speciesId) return;

            let dx = victim.px - agent.px;
            let dy = victim.py - agent.py;
            if (dx > logicalWidth / 2) dx -= logicalWidth;
            if (dx < -logicalWidth / 2) dx += logicalWidth;
            if (dy > logicalHeight / 2) dy -= logicalHeight;
            if (dy < -logicalHeight / 2) dy += logicalHeight;

            const dist = Math.sqrt(dx*dx + dy*dy);
            if (dist <= biteRange) {
              // BITE ATTAINMENT!
              victim.energy = Math.max(0.0, victim.energy - 50.0); // deal damage to victim

              const carnivoreEfficiency = agent.phenotype.carnivory;
              const energyGain = 45.0 * carnivoreEfficiency * 1.25;
              agent.energy = Math.min(agent.phenotype.stomachCapacity, agent.energy + energyGain);
              agent.hasEaten = true; // Physiological feeding guarantee met!

              // Spawn bite crimson flash circles!
              biteImpacts.push({ x: victim.px, y: victim.py, age: 0 });

              logToConsole(`⚡ [BISS-ATTACKE] ${agent.phenotype.latinName.substring(0, 16)} #${agent.id} beißt #${victim.id}! (+${Math.round(energyGain)}nJ / -50nJ Schaden)`, "mutation");
            }
          });
        }

        // ==========================================
        // Mitosis Division (Asexual Reproduction)
        // ==========================================
        // Säule 3: Geschlechtliche Reife, variable Energieschwelle und Reibungstax!
        // We require the creature to have consumed at least one spore/prey (hasEaten) to reproduce!
        // Safeguards to prevent the infinite-mitosis "bomb" exploit: matureAge >= 600f (10s) & reproThreshold >= 65%!
        const canReproduce = agent.age >= Math.max(600, agent.phenotype.matureAge) && agent.hasEaten;
        const reachedReproThreshold = agent.energy >= agent.phenotype.stomachCapacity * Math.max(0.65, agent.phenotype.reproThreshold);

        if (canReproduce && reachedReproThreshold && creatures.length < 22) {
          const splitLoss = agent.phenotype.splitLoss;
          const energyPool = agent.energy;
          
          // Split energy: parent and child share remaining pool equally after splitLoss tax
          const parentEnergyAfter = energyPool * 0.4 * (1.0 - splitLoss);
          const childEnergyAfter = energyPool * 0.4 * (1.0 - splitLoss);
          agent.energy = parentEnergyAfter;
          agent.hasEaten = false; // Reset parent's feeding guarantee (must feed again to divide again!)

          // Lost energy triggers a physical kinetic recoil shockwave pushing them apart!
          const recoilVelocity = splitLoss * 15.0; // max 6.0 px/frame recoil force
          agent.vx += recoilVelocity * Math.cos(agent.headingAngle + Math.PI);
          agent.vy += recoilVelocity * Math.sin(agent.headingAngle + Math.PI);

          let childGenome = agent.genome;
          let isMutated = false;
          let isLamarckian = false;

          // Advanced Lamarckian Genetic Assimilation:
          if (agent.age > 1200) {
            let mCopy = [...agent.phenotype.methylations];
            let tempGenome = childGenome;
            for (let i = 16; i <= 31; i++) {
              if (mCopy[i] !== 0 && Math.random() < 0.25) {
                const baseVal = ALPHABET.indexOf(tempGenome[i]);
                const shift = mCopy[i];
                const assimilatedChar = ALPHABET[(baseVal + shift + 26) % 26];
                
                tempGenome = tempGenome.substring(0, i) + assimilatedChar + tempGenome.substring(i + 1);
                mCopy[i] = 0; // reset methylation offset since it is now genetically fixed!
                isLamarckian = true;
              }
            }
            if (isLamarckian) {
              childGenome = tempGenome;
              agent.phenotype.methylations = mCopy; // sync back to parent to prevent duplication
            }
          }

          // Use the child-mutations probability directly derived from parent's Repair Fidelity (Locus 11)!
          const childMutationRate = 1.0 - agent.phenotype.repairFidelity; // r- vs K-strategy mutations!
          if (Math.random() < childMutationRate) {
            const mut = mutateGenome(childGenome);
            childGenome = mut.newGenome;
            isMutated = true;
          }

          const childAntisense = getComplementaryString(childGenome);
          const childPhenotype = parseGenome(childGenome, childAntisense, [...agent.phenotype.methylations]);
          
          // Evolve a NEW species record if the genome mutated
          const childSpeciesId = childGenome;
          if (isMutated) {
            const childRecord: SpeciesRecord = {
              id: childSpeciesId,
              name: childPhenotype.latinName,
              genome: childGenome,
              antisense: childAntisense,
              parentSpeciesId: agent.speciesId,
              status: "alive",
              peakPopulation: 1,
              birthTime: Date.now(),
              generation: agent.generation + 1,
              carnivory: childPhenotype.carnivory
            };
            saveSpecies(childRecord);
            cachedAliveSpecies.push(childRecord); // Sync with local memory cache synchronously!
          }

          const child: CreatureAgent = {
            id: nextAgentId++,
            speciesId: childSpeciesId,
            genome: childGenome,
            antisense: childAntisense,
            phenotype: childPhenotype,
            px: (agent.px + Math.random() * 32 - 16 + logicalWidth) % logicalWidth,
            py: (agent.py + Math.random() * 32 - 16 + logicalHeight) % logicalHeight,
            vx: -agent.vx * 0.4 + recoilVelocity * Math.cos(agent.headingAngle),
            vy: -agent.vy * 0.4 + recoilVelocity * Math.sin(agent.headingAngle),
            headingAngle: agent.headingAngle + Math.PI, 
            omegaRot: 0,
            energy: childEnergyAfter, 
            age: 0,
            generation: agent.generation + 1,
            adrenaline: 1.0, // Initialize adrenaline to baseline!
            hasEaten: false, // MUST hunt/graze to reproduce!
            neuronStates: [],
            neuronActivations: []
          };

          newbornAgents.push(child);
          highestGeneration = Math.max(highestGeneration, child.generation);

          if (isLamarckian) {
            logToConsole(`⚡ [Lamarckismus] Spezies #${agent.id} hat erlernte Erfahrungen erfolgreich in das Erbgut des Nachkommen #${child.id} assimiliert!`, "repair");
          }
          logToConsole(`Mitose-Geburt! Spezies #${child.id} entstanden (Nachkomme von #${agent.id}, Gen: ${child.generation}${isMutated ? ", MUTIERT!" : ", Klon"}).`, isMutated ? "mutation" : "system");
        }

        // Render continuous agent
        renderer.render(agent.phenotype, timestamp, agent.px, agent.py, agent.headingAngle, agent.omegaRot);

        // Update selected inspect overlays in real-time inside the Top-Right HUD!
        if (agent.id === selectedAgentId) {
          updateInspectTraitsDashboard(agent);
          
          // Draw dashed selection indicator ring on the canvas
          if (ctx) {
            ctx.save();
            // Cyan for Prey, Red for Predator!
            ctx.strokeStyle = isPred ? "rgba(239, 68, 68, 0.75)" : "rgba(0, 242, 254, 0.75)";
            ctx.lineWidth = 1.5;
            ctx.setLineDash([4, 4]);
            ctx.beginPath();
            ctx.arc(agent.px, agent.py, meanRadius * 2.6 * 0.5, 0, Math.PI * 2);
            ctx.stroke();
            ctx.restore();
          }
        }
      });

      // Track active species extinction events
      activeSpeciesOnCanvas.forEach(speciesId => {
        // If this species was alive last frame but is now extinct inside the creatures array
        if (!currentAliveSpeciesThisFrame.has(speciesId)) {
          markSpeciesAsExtinct(speciesId);
          // Sync with our synchronous in-memory cache!
          cachedAliveSpecies = cachedAliveSpecies.filter(rec => rec.id !== speciesId);
          getSpeciesById(speciesId).then(rec => {
            if (rec) {
              logToConsole(`☠️ [Aussterben] Die Spezies '${rec.name}' ist im Ozean endgültig erloschen!`, "mutation");
            }
          });
        }
      });
      // Sync active species pointer
      activeSpeciesOnCanvas = currentAliveSpeciesThisFrame;

      // Filter deaths and inject births
      creatures = creatures.filter(agent => !deadAgentIds.includes(agent.id));
      creatures.push(...newbornAgents);

      // Maintain a stable target population of N=15 creatures synchronously! (NO 60fps file-thrashing!)
      const targetPopulation = 15;
      while (creatures.length < targetPopulation) {
        let g = generateRandomGenome(256);
        let gen = 1;

        // Introduce genetic diversity during restocking: 40% chance of a fresh new Urzelle (rando),
        // and 60% chance of cloning/restocking one of the surviving species from cache!
        if (cachedAliveSpecies.length > 0 && Math.random() < 0.60) {
          // Pick a random alive species from synchronous memory cache!
          const idx = Math.floor(Math.random() * cachedAliveSpecies.length);
          const record = cachedAliveSpecies[idx];
          g = record.genome;
          gen = record.generation;
        }

        const anti = getComplementaryString(g);
        const pheno = parseGenome(g, anti);

        // Instantly and synchronously push to creatures list (immediately increments length!)
        creatures.push({
          id: nextAgentId++,
          speciesId: g,
          genome: g,
          antisense: anti,
          phenotype: pheno,
          px: Math.random() * logicalWidth,
          py: Math.random() * logicalHeight,
          vx: (Math.random() * 0.8 - 0.4),
          vy: (Math.random() * 0.8 - 0.4),
          headingAngle: Math.random() * Math.PI * 2,
          omegaRot: 0,
          energy: 120 + Math.random() * 40,
          age: 0, 
          generation: gen,
          adrenaline: 1.0,
          hasEaten: true, // restored cells can seed the tank on their first cycle
          neuronStates: [],
          neuronActivations: []
        });

        // Ensure it's in our active species tracking
        activeSpeciesOnCanvas.add(g);
        
        logToConsole(`[Evolution] Populations-Absturz abgewehrt! Eine Zelle der Spezies '${pheno.latinName}' wurde freigesetzt (Gen: ${gen}).`);
      }

      // 3. Render glowing bite impact flashes on collision points
      if (ctx) {
        ctx.save();
        biteImpacts.forEach((impact) => {
          impact.age += 1;
          const progress = impact.age / 24; // lasts 24 frames
          
          const radius = 5 + progress * 28;
          const opacity = 1.0 - progress;

          ctx.strokeStyle = `rgba(255, 0, 127, ${opacity})`;
          ctx.lineWidth = 2.0;
          ctx.beginPath();
          ctx.arc(impact.x, impact.y, radius, 0, Math.PI * 2);
          ctx.stroke();
        });
        // Clear finished impacts
        biteImpacts = biteImpacts.filter(i => i.age < 24);
        ctx.restore();
      }

      // Safeguard selection: if selected agent died, select oldest living agent
      if (selectedAgentId !== null && !creatures.some(a => a.id === selectedAgentId) && creatures.length > 0) {
        selectedAgentId = creatures[0].id;
        updateGenomeGrid(creatures[0].phenotype);
        renderSynapseWeb(creatures[0].phenotype);
      }

      // Update Evo Stats Dashboard Bar occasionally
      if (timestamp - lastUiUpdate > 150) {
        if (evoPopCountEl) evoPopCountEl.innerText = `${creatures.length} Organismen`;
        if (evoGenMaxEl) evoGenMaxEl.innerText = `${highestGeneration}. Gen`;
        if (evoFoodCountEl) evoFoodCountEl.innerText = `${foodPellets.length} Sporen`;
        lastUiUpdate = timestamp;

        // Periodically save exact simulation running state to disk (once every 1.5 seconds)
        if (timestamp - lastStateSaveTime > 1500) {
          lastStateSaveTime = timestamp;
          
          const statePayload = {
            creatures: creatures.map(c => ({
              id: c.id,
              speciesId: c.speciesId,
              genome: c.genome,
              antisense: c.antisense,
              methylations: c.phenotype.methylations, // preserve epigenetically learned synapses!
              px: c.px,
              py: c.py,
              vx: c.vx,
              vy: c.vy,
              headingAngle: c.headingAngle,
              omegaRot: c.omegaRot,
              energy: c.energy,
              age: c.age,
              generation: c.generation,
              adrenaline: c.adrenaline,
              hasEaten: c.hasEaten,
              neuronStates: c.neuronStates,
              neuronActivations: c.neuronActivations
            })),
            foodPellets: foodPellets,
            nextAgentId: nextAgentId,
            highestGeneration: highestGeneration
          };
          
          saveSimulationState(statePayload).catch(err => {
            console.error("Failed to auto-save simulation state:", err);
          });
        }
      }
    }
  }

  requestAnimationFrame(animate);
}

// ============================================================================
// Interactive Mode Switch Handler (Sandbox vs. Evo)
// ============================================================================
function switchToMode(mode: SimMode): void {
  currentMode = mode;

  const dpr = window.devicePixelRatio || 1;
  const logicalWidth = canvas.width / dpr;
  const logicalHeight = canvas.height / dpr;

  if (mode === "sandbox") {
    modeSandboxBtn.classList.add("active");
    modeEvoBtn.classList.remove("active");
    evoStatsCard.style.display = "none";
    brainControlCard.style.display = "block";
    sandboxButtonGroup.style.display = "flex";
    traitsHeaderTitle.innerText = "Phänotyp Merkmale";
    inspectOverlay.style.display = "none"; // Hide Inspect HUD in Sandbox!

    // Re-initialize Sandbox creature
    updateSimulation();
    logToConsole("Kombüse umgeschaltet auf Sandbox-Labor. Einzelnes physisches Exemplar geladen.");
  } else {
    modeSandboxBtn.classList.remove("active");
    modeEvoBtn.classList.add("active");
    evoStatsCard.style.display = "block";
    brainControlCard.style.display = "none"; // AI is always active in Evo!
    sandboxButtonGroup.style.display = "none";
    traitsHeaderTitle.innerText = "Ausgewählte Spezies";
    inspectOverlay.style.display = "flex"; // Open the Inspect HUD in Evo Mode!

    // Query the local server for a saved simulation resumption state
    getSavedSimulationState().then(state => {
      if (state && !state.empty && state.creatures && state.creatures.length > 0) {
        // 1. RESUME SAVED EXPERIMENT STATE DOWN TO THE PIXEL!
        creatures = state.creatures.map((c: any) => ({
          id: c.id,
          speciesId: c.speciesId,
          genome: c.genome,
          antisense: c.antisense,
          phenotype: parseGenome(c.genome, c.antisense, c.methylations), // Reconstruct brain and spine in-memory!
          px: c.px,
          py: c.py,
          vx: c.vx,
          vy: c.vy,
          headingAngle: c.headingAngle,
          omegaRot: c.omegaRot,
          energy: c.energy,
          age: c.age,
          generation: c.generation,
          adrenaline: c.adrenaline !== undefined ? c.adrenaline : 1.0,
          hasEaten: c.hasEaten !== undefined ? c.hasEaten : true,
          neuronStates: c.neuronStates !== undefined ? c.neuronStates : [],
          neuronActivations: c.neuronActivations !== undefined ? c.neuronActivations : []
        }));

        foodPellets = state.foodPellets || [];
        nextAgentId = state.nextAgentId || 1;
        highestGeneration = state.highestGeneration || 1;

        // Fetch alive species lsit to keep our synchronous memory cache in sync!
        getAliveSpecies().then(aliveList => {
          cachedAliveSpecies = aliveList;
          activeSpeciesOnCanvas.clear();
          creatures.forEach(agent => {
            activeSpeciesOnCanvas.add(agent.speciesId);
            highestGeneration = Math.max(highestGeneration, agent.generation);
          });

          // Select first creature to draw its brain
          if (creatures.length > 0) {
            selectedAgentId = creatures[0].id;
            updateGenomeGrid(creatures[0].phenotype);
            renderSynapseWeb(creatures[0].phenotype);
          }
          logToConsole(`[Datenbank] Vorheriges Experiment fortgesetzt! ${creatures.length} Organismen exakt am Todesort reaktiviert.`, "repair");
        });

      } else {
        // 2. FALLBACK: INITIALIZE FRESH FOUNDERS POPULATION
        getAliveSpecies().then(aliveList => {
          cachedAliveSpecies = aliveList;
          creatures = [];
          nextAgentId = 1;
          highestGeneration = 1;
          activeSpeciesOnCanvas.clear();

          const numToSpawn = 15;

          for (let k = 0; k < numToSpawn; k++) {
            let g = generateRandomGenome(256);
            let parentId: string | null = null;
            let gen = 1;

            if (aliveList.length > 0) {
              // Re-load and spawn currently alive species from previous sessions!
              const idx = Math.floor(Math.random() * aliveList.length);
              const record = aliveList[idx];
              g = record.genome;
              parentId = record.parentSpeciesId;
              gen = record.generation;
            }

            const anti = getComplementaryString(g);
            const pheno = parseGenome(g, anti);

            // Save new founder to DB if not already present
            getSpeciesById(g).then(exists => {
              if (!exists) {
                const rec: SpeciesRecord = {
                  id: g,
                  name: pheno.latinName,
                  genome: g,
                  antisense: anti,
                  parentSpeciesId: parentId,
                  status: "alive",
                  peakPopulation: 1,
                  birthTime: Date.now(),
                  generation: gen,
                  carnivory: pheno.carnivory
                };
                saveSpecies(rec);
                cachedAliveSpecies.push(rec); // sync synchronously!
              }
            });

            creatures.push({
              id: nextAgentId++,
              speciesId: g,
              genome: g,
              antisense: anti,
              phenotype: pheno,
              px: Math.random() * logicalWidth,
              py: Math.random() * logicalHeight,
              vx: (Math.random() * 0.8 - 0.4),
              vy: (Math.random() * 0.8 - 0.4),
              headingAngle: Math.random() * Math.PI * 2,
              omegaRot: 0,
              energy: 120 + Math.random() * 40, 
              age: Math.floor(Math.random() * 600), 
              generation: gen,
              adrenaline: 1.0,
              hasEaten: true, // founders can seed the tank on their first cycle
              neuronStates: [],
              neuronActivations: []
            });
            
            activeSpeciesOnCanvas.add(g);
            highestGeneration = Math.max(highestGeneration, gen);
          }

          // Initialize M=30 drifting green food pellets
          foodPellets = [];
          for (let m = 0; m < 30; m++) {
            foodPellets.push({
              x: Math.random() * logicalWidth,
              y: Math.random() * logicalHeight,
              vx: (Math.random() * 0.3 - 0.15),
              vy: (Math.random() * 0.3 - 0.15)
            });
          }

          // Select the first agent for Left Sidebar inspect
          if (creatures.length > 0) {
            selectedAgentId = creatures[0].id;
            updateGenomeGrid(creatures[0].phenotype);
            renderSynapseWeb(creatures[0].phenotype);
          }

          logToConsole(`Kombüse umgeschaltet auf Evolutions-Ozean. N=15 Organismen freigesetzt (${aliveList.length} Spezies aus DB rekonstruiert).`);
        });
      }
    }).catch(err => {
      console.error("Failed to load simulation state:", err);
    });
  }
}

modeSandboxBtn.addEventListener("click", () => {
  isDiagnosticsModalOpen = false;
  diagnosticsModal.style.display = "none";
  switchToMode("sandbox");
});

modeEvoBtn.addEventListener("click", () => {
  isDiagnosticsModalOpen = false;
  diagnosticsModal.style.display = "none";
  switchToMode("evo");
});

// Close the Inspect Overlay HUD button
closeInspectBtn.addEventListener("click", () => {
  inspectOverlay.style.display = "none";
  selectedAgentId = null;
  isDiagnosticsModalOpen = false;
  diagnosticsModal.style.display = "none";
  logToConsole("[Schnittstelle] Diagnose-HUD geschlossen.");
});

// Open near-fullscreen diagnostics modal (Stammbaum & Archiv)
openDiagnosticsBtn.addEventListener("click", () => {
  isDiagnosticsModalOpen = true;
  diagnosticsModal.style.display = "flex";
  
  // Synchronously trigger Stammbaum and Catalog rebuild
  updateStammbaumAndSpeciesModal();
  logToConsole("[Schnittstelle] Stammbaum-Archiv geöffnet.");
});

// Close near-fullscreen diagnostics modal
closeModalBtn.addEventListener("click", () => {
  isDiagnosticsModalOpen = false;
  diagnosticsModal.style.display = "none";
  modalSpeciesDetails.style.display = "none"; // Hide genetic details drawer too!
  logToConsole("[Schnittstelle] Stammbaum-Archiv geschlossen.");
});

// Close the slide-out genetic details drawer inside the modal
closeDrawerBtn.addEventListener("click", () => {
  modalSpeciesDetails.style.display = "none";
  // Remove highlights
  document.querySelectorAll(".roster-item-active").forEach(el => el.classList.remove("roster-item-active"));
  document.querySelectorAll(".stammbaum-node-active").forEach(el => el.classList.remove("stammbaum-node-active"));
});

// Click event delegation for selecting a species card in the Directory List (Arten-Archiv)
modalSpeciesList.addEventListener("click", (e) => {
  const item = (e.target as HTMLElement).closest(".roster-item") as HTMLDivElement | null;
  if (!item) return;
  const id = item.getAttribute("data-id");
  if (id) selectSpeciesForInspection(id, item);
});

// Click event delegation for selecting a node in the Visual Lineage Tree (Stammbaum)
modalStammbaumTreeContainer.addEventListener("click", (e) => {
  const item = (e.target as HTMLElement).closest(".stammbaum-node") as HTMLDivElement | null;
  if (!item) return;
  const id = item.getAttribute("data-id");
  if (id) selectSpeciesForInspection(id, item);
});

/**
 * Click selection inside the canvas viewport.
 * Find the closest swimming agent in Evo Mode and select it for Left Sidebar inspection.
 */
canvas.addEventListener("click", (e) => {
  if (currentMode !== "evo") return;

  const rect = canvas.getBoundingClientRect();
  const scaleX = canvas.width / rect.width;
  const scaleY = canvas.height / rect.height;
  
  const dpr = window.devicePixelRatio || 1;
  const mx = ((e.clientX - rect.left) * scaleX) / dpr;
  const my = ((e.clientY - rect.top) * scaleY) / dpr;

  let closestAgent: CreatureAgent | null = null;
  let minDist = 120; // selection distance threshold

  creatures.forEach(agent => {
    const dx = agent.px - mx;
    const dy = agent.py - my;
    const dist = Math.sqrt(dx*dx + dy*dy);
    if (dist < minDist) {
      minDist = dist;
      closestAgent = agent;
    }
  });

  if (closestAgent) {
    const agent = closestAgent as CreatureAgent;
    selectedAgentId = agent.id;
    
    // Open the Inspect HUD overlay if it was closed
    inspectOverlay.style.display = "flex";

    updateGenomeGrid(agent.phenotype);
    renderSynapseWeb(agent.phenotype);
    logToConsole(`[Untersuchung] Spezies #${agent.id} ausgewählt (Generation: ${agent.generation}, Energie: ${Math.round(agent.energy)}nJ, Alter: ${Math.round(agent.age/60)}s).`);
  }
});

// Window Resize Listener for full-screen immersive sizing
window.addEventListener("resize", () => {
  resizeCanvas();
});

// Keyboard Event Listeners for Muscle Stimulation
window.addEventListener("keydown", (e) => {
  const k = e.key.toLowerCase();
  if (k in keys) {
    keys[k as keyof typeof keys] = true;
    e.preventDefault();
  }
});

window.addEventListener("keyup", (e) => {
  const k = e.key.toLowerCase();
  if (k in keys) {
    keys[k as keyof typeof keys] = false;
    e.preventDefault();
  }
});

// Controls Event Listeners
shuffleBtn.addEventListener("click", () => {
  sandboxGenome = generateRandomGenome(256); // Reset to standard 256
  sandboxAntisense = getComplementaryString(sandboxGenome); // Perfect complementary reset
  updateSimulation();
  logToConsole("Genom komplett neu gemischt (Sense/Antisense Double Helix). Neue biologische Urzelle entstanden.");
});

/**
 * DNA Mutation handler with real-time delayed polymerase replication repair check
 * Supports DNA-encoded Insertion (grow) and Deletion (shrink) mutations!
 */
mutateBtn.addEventListener("click", () => {
  const insRate = sandboxPhenotype.insertionRate; // Mapped from Locus 9
  const delRate = sandboxPhenotype.deletionRate;  // Mapped from Locus 10
  
  const currentLength = sandboxGenome.length;
  const roll = Math.random();
  
  const originalSenseBackup = sandboxGenome;
  const originalAntisenseBackup = sandboxAntisense;

  let mutatedIndex = -1;
  let oldChar = "";
  let newChar = "";
  let mutationType: "point" | "insertion" | "deletion" = "point";

  if (roll < delRate && currentLength > 128) {
    mutationType = "deletion";
    mutatedIndex = Math.floor(Math.random() * currentLength);
    oldChar = sandboxGenome[mutatedIndex];
    
    sandboxGenome = sandboxGenome.substring(0, mutatedIndex) + sandboxGenome.substring(mutatedIndex + 1);
    updateSimulation(mutatedIndex);
    logToConsole(`Locus ${mutatedIndex}: REPLIKATIONS-DELETION detektiert! Eine Base wurde übersprungen. Genom-Länge sinkt auf ${sandboxGenome.length} Loci! Polymerase-Reparatur active...`, "mutation");
    
  } else if (roll < delRate + insRate && currentLength < 384) {
    mutationType = "insertion";
    mutatedIndex = Math.floor(Math.random() * currentLength);
    oldChar = sandboxGenome[mutatedIndex];
    newChar = oldChar; 
    
    sandboxGenome = sandboxGenome.substring(0, mutatedIndex) + newChar + sandboxGenome.substring(mutatedIndex);
    updateSimulation(mutatedIndex);
    logToConsole(`Locus ${mutatedIndex}: REPLIKATIONS-INSERTION detektiert! Eine Base wurde dupliziert. Kaskaden-Frameshift! Genom-Länge steigt auf ${sandboxGenome.length} Loci!`, "mutation");
    
  } else {
    mutationType = "point";
    mutatedIndex = Math.floor(Math.random() * currentLength);
    const currentChar = sandboxGenome[mutatedIndex];
    oldChar = currentChar;
    
    newChar = currentChar;
    while (newChar === currentChar) {
      const r = Math.floor(Math.random() * 26);
      newChar = ALPHABET[r];
    }

    const oldAntiChar = sandboxAntisense[mutatedIndex];

    sandboxGenome = sandboxGenome.substring(0, mutatedIndex) + newChar + sandboxGenome.substring(mutatedIndex + 1);
    updateSimulation(mutatedIndex);
    logToConsole(`Locus ${mutatedIndex}: Replikations-Mismatch detektiert! Sense '${newChar}' unpaarig zu Antisense '${oldAntiChar}'! Polymerase-Reparatur active...`, "mutation");
  }

  mutateBtn.disabled = true;

  setTimeout(() => {
    mutateBtn.disabled = false;
    
    const repairRoll = Math.random();
    const fidelity = sandboxPhenotype.repairFidelity; 

    if (repairRoll < fidelity) {
      sandboxGenome = originalSenseBackup;
      sandboxAntisense = originalAntisenseBackup;
      
      updateSimulation(); 
      logToConsole("DNA-Polymerase hat Replikationsfehler erfolgreich repariert! Mutation abgewehrt und Genom-Länge stabilisiert.", "repair");
    } else {
      sandboxAntisense = getComplementaryString(sandboxGenome);
      
      const oldPheno = parseGenome(originalSenseBackup, originalAntisenseBackup, sandboxPhenotype.methylations);
      updateSimulation(mutatedIndex);
      
      const newPheno = sandboxPhenotype;
      
      logToConsole(`Reparatur gescheitert. Mutation permanent etabliert! Antisense-Strang korrigiert auf komplementäre Basen. Genom-Länge beträgt nun ${sandboxGenome.length} Loci.`, "repair");
      
      if (mutationType === "point") {
        const explanation = explainMutation(mutatedIndex, oldChar, newChar, oldPheno, newPheno);
        logToConsole(explanation, "system");
      } else if (mutationType === "insertion") {
        logToConsole(`[Frameshift] Eine Genom-Duplikation wurde erfolgreich stabilisiert (+1). Neue proteincodierende Areale wurden im Euchromatin geöffnet!`, "system");
      } else {
        logToConsole(`[Frameshift] Eine Genom-Deletion wurde erfolgreich stabilisiert (-1). Segment-Loci wurden gelöscht und stumm geschaltet.`, "system");
      }
    }
  }, 950);
});

// Evo Mode action button event handlers
injectUrzelleBtn.addEventListener("click", async () => {
  if (currentMode !== "evo") return;

  const dpr = window.devicePixelRatio || 1;
  const logicalWidth = canvas.width / dpr;
  const logicalHeight = canvas.height / dpr;

  const g = generateRandomGenome(256);
  const anti = getComplementaryString(g);
  const pheno = parseGenome(g, anti);

  // Save new species record to our physical file database
  const rec: SpeciesRecord = {
    id: g,
    name: pheno.latinName,
    genome: g,
    antisense: anti,
    parentSpeciesId: null, // founder
    status: "alive",
    peakPopulation: 1,
    birthTime: Date.now(),
    generation: 1,
    carnivory: pheno.carnivory
  };

  try {
    await saveSpecies(rec);
    cachedAliveSpecies.push(rec); // Sync with local memory cache synchronously!
    logToConsole(`⚡ [Datenbank] '${pheno.latinName}' erfolgreich im lokalen Speicher registriert!`, "repair");
  } catch (err) {
    console.error("Failed to persist injected Urzelle:", err);
    logToConsole(`❌ [Datenbank-Fehler] Konnte Urzelle nicht im Speicher sichern!`, "mutation");
  }

  const agent: CreatureAgent = {
    id: nextAgentId++,
    speciesId: g,
    genome: g,
    antisense: anti,
    phenotype: pheno,
    px: Math.random() * logicalWidth,
    py: Math.random() * logicalHeight,
    vx: (Math.random() * 0.8 - 0.4),
    vy: (Math.random() * 0.8 - 0.4),
    headingAngle: Math.random() * Math.PI * 2,
    omegaRot: 0,
    energy: 150,
    age: 0,
    generation: 1,
    adrenaline: 1.0,
    hasEaten: true, // manually injected cells can seed on their first cycle
    neuronStates: [],
    neuronActivations: []
  };

  creatures.push(agent);
  activeSpeciesOnCanvas.add(g);

  // Instantly select our newly injected god-mode Urzelle to inspect its brain live!
  selectedAgentId = agent.id;
  inspectOverlay.style.display = "flex";
  updateGenomeGrid(agent.phenotype);
  renderSynapseWeb(agent.phenotype);

  logToConsole(`⚡ [Injektion] Eine neue Gen-1 Urzelle '${pheno.latinName}' wurde manuell in den Ozean injiziert!`, "system");
});

resetEvolutionBtn.addEventListener("click", () => {
  if (currentMode !== "evo") return;

  // Clear all local in-memory states instantly to prevent any carried-over rewrites!
  creatures = [];
  cachedAliveSpecies = [];
  activeSpeciesOnCanvas.clear();
  nextAgentId = 1;
  highestGeneration = 1;

  // Clear physical database file and simulation state files synchronously!
  clearDb().then(() => {
    clearSimulationState().then(() => {
      logToConsole("🧹 [Reset] Die gesamte Evolutionsgeschichte wurde gelöscht. Ein neues biologisches Zeitalter bricht an!", "system");
      
      // Close inspect overlay HUD
      inspectOverlay.style.display = "none";
      selectedAgentId = null;

      // Reset ocean with 15 fresh Urzellen
      switchToMode("evo");
    });
  });
});

// Initialization
window.addEventListener("load", () => {
  resizeCanvas(); // Make canvas fill full screen viewport in native high-DPI crispness!
  renderer = new CreatureRenderer(canvas);
  
  // Initialize client-side file-based Database REST API check!
  initDb().then(() => {
    logToConsole("[Datenbank] Physischer Lokaler Gen-Speicher erfolgreich initialisiert.");
  });

  // Parse initial Sandbox specimen
  sandboxPhenotype = parseGenome(sandboxGenome, sandboxAntisense);
  updateSimulation();
  
  requestAnimationFrame(animate);
  logToConsole("Simulation gestartet. Biologisches Substrat geladen. Steuerung: WASD + QE.");
});
