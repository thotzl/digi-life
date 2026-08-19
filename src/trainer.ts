import { generateWorld, getVectoredCurrentAt, ProceduralWorld, checkObstacleCollision } from './shared/mapGenerator';
import { parseGenome, generateRandomGenome, getComplementaryString, executeBrain, mutateGenome } from './biology/dna';
import { CreatureRenderer } from './render/creatureRenderer';
import { CreatureAgent, FoodSpore } from './shared/types';
import { SpatialGrid } from './server/spatialGrid';

// Dimensions for the mini-canvases
const canvasWidth = 500;
const canvasHeight = 500;
const epochDurationTicks = 300; // 5 seconds at 60Hz

// DOM Elements
const gridContainer = document.getElementById("sandbox-grid") as HTMLDivElement;
const btnStart = document.getElementById("btn-start") as HTMLButtonElement;
const btnReset = document.getElementById("btn-reset-train") as HTMLButtonElement;
const txtDna = document.getElementById("txt-dna") as HTMLTextAreaElement;
const btnCopyDna = document.getElementById("btn-copy-dna") as HTMLButtonElement;

// Diagnostics Sidebar Elements
const diagCanvas = document.getElementById("diagnostics-preview-canvas") as HTMLCanvasElement;
const diagCtx = diagCanvas ? diagCanvas.getContext("2d")! : null;
const diagRenderer = diagCanvas ? new CreatureRenderer(diagCanvas) : null;
const focusGenome = document.getElementById("focus-genome") as HTMLTextAreaElement;

const statGen = document.getElementById("stat-gen") as HTMLSpanElement;
const statBestFit = document.getElementById("stat-best-fit") as HTMLSpanElement;
const statAvgFit = document.getElementById("stat-avg-fit") as HTMLSpanElement;
const statTimer = document.getElementById("stat-timer") as HTMLSpanElement;

const sliderGridSize = document.getElementById("slider-grid-size") as HTMLInputElement;
const sliderSpeedup = document.getElementById("slider-speedup") as HTMLInputElement;
const sliderEliteRatio = document.getElementById("slider-elite-ratio") as HTMLInputElement;
const sliderMutation = document.getElementById("slider-mutation-rate") as HTMLInputElement;
const sliderInflow = document.getElementById("slider-inflow-rate") as HTMLInputElement;
const sliderHof = document.getElementById("slider-hof-rate") as HTMLInputElement;
const chkMultiTrial = document.getElementById("chk-multi-trial") as HTMLInputElement;

const lblGridSize = document.getElementById("lbl-grid-size") as HTMLSpanElement;
const lblSpeedup = document.getElementById("lbl-speedup") as HTMLSpanElement;
const lblEliteRatio = document.getElementById("lbl-elite-ratio") as HTMLSpanElement;
const lblMutation = document.getElementById("lbl-mutation-rate") as HTMLSpanElement;
const lblInflow = document.getElementById("lbl-inflow-rate") as HTMLSpanElement;
const lblHof = document.getElementById("lbl-hof-rate") as HTMLSpanElement;

const focusMeta = document.getElementById("focus-meta") as HTMLParagraphElement;
const neuronMeta = document.getElementById("neuron-meta") as HTMLDivElement;
const brainContainer = document.getElementById("inspect-brain-container") as HTMLDivElement;

// State Variables
let isRunning = false;
let warpSpeed = 1;
let N = 16; // Grid count (N Sandboxen)
let eliteRatio = 0.25; // Keep Top 25%
let genomeMutationRate = 0.15; // 15% genome mutation rate
let randomInflowRate = 0.10; // 10% random immigrants
let randomHofRate = 0.10; // 10% Hall of Fame re-injection
let isMultiTrial = false; // Multi-Trial evaluation
let currentTrial = 1; // Current active trial count (1 to 3)
const totalTrials = 3; // 3 runs per generation
let currentGeneration = 1;
let highestFitness = 0.0;
let epochTicks = 0;
let runId = "default_run";

interface Sandbox {
  id: number;
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  renderer: CreatureRenderer;
  agent: CreatureAgent;
  foods: FoodSpore[];
  finished: boolean;
  finishTick?: number;
  startDistance: number;
  currentFitness: number;
  accumulatedFitness?: number;
  distanceTraveled: number; // cumulative physical path length
  world: ProceduralWorld;
  epochTicks: number;
}

let sandboxes: Sandbox[] = [];
let selectedSandboxIdx = 0;
let hoveredNeuronId: number | null = null;

// Expose state closures to window for live DevTools debugging
(window as any).getIsRunning = () => isRunning;
(window as any).getEpochTicks = () => epochTicks;
(window as any).getIsEvaluating = () => isEvaluating;
(window as any).getSandboxes = () => sandboxes;

// --------------------------------------------------------------------------
// Physically Equipped, Neurally Naive (PE-NN) Progenitor Factory
// --------------------------------------------------------------------------
function generatePEN_Progenitor(): string {
  let attempts = 0;
  while (attempts < 1500) {
    attempts++;
    const genome = generateRandomGenome(256);
    const phenotype = parseGenome(genome, getComplementaryString(genome));
    
    // Body size must be balanced (Radius 16 to 28)
    const r = phenotype.spinalHarmonics.meanRadius;
    if (r < 16 || r > 28) continue;

    // Must have exactly 1 or 2 forward-facing algae food chemoreceptors / photoreceptors
    const foodSensors = phenotype.organelles.filter(patch => {
      const angle = patch.angle; // 0° to 180°
      const isForward = angle <= 45; // Front quadrant
      const isReceptor = patch.expressionStyle < 0.72; // Not a muscle fin
      const isAlgaeTuned = patch.spectralAffinity > 0.25 && patch.spectralAffinity < 0.8;
      return isForward && isReceptor && isAlgaeTuned;
    });

    if (foodSensors.length >= 1 && foodSensors.length <= 2) {
      console.log(`[TRAINER] PE-NN Progenitor successfully compiled in ${attempts} attempts! (Radius: ${r.toFixed(1)}, Sensors: ${foodSensors.length})`);
      return genome;
    }
  }

  // Pure fallback seed string if search exceeds limits
  return "HJKLABCDPQRS1234EFGHTRUSTANDBENDPROGENITORALIFEWELLFORMEDMEMBRANEFOURIERSEGMENTSHARMONICSWAVEPHASEPULSESTIFFNESS";
}

// --------------------------------------------------------------------------
// REST API Server Connections
// --------------------------------------------------------------------------
const BACKEND_BASE = `http://${window.location.hostname || 'localhost'}:3002`;

async function fetchServerPopulation(): Promise<any[]> {
  try {
    const res = await fetch(`${BACKEND_BASE}/api/trainer/population?runId=${runId}&limit=${N}`);
    if (res.ok) {
      return await res.json();
    }
  } catch (err) {
    console.warn("[Trainer API] Failed to fetch existing population from SQLite server, defaulting to random setup.", err);
  }
  return [];
}

async function fetchServerHof(): Promise<any[]> {
  try {
    const res = await fetch(`${BACKEND_BASE}/api/trainer/hof?runId=${runId}&limit=10`);
    if (res.ok) {
      return await res.json();
    }
  } catch (err) {
    console.warn("[Trainer API] Failed to fetch Hall of Fame from SQLite server:", err);
  }
  return [];
}

async function fetchServerRuns(): Promise<any[]> {
  try {
    const res = await fetch(`${BACKEND_BASE}/api/trainer/runs`);
    if (res.ok) {
      return await res.json();
    }
  } catch (err) {
    console.error("[Trainer API] Failed to fetch training runs from SQLite:", err);
  }
  return [];
}

async function saveServerGeneration(generation: number, population: { name: string; genome: string; fitness: number }[]) {
  try {
    await fetch(`${BACKEND_BASE}/api/trainer/generation`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ runId, generation, population })
    });
  } catch (err) {
    console.error("[Trainer API] Error saving generation to server:", err);
  }
}

async function resetServerTrainer() {
  try {
    await fetch(`${BACKEND_BASE}/api/trainer/reset`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ runId })
    });
  } catch (err) {
    console.error("[Trainer API] Error resetting trainer server db:", err);
  }
}

async function toggleBackgroundSimulation(running: boolean) {
  try {
    await fetch(`${BACKEND_BASE}/api/simulation/toggle`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ running })
    });
  } catch (err) {
    console.error("[Trainer API] Error toggling background simulation:", err);
  }
}

async function applyServerChampion(genome: string) {
  try {
    const res = await fetch(`${BACKEND_BASE}/api/trainer/apply`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ genome })
    });
    if (res.ok) {
      alert("Champion DNA successfully applied to the live Substrate ocean! (Rules updated & hot-reloaded)");
    }
  } catch (err) {
    console.error("[Trainer API] Error applying champion to server:", err);
  }
}

// --------------------------------------------------------------------------
// Sandbox Initializations
// --------------------------------------------------------------------------
function initSandbox(id: number, canvas: HTMLCanvasElement, parentGenome: string, mutate = false): Sandbox {
  const ctx = canvas.getContext('2d')!;
  const renderer = new CreatureRenderer(canvas);
  
  // Use unique seed per sandbox world
  const world = generateWorld("SANDBOX_SEED_" + id + "_GEN_" + currentGeneration);

  // Generate / Mutate genome
  let genome = parentGenome;
  if (mutate && Math.random() < genomeMutationRate) {
    // apply slight random mutation to brain segment
    const mut = mutateGenome(genome);
    genome = mut.newGenome;
  }

  const anti = getComplementaryString(genome);
  const pheno = parseGenome(genome, anti);

  const agent: CreatureAgent = {
    id,
    speciesId: genome,
    genome,
    antisense: anti,
    phenotype: pheno,
    px: canvasWidth / 2,
    py: canvasHeight / 2,
    vx: 0,
    vy: 0,
    headingAngle: Math.random() * Math.PI * 2,
    omegaRot: 0,
    energy: 100,
    age: 0,
    generation: currentGeneration,
    adrenaline: 1.0,
    hasEaten: false,
    neuronStates: [],
    neuronActivations: [],
    bendAngle: 0.0
  };

  // Setup two food spores (one plant, one meat) at a minimum distance of 80 pixels
  const foods: FoodSpore[] = [
    { x: 0, y: 0, vx: 0, vy: 0, type: 'plant' },
    { x: 0, y: 0, vx: 0, vy: 0, type: 'meat' }
  ];

  foods.forEach(spore => {
    let validSpawn = false;
    while (!validSpawn) {
      spore.x = 25 + Math.random() * (canvasWidth - 50);
      spore.y = 25 + Math.random() * (canvasHeight - 50);
      const dx = spore.x - agent.px;
      const dy = spore.y - agent.py;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist >= 80) {
        validSpawn = true;
      }
    }
  });

  // Calculate startDistance to the food that is compatible with the agent's diet
  const targetType = pheno.carnivory >= 0.35 ? 'meat' : 'plant';
  const targetFood = foods.find(f => f.type === targetType)!;
  const startDistance = Math.sqrt((targetFood.x - agent.px) ** 2 + (targetFood.y - agent.py) ** 2);

  return {
    id,
    canvas,
    ctx,
    renderer,
    agent,
    foods,
    finished: false,
    startDistance,
    currentFitness: 0,
    distanceTraveled: 0.0,
    world,
    epochTicks: 0
  };
}

async function rebuildSandboxGrid() {
  gridContainer.innerHTML = "";
  sandboxes = [];
  
  // 1. Fetch both active population and all-time Hall of Fame
  const savedPool = await fetchServerPopulation();
  const savedHof = await fetchServerHof();
  
  if (savedPool.length > 0) {
    currentGeneration = savedPool[0].generation + 1;
    highestFitness = savedPool[0].fitness;
    statGen.innerText = (currentGeneration - 1).toString();
    statBestFit.innerText = highestFitness.toFixed(1);
    txtDna.value = savedPool[0].genome;
  } else {
    currentGeneration = 1;
    highestFitness = 0.0;
    statGen.innerText = "1";
    statBestFit.innerText = "0.0";
    txtDna.value = "";
  }

  // Calculate distinct populations for this generation
  const eliteCount = Math.max(1, Math.ceil(N * eliteRatio));
  const hofCount = Math.floor(N * randomHofRate);
  const inflowCount = Math.floor(N * randomInflowRate);

  for (let i = 0; i < N; i++) {
    const card = document.createElement("div");
    card.className = `sandbox-card ${i === selectedSandboxIdx ? "selected" : ""}`;
    card.setAttribute("data-idx", i.toString());

    const canvas = document.createElement("canvas");
    canvas.width = canvasWidth;
    canvas.height = canvasHeight;

    const meta = document.createElement("div");
    meta.className = "sandbox-meta";
    meta.innerHTML = `<span>#${i + 1}</span><span id="card-fit-${i}">F: 0.0</span>`;

    card.appendChild(canvas);
    card.appendChild(meta);
    gridContainer.appendChild(card);

    // Setup click selection for diagnostics focusing
    canvas.addEventListener("click", () => {
      document.querySelectorAll(".sandbox-card").forEach(c => c.classList.remove("selected"));
      card.classList.add("selected");
      selectedSandboxIdx = i;
      compileBrainSVG();
    });

    // Populate Sandbox
    let parent = "";
    let shouldMutate = false;

    if (savedPool.length > 0) {
      // A. We have a saved pool of champions!
      if (i < eliteCount) {
        // 1. Elite Survivors: load exact saved champions unmutated
        parent = (i < savedPool.length) ? savedPool[i].genome : savedPool[0].genome;
        shouldMutate = false;
      } else if (i < eliteCount + hofCount && savedHof.length > 0) {
        // 2. Hall of Fame re-injection: clone from historically best ever, and mutate slightly
        const hofIdx = (i - eliteCount) % savedHof.length;
        parent = savedHof[hofIdx].genome;
        shouldMutate = true;
      } else if (i < eliteCount + hofCount + inflowCount) {
        // 3. Random Immigrants: spawn a brand new independent naive progenitor
        parent = generatePEN_Progenitor();
        shouldMutate = false;
      } else {
        // 4. Cloned Descendants: clone from all champions evenly (random selection from savedPool)
        const parentIdx = Math.floor(Math.random() * savedPool.length);
        parent = savedPool[parentIdx].genome;
        shouldMutate = true;
      }
    } else {
      // B. GREENFIELD START (Generation 1):
      // Generate N completely different, independent random PE-NN progenitors!
      parent = generatePEN_Progenitor();
      shouldMutate = false; // start pure and unmutated on generation 1
    }

    const sb = initSandbox(i + 1, canvas, parent, shouldMutate);
    sandboxes.push(sb);
  }

  compileBrainSVG();
}

// --------------------------------------------------------------------------
// Core Physics & Brain Evaluation Steps (Concurrent)
// --------------------------------------------------------------------------
function stepPhysics(sb: Sandbox) {
  if (sb.finished) return;

  sb.agent.age++;
  sb.epochTicks = (sb.epochTicks || 0) + 1;

  // 1. Gather Sensory Inputs via isolated mini SpatialGrid
  const grid = new SpatialGrid();
  
  // Register plant spores as food, and mock meat spores as a prey peer (meatball)!
  sb.foods.forEach(f => {
    if (f.type === 'plant') {
      grid.insertFood(f);
    } else {
      // Mock meat spore as a living prey peer (meatball) inside the creature layer
      const meatball: CreatureAgent = {
        id: 9999, // dummy ID
        speciesId: "MEATBALL",
        genome: "",
        antisense: "",
        phenotype: {
          latinName: "Meat Spore",
          primaryColor: { h: 360, s: 100, l: 50 }, // Red visual hue (1.0)
          secondaryColor: { h: 360, s: 100, l: 50 },
          basalMetabolicRate: 50, // Smell: 0.50
          pulseSpeed: 0.5, // Vibration: 0.50
          carnivory: 0.1, // Prey (targetHeat = 0.15)
          stiffness: 0.5,
          wavePhase: 0,
          matureAge: 0,
          dietClass: "Prey",
          organelles: [],
          spinalHarmonics: { meanRadius: 10, baseLength: 20 },
          brain: { neurons: [], synapses: [] }
        } as any,
        px: f.x,
        py: f.y,
        vx: 0,
        vy: 0,
        headingAngle: 0,
        omegaRot: 0,
        energy: 100,
        age: 0,
        generation: 1,
        adrenaline: 1.0,
        hasEaten: false,
        neuronStates: [],
        neuronActivations: []
      };
      grid.insertCreature(meatball);
    }
  });
  
  // Sensory clocks
  const clockVal = 0.5 + 0.5 * Math.sin(sb.agent.age * 0.1);
  
  // Custom sensory extraction
  const K = sb.agent.phenotype.organelles.length;
  const inputs: number[] = Array(K + 1).fill(0.0);
  inputs[K] = clockVal;

  sb.agent.phenotype.organelles.forEach((patch, idx) => {
    const range = patch.scale * 350.0;
    const alpha = (patch.angle - 90) * (Math.PI / 180);
    const halfCone = Math.max(0.1, patch.bandwidth * 1.5);
    const aff = patch.spectralAffinity;
    const organPower = patch.scale * (1.1 - patch.bandwidth);

    let maxStimulus = 0.0;

    // A. Food / Algae Scan (Herbivore sensors)
    const nearbyFood = grid.getNearbyFood(sb.agent.px, sb.agent.py, range);
    nearbyFood.forEach(pellet => {
      const dx = pellet.x - sb.agent.px;
      const dy = pellet.y - sb.agent.py;
      const dist = Math.sqrt(dx*dx + dy*dy);

      if (dist <= range) {
        let angleRel = Math.atan2(dy, dx) - sb.agent.headingAngle;
        while (angleRel > Math.PI) angleRel -= Math.PI * 2;
        while (angleRel < -Math.PI) angleRel += Math.PI * 2;
        let deltaBeta = angleRel - alpha;
        while (deltaBeta > Math.PI) deltaBeta -= Math.PI * 2;
        while (deltaBeta < -Math.PI) deltaBeta += Math.PI * 2;

        if (Math.abs(deltaBeta) <= halfCone) {
          let match = 0.0;
          if (aff >= 0.8) {
            match = Math.max(0, 1.0 - Math.abs(aff - 0.33) / (patch.bandwidth * 1.8 + 0.12));
          } else if (aff >= 0.25 && aff <= 0.65) {
            match = Math.max(0, 1.0 - Math.abs(aff - 0.15) / (patch.bandwidth * 1.8 + 0.12));
          } else if (aff < 0.25) {
            match = Math.max(0, 1.0 - Math.abs(aff - 0.05) / (patch.bandwidth * 1.8 + 0.12));
          }
          if (match > 0.05) {
            const strength = match * organPower * (1.0 - dist / range) * Math.cos(deltaBeta);
            maxStimulus = Math.max(maxStimulus, strength);
          }
        }
      }
    });

    // B. Peer / Prey Scan (Carnivore sensors - detecting our mocked Meat Spore!)
    const nearbyPeers = grid.getNearbyCreatures(sb.agent.px, sb.agent.py, range);
    nearbyPeers.forEach(other => {
      if (other.id === sb.agent.id) return;
      const dx = other.px - sb.agent.px;
      const dy = other.py - sb.agent.py;
      const dist = Math.sqrt(dx*dx + dy*dy);

      if (dist <= range) {
        let angleRel = Math.atan2(dy, dx) - sb.agent.headingAngle;
        while (angleRel > Math.PI) angleRel -= Math.PI * 2;
        while (angleRel < -Math.PI) angleRel += Math.PI * 2;
        let deltaBeta = angleRel - alpha;
        while (deltaBeta > Math.PI) deltaBeta -= Math.PI * 2;
        while (deltaBeta < -Math.PI) deltaBeta += Math.PI * 2;

        if (Math.abs(deltaBeta) <= halfCone) {
          let match = 0.0;
          
          if (aff >= 0.8) {
            // Thermal Heat Scan
            const targetHeat = (other.phenotype.carnivory >= 0.55) ? 0.85 * other.adrenaline : 0.15;
            match = Math.max(0, 1.0 - Math.abs(aff - targetHeat) / (patch.bandwidth * 1.8 + 0.12));
          } else if (aff >= 0.65 && aff < 0.8) {
            // Vibration Scan
            const targetVibration = (other.phenotype.pulseSpeed * 1000) % 1.0;
            match = Math.max(0, 1.0 - Math.abs(aff - targetVibration) / (patch.bandwidth * 1.8 + 0.12));
          } else if (aff >= 0.25 && aff < 0.65) {
            // Olfactory/Smell Scan
            const targetSmell = (other.phenotype.basalMetabolicRate % 100) / 100;
            match = Math.max(0, 1.0 - Math.abs(aff - targetSmell) / (patch.bandwidth * 1.8 + 0.12));
          } else {
            // Visual Eye Scan
            const targetVisual = other.phenotype.primaryColor.h / 360;
            match = Math.max(0, 1.0 - Math.abs(aff - targetVisual) / (patch.bandwidth * 1.8 + 0.12));
          }

          if (match > 0.05) {
            const strength = match * organPower * (1.0 - dist / range) * Math.cos(deltaBeta);
            maxStimulus = Math.max(maxStimulus, strength);
          }
        }
      }
    });

    // C. Boundary wall pressure warning
    if (aff < 0.25) {
      const wallWarningZone = range * 0.5;
      let boundaryPressure = 0.0;
      if (sb.agent.px < wallWarningZone) boundaryPressure = 1.0 - sb.agent.px / wallWarningZone;
      else if (sb.agent.px > canvasWidth - wallWarningZone) boundaryPressure = 1.0 - (canvasWidth - sb.agent.px) / wallWarningZone;
      if (sb.agent.py < wallWarningZone) boundaryPressure = Math.max(boundaryPressure, 1.0 - sb.agent.py / wallWarningZone);
      else if (sb.agent.py > canvasHeight - wallWarningZone) boundaryPressure = Math.max(boundaryPressure, 1.0 - (canvasHeight - sb.agent.py) / wallWarningZone);

      if (boundaryPressure > 0.0) {
        maxStimulus = Math.max(maxStimulus, boundaryPressure * organPower);
      }
    }

    inputs[idx] = Math.max(0.0, Math.min(1.0, maxStimulus));
  });

  // 2. Execute Brain
  const brainRes = executeBrain(sb.agent.phenotype.brain, inputs, sb.agent.neuronStates, sb.agent.neuronActivations);
  const outputs = brainRes.outputs;

  const outThrust = outputs[0];
  const outLeft = outputs[1]; // maps to Bending Left/Right

  // 4. Locomotion Kinematics
  const stiffness = sb.agent.phenotype.stiffness;
  const pulse = sb.agent.phenotype.pulseSpeed;
  const meanRadius = sb.agent.phenotype.spinalHarmonics.meanRadius;
  const baseLength = sb.agent.phenotype.spinalHarmonics.baseLength;

  let thrustMag = stiffness * (pulse * 1000 * pulse * 1000) * 6.0;
  const wavePhase = sb.agent.phenotype.wavePhase;
  const etaSwim = Math.max(0.1, Math.min(3.2, (baseLength / (meanRadius * 3.5)) * Math.sin(wavePhase) * stiffness));
  thrustMag *= etaSwim;

  const netThrustForce = outThrust * thrustMag;
  const fx = netThrustForce * Math.cos(sb.agent.headingAngle);
  const fy = netThrustForce * Math.sin(sb.agent.headingAngle);

  // Body Bending
  const maxFlexion = 1.2;
  const targetBending = outLeft * (maxFlexion / Math.max(0.2, stiffness));
  sb.agent.bendAngle = sb.agent.bendAngle || 0.0;
  sb.agent.bendAngle += (targetBending - sb.agent.bendAngle) * 0.15;
  sb.agent.bendAngle = Math.max(-maxFlexion, Math.min(maxFlexion, sb.agent.bendAngle));

  const mass = Math.pow(meanRadius, 1.5) * (baseLength / 25);
  const vForward = sb.agent.vx * Math.cos(sb.agent.headingAngle) + sb.agent.vy * Math.sin(sb.agent.headingAngle);

  // Curve turn coupling
  const deltaHeading = vForward * sb.agent.bendAngle * 0.015;
  sb.agent.headingAngle += deltaHeading;
  sb.agent.headingAngle = Math.atan2(Math.sin(sb.agent.headingAngle), Math.cos(sb.agent.headingAngle));

  sb.agent.omegaRot = sb.agent.bendAngle / 12.0;

  const receptorBallast = sb.agent.phenotype.organelles.length * 0.18;
  const dragForward = (meanRadius * 0.015 + receptorBallast) * (1.0 - stiffness * 0.3);

  const dragForceForward = -dragForward * vForward;
  const ax = (fx + dragForceForward * Math.cos(sb.agent.headingAngle)) / mass;
  const ay = (fy + dragForceForward * Math.sin(sb.agent.headingAngle)) / mass;

  sb.agent.vx = (sb.agent.vx + ax) * 0.94;
  sb.agent.vy = (sb.agent.vy + ay) * 0.94;

  // Lock-on heading movement (No slip!)
  const netSpeed = sb.agent.vx * Math.cos(sb.agent.headingAngle) + sb.agent.vy * Math.sin(sb.agent.headingAngle);
  sb.agent.vx = netSpeed * Math.cos(sb.agent.headingAngle);
  sb.agent.vy = netSpeed * Math.sin(sb.agent.headingAngle);

  // Spore Verdrängung currents
  const current = getVectoredCurrentAt(sb.world, sb.agent.px, sb.agent.py);
  sb.agent.vx += current.vx;
  sb.agent.vy += current.vy;

  sb.agent.px += sb.agent.vx;
  sb.agent.py += sb.agent.vy;

  // Track cumulative distance traveled
  const movement = Math.sqrt(sb.agent.vx ** 2 + sb.agent.vy ** 2);
  sb.distanceTraveled += movement;

  // Boundary box collisions
  const r = meanRadius;
  const wallRestitution = 0.5;
  if (sb.agent.px < r) { sb.agent.px = r; sb.agent.vx = -Math.abs(sb.agent.vx) * wallRestitution; }
  else if (sb.agent.px > canvasWidth - r) { sb.agent.px = canvasWidth - r; sb.agent.vx = -Math.abs(sb.agent.vx) * wallRestitution; }
  if (sb.agent.py < r) { sb.agent.py = r; sb.agent.vy = -Math.abs(sb.agent.vy) * wallRestitution; }
  else if (sb.agent.py > canvasHeight - r) { sb.agent.py = canvasHeight - r; sb.agent.vy = -Math.abs(sb.agent.vy) * wallRestitution; }

  // Obstacle collisions
  const col = checkObstacleCollision(sb.world, sb.agent.px, sb.agent.py, meanRadius);
  if (col.collided) {
    sb.agent.px += col.normalX * col.overlap;
    sb.agent.py += col.normalY * col.overlap;
    const dot = sb.agent.vx * col.normalX + sb.agent.vy * col.normalY;
    sb.agent.vx = (sb.agent.vx - 2.0 * dot * col.normalX) * 0.45;
    sb.agent.vy = (sb.agent.vy - 2.0 * dot * col.normalY) * 0.45;
  }

  // Spore collision & consumption check (diet-compatible)
  const targetType = sb.agent.phenotype.carnivory >= 0.35 ? 'meat' : 'plant';
  const targetFood = sb.foods.find(f => f.type === targetType)!;
  
  const foodDist = Math.sqrt((targetFood.x - sb.agent.px) ** 2 + (targetFood.y - sb.agent.py) ** 2);
  
  // Herbivore algae grazing range vs Carnivore bite range
  const eatDist = targetType === 'meat'
    ? meanRadius * 1.6 * 0.5 + 5.0 // Biting radius
    : meanRadius * 1.5 * 0.5 + 8.0; // Algae grazing range
  
  if (foodDist <= eatDist) {
    sb.finished = true;
    sb.finishTick = sb.epochTicks;
    sb.agent.hasEaten = true;
  }
}

// --------------------------------------------------------------------------
// Evolutionary Selection & Epoch Resets
// --------------------------------------------------------------------------
let isEvaluating = false;

async function evaluateGeneration(wasRunningBefore = false) {
  if (isEvaluating) return;
  isEvaluating = true;

  isRunning = false; // pause to process evaluation

  try {
    if (isMultiTrial && currentTrial < totalTrials) {
      // 1. Process intermediate trial reset
      sandboxes.forEach(sb => {
        const targetType = sb.agent.phenotype.carnivory >= 0.35 ? 'meat' : 'plant';
        const targetFood = sb.foods.find(f => f.type === targetType)!;
        const curDist = Math.sqrt((targetFood.x - sb.agent.px) ** 2 + (targetFood.y - sb.agent.py) ** 2);
        
        let trialFit = 0.0;
        if (sb.finished && sb.finishTick) {
          // Path efficiency: ratio of ideal straight-line distance to actual distance traveled
          // Weighs extremely heavily (up to 2000.0 points), while speed is minor tie-breaker (0.2x)
          const pathEfficiency = sb.startDistance / Math.max(sb.startDistance, sb.distanceTraveled);
          trialFit = 2000.0 * pathEfficiency + (epochDurationTicks - sb.finishTick) * 0.2;
        } else {
          // Unsuccessful: proximity reward only, distanceTraveled does NOT penalize here
          if (curDist < sb.startDistance) {
            trialFit = 100.0 * (1.0 - curDist / sb.startDistance);
          }
        }
        sb.accumulatedFitness = (sb.accumulatedFitness || 0.0) + trialFit;
        
        // Reset state for next trial (keep genome unchanged)
        sb.finished = false;
        sb.finishTick = undefined;
        sb.distanceTraveled = 0.0; // reset path tracker
        sb.agent.px = canvasWidth / 2;
        sb.agent.py = canvasHeight / 2;
        sb.agent.vx = 0;
        sb.agent.vy = 0;
        sb.agent.headingAngle = Math.random() * Math.PI * 2;
        
        sb.foods.forEach(spore => {
          let validSpawn = false;
          while (!validSpawn) {
            spore.x = 25 + Math.random() * (canvasWidth - 50);
            spore.y = 25 + Math.random() * (canvasHeight - 50);
            const dx = spore.x - sb.agent.px;
            const dy = spore.y - sb.agent.py;
            const dist = Math.sqrt(dx * dx + dy * dy);
            if (dist >= 80) validSpawn = true;
          }
        });
        
        const finalTargetFood = sb.foods.find(f => f.type === targetType)!;
        sb.startDistance = Math.sqrt((finalTargetFood.x - sb.agent.px) ** 2 + (finalTargetFood.y - sb.agent.py) ** 2);
      });
      
      currentTrial++;
      return; // Exit here, bypass SQLite saving and generation incrementing!
    }

    // 2. Finalize Generation (if Multi-Trial finished or disabled)
    sandboxes.forEach(sb => {
      const targetType = sb.agent.phenotype.carnivory >= 0.35 ? 'meat' : 'plant';
      const targetFood = sb.foods.find(f => f.type === targetType)!;
      const curDist = Math.sqrt((targetFood.x - sb.agent.px) ** 2 + (targetFood.y - sb.agent.py) ** 2);
      
      let trialFit = 0.0;
      if (sb.finished && sb.finishTick) {
        const pathEfficiency = sb.startDistance / Math.max(sb.startDistance, sb.distanceTraveled);
        trialFit = 2000.0 * pathEfficiency + (epochDurationTicks - sb.finishTick) * 0.2;
      } else {
        if (curDist < sb.startDistance) {
          trialFit = 100.0 * (1.0 - curDist / sb.startDistance);
        } else {
          trialFit = 0.0;
        }
      }
      
      if (isMultiTrial) {
        sb.currentFitness = ((sb.accumulatedFitness || 0.0) + trialFit) / totalTrials;
        sb.accumulatedFitness = 0.0;
      } else {
        sb.currentFitness = trialFit;
      }
      sb.distanceTraveled = 0.0; // reset path tracker for the next generation
    });

    currentTrial = 1; // reset trial counter

    // 2. Sort by Fitness
    sandboxes.sort((a, b) => b.currentFitness - a.currentFitness);

    // 3. Collect statistics
    const bestFit = sandboxes[0].currentFitness;
    const avgFit = sandboxes.reduce((acc, sb) => acc + sb.currentFitness, 0) / N;

    if (bestFit > highestFitness) {
      highestFitness = bestFit;
      statBestFit.innerText = highestFitness.toFixed(1);
      txtDna.value = sandboxes[0].agent.genome;
    }
    statAvgFit.innerText = avgFit.toFixed(1);

    // 4. Save Elite Champions to SQLite
    const eliteCount = Math.max(1, Math.ceil(N * eliteRatio));
    const elitePopulation = sandboxes.slice(0, eliteCount).map(sb => ({
      name: sb.agent.phenotype.latinName,
      genome: sb.agent.genome,
      fitness: sb.currentFitness
    }));

    await saveServerGeneration(currentGeneration, elitePopulation);

    // 5. Setup next generation
    currentGeneration++;
    statGen.innerText = (currentGeneration - 1).toString();

    // Re-read Elite from DB to guarantee absolute sync and start next run
    await rebuildSandboxGrid();
  } catch (err) {
    console.error("[Trainer Evaluation Error] ", err);
  } finally {
    isEvaluating = false;

    // Endless loop resumption
    if (wasRunningBefore) {
      isRunning = true;
      epochTicks = 0; // reset local tick counter!
      tick(); // continue loop automatically!
    } else {
      // If paused, at least draw a single static frame so the newly generated sandboxes are visible!
      sandboxes.forEach(sb => drawSandbox(sb));
    }
  }
}

// --------------------------------------------------------------------------
// Render & Drawing Routines
// --------------------------------------------------------------------------
function drawSandbox(sb: Sandbox) {
  const ctx = sb.ctx;
  ctx.fillStyle = '#020617';
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);

  // Draw World terrain
  // Biomes background cache drawing
  ctx.fillStyle = "rgba(15, 23, 42, 0.4)";
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);

  // Draw obstacles
  sb.world.obstacles.forEach(obs => {
    ctx.beginPath();
    // Translate $19200 \times 10800$ to $250 \times 250$
    const sx = (obs.x / 19200) * canvasWidth;
    const sy = (obs.y / 10800) * canvasHeight;
    const sr = (obs.radius / 19200) * canvasWidth * 1.5;
    
    ctx.arc(sx, sy, sr, 0, Math.PI * 2);
    ctx.fillStyle = "#334155";
    ctx.fill();
  });

  // Draw Spores (green for plants, red/crimson for meat)
  sb.foods.forEach(spore => {
    const isEaten = sb.finished && (sb.agent.phenotype.carnivory >= 0.35 ? spore.type === 'meat' : spore.type === 'plant');
    if (!isEaten) {
      ctx.beginPath();
      ctx.arc(spore.x, spore.y, 4, 0, Math.PI * 2);
      ctx.fillStyle = spore.type === 'meat' ? '#ef4444' : '#10b981';
      ctx.shadowBlur = 4;
      ctx.shadowColor = spore.type === 'meat' ? '#ef4444' : '#10b981';
      ctx.fill();
      ctx.shadowBlur = 0;
    }
  });

  // Draw Agent
  sb.renderer.render(
    sb.agent.phenotype,
    Date.now() * 0.05,
    sb.agent.px,
    sb.agent.py,
    sb.agent.headingAngle,
    sb.agent.omegaRot
  );

  // Draw 2.2x scale centered real-time animated preview in the diagnostics sidebar
  if (sb.id === (selectedSandboxIdx + 1) && diagCtx && diagRenderer) {
    diagCtx.fillStyle = '#020617';
    diagCtx.fillRect(0, 0, 80, 80);
    
    diagCtx.save();
    // Translate to center of 80x80 canvas
    diagCtx.translate(40, 40);
    // Scale by 2.2x to make it beautifully large
    diagCtx.scale(2.2, 2.2);
    // Render the creature at center (0,0) in translated coordinates
    diagRenderer.render(
      sb.agent.phenotype,
      Date.now() * 0.05,
      0,
      0,
      sb.agent.headingAngle,
      sb.agent.omegaRot
    );
    diagCtx.restore();
  }

  // Meta stats update
  const fitEl = document.getElementById(`card-fit-${sb.id - 1}`);
  if (fitEl) {
    fitEl.innerText = `F: ${sb.currentFitness.toFixed(1)}`;
  }
}

// --------------------------------------------------------------------------
// Brain Directed Graph SVG Live Glow Updates (Mirroring beta.ts)
// --------------------------------------------------------------------------
const brainSvgCache = new Map<string, SVGElement>();

function compileBrainSVG(): void {
  brainSvgCache.clear();
  const sb = sandboxes[selectedSandboxIdx];
  if (!sb) {
    brainContainer.innerHTML = "";
    return;
  }

  const brain = sb.agent.phenotype.brain;
  let svgContent = `<svg viewBox="0 0 320 210" style="width:100%; height:100%;">`;

  // 1. Draw Synapses
  brain.synapses.forEach((syn: any) => {
    const from = brain.neurons[syn.fromNode];
    const to = brain.neurons[syn.toNode];
    if (from && to) {
      const synId = `trainer-syn-${syn.fromNode}-${syn.toNode}`;
      const color = syn.weight > 0 ? "rgba(16, 185, 129, 0.28)" : "rgba(239, 68, 68, 0.28)";
      svgContent += `
        <line id="${synId}" x1="${(from.x || 0.1) * 320}" y1="${(from.y || 0.1) * 210}" x2="${(to.x || 0.9) * 320}" y2="${(to.y || 0.9) * 210}" 
              stroke="${color}" stroke-width="${Math.max(0.5, Math.abs(syn.weight) * 1.5)}" />
      `;
    }
  });

  // 2. Draw Neuron Nodes
  brain.neurons.forEach((n: any) => {
    const nodeId = `trainer-node-${n.id}`;
    const isInput = n.type === "input";
    const isOutput = n.type === "output";
    const color = isInput ? "var(--primary-cyan)" : (isOutput ? "var(--accent-purple)" : "var(--text-muted)");

    svgContent += `
      <circle id="${nodeId}" cx="${(n.x || 0.5) * 320}" cy="${(n.y || 0.5) * 210}" r="${isInput || isOutput ? 4.5 : 3.2}" 
              fill="#111827" stroke="${color}" stroke-width="1.5" />
    `;
  });

  svgContent += `</svg>`;
  brainContainer.innerHTML = svgContent;

  // Cache element references
  brain.neurons.forEach((n: any) => {
    const id = `trainer-node-${n.id}`;
    const el = document.getElementById(id) as any;
    if (el) {
      brainSvgCache.set(id, el);
      
      // Bind hover events to track selected neuron for live telemetry tooltips
      el.addEventListener("mouseenter", () => {
        hoveredNeuronId = n.id;
      });
      el.addEventListener("mouseleave", () => {
        hoveredNeuronId = null;
        if (neuronMeta) {
          neuronMeta.innerHTML = `Hover a neuron node to see live telemetry...`;
        }
      });
    }
  });

  brain.synapses.forEach((syn: any) => {
    const id = `trainer-syn-${syn.fromNode}-${syn.toNode}`;
    const el = document.getElementById(id) as any;
    if (el) brainSvgCache.set(id, el);
  });
}

function updateBrainLiveGlows(): void {
  const sb = sandboxes[selectedSandboxIdx];
  if (!sb) return;

  const brain = sb.agent.phenotype.brain;
  const activations = sb.agent.neuronActivations;
  if (!brain || !activations) return;

  // 1. Update Nodes
  brain.neurons.forEach((n: any) => {
    const id = `trainer-node-${n.id}`;
    const el = brainSvgCache.get(id);
    if (el) {
      const rawAct = Math.max(0.0, Math.min(1.0, Math.abs(activations[n.id] || 0.0)));
      const isInput = n.type === "input";
      const isOutput = n.type === "output";

      // Soften the exponent for input neurons so they glow nicely upon detecting stimulus!
      const exponent = isInput ? 1.5 : 4.0;
      const act = Math.pow(rawAct, exponent);

      const colorGlow = isInput ? "#00f2fe" : (isOutput ? "#c084fc" : "#e2e8f0");

      const fill = act > 0.35 ? colorGlow : "#111827";
      const radius = isInput || isOutput ? (act > 0.45 ? 6.5 : 4.5) : (act > 0.45 ? 5.0 : 3.2);

      el.setAttribute("fill", fill);
      el.setAttribute("r", radius.toString());
    }
  });

  // 2. Update Synapses
  brain.synapses.forEach((syn: any) => {
    const id = `trainer-syn-${syn.fromNode}-${syn.toNode}`;
    const el = brainSvgCache.get(id);
    if (el) {
      const preVal = Math.max(0.0, Math.min(1.0, Math.abs(activations[syn.fromNode] || 0.0)));
      const act = Math.pow(preVal, 4.0);
      const isExcitatory = syn.weight > 0;
      const baseColor = isExcitatory ? "16, 185, 129" : "239, 68, 68";
      const opacity = act > 0.35 ? 0.95 : 0.28;
      el.setAttribute("stroke", `rgba(${baseColor}, ${opacity})`);
    }
  });

  // Update text metadata
  if (focusGenome) {
    focusGenome.value = sb.agent.genome;
  }

  const targetType = sb.agent.phenotype.carnivory >= 0.35 ? 'meat' : 'plant';
  const seedStr = `SANDBOX_SEED_${sb.id}_GEN_${currentGeneration}`;
  focusMeta.innerHTML = `
    Sandbox: #${sb.id}<br/>
    Status: ${sb.finished ? "🏁 SUCCESS" : "🏃 TRAINING"}<br/>
    Fitness: ${sb.currentFitness.toFixed(1)}<br/>
    Diet: ${sb.agent.phenotype.dietClass} (${targetType})<br/>
    Seed: <span style="color: var(--primary-cyan); font-size: 0.58rem; word-break: break-all;">${seedStr}</span>
  `;

  // 3. Dynamically update hovered neuron floating tooltip with live potentials and adaptive formulas
  if (hoveredNeuronId !== null) {
    const K = sb.agent.phenotype.organelles.length;
    const isInput = hoveredNeuronId <= K;
    const isOutput = hoveredNeuronId >= K + 1 && hoveredNeuronId <= K + 4;
    
    let baseDesc = `Neuron #${hoveredNeuronId}`;
    let mathFormula = "";
    let liveValues = "";

    if (isInput) {
      mathFormula = "f(x) = Identity (Bounded [0, 1])";
      const act = sb.agent.neuronActivations[hoveredNeuronId] || 0.0;
      liveValues = `<b>Activation (a):</b> ${act.toFixed(3)}`;
      
      if (hoveredNeuronId === K) {
        baseDesc = `Input #${hoveredNeuronId}: Internal Clock`;
      } else {
        const patch = sb.agent.phenotype.organelles[hoveredNeuronId];
        if (patch) {
          const aff = patch.spectralAffinity;
          let sensorName = "Visual Eye";
          if (aff >= 0.8) sensorName = "Thermal (Heat)";
          else if (aff >= 0.65) sensorName = "Vibration";
          else if (aff >= 0.25) sensorName = "Olfactory (Smell)";
          baseDesc = `Input #${hoveredNeuronId}: Organelle #${hoveredNeuronId + 1} (${sensorName})`;
        }
      }
    } else {
      const neuron = sb.agent.phenotype.brain.neurons[hoveredNeuronId];
      const state = sb.agent.neuronStates[hoveredNeuronId] || 0.0;
      const act = sb.agent.neuronActivations[hoveredNeuronId] || 0.0;
      
      // Determine genetically encoded activation function
      const actType = (neuron && neuron.activationType) || "tanh";
      if (actType === "relu") {
        mathFormula = "f(s) = max(0, s) [ReLU]";
      } else if (actType === "sigmoid") {
        mathFormula = "f(s) = 1 / (1 + e^-s) [Sigmoid]";
      } else if (actType === "sin") {
        mathFormula = "f(s) = sin(s) [-1.0 to 1.0] [Oscillatory]";
      } else {
        mathFormula = "f(s) = tanh(s) [-1.0 to 1.0] [Hyperbolic]";
      }

      liveValues = `<b>Potential (s):</b> ${state.toFixed(3)}<br/><b>Activation (a):</b> ${act.toFixed(3)}`;
      
      if (isOutput) {
        const outputIndex = hoveredNeuronId - (K + 1);
        if (outputIndex === 0) {
          baseDesc = `Output #${hoveredNeuronId}: Thrust`;
        } else if (outputIndex === 1) {
          baseDesc = `Output #${hoveredNeuronId}: Flexion Steering`;
        } else if (outputIndex === 2) {
          baseDesc = `Output #${hoveredNeuronId}: Biolum Flash`;
        } else {
          baseDesc = `Output #${hoveredNeuronId}: Reserved Motor`;
        }
      } else {
        baseDesc = `Neuron #${hoveredNeuronId} (Interneuron #${hoveredNeuronId - K - 4})`;
      }
      
      if (neuron) {
        liveValues += `<br/><b>Decay (tau):</b> ${neuron.tau.toFixed(1)}f | <b>Bias:</b> ${neuron.bias.toFixed(2)}`;
      }
    }

    if (neuronMeta) {
      neuronMeta.innerHTML = `
        <span style="color: #00f2fe; font-weight: bold;">${baseDesc}</span><br/>
        <span style="color: var(--text-muted); font-size: 0.53rem;">Formula: ${mathFormula}</span><br/>
        ${liveValues}
      `;
    }
  }
}

// --------------------------------------------------------------------------
// Core Trainer Main Loops & Key Listeners
// --------------------------------------------------------------------------
function tick() {
  if (!isRunning) return;

  for (let step = 0; step < warpSpeed; step++) {
    epochTicks++;
    
    // Step all sandbox physical loops
    sandboxes.forEach(sb => stepPhysics(sb));

    // Live sensory update of diagnostics
    if (step === 0) {
      updateBrainLiveGlows();
    }

    if (epochTicks >= epochDurationTicks) {
      const wasRunning = isRunning;
      isRunning = false;
      evaluateGeneration(wasRunning);
      return;
    }
  }

  // Render frames at 60Hz
  sandboxes.forEach(sb => drawSandbox(sb));
  const timeStr = ((epochDurationTicks - epochTicks) / 60).toFixed(1) + "s";
  statTimer.innerText = isMultiTrial ? `T${currentTrial}: ${timeStr}` : timeStr;

  requestAnimationFrame(tick);
}

// --------------------------------------------------------------------------
// UI Interactivity Bindings
// --------------------------------------------------------------------------
btnStart.addEventListener("click", () => {
  if (!isRunning) {
    isRunning = true;
    btnStart.innerText = "Pause Training";
    btnStart.classList.add("btn-danger");
    btnStart.classList.remove("btn-primary");
    
    // Resume tick loop
    tick();
  } else {
    isRunning = false;
    btnStart.innerText = "Start Training";
    btnStart.classList.add("btn-primary");
    btnStart.classList.remove("btn-danger");
  }
});

btnReset.addEventListener("click", async () => {
  const confirmReset = confirm("Are you sure you want to completely wipe the evolutionary trainer database and start from Gen 1?");
  if (confirmReset) {
    isRunning = false;
    btnStart.innerText = "Start Training";
    btnStart.classList.add("btn-primary");
    btnStart.classList.remove("btn-danger");
    
    currentGeneration = 1;
    highestFitness = 0.0;
    epochTicks = 0;
    statGen.innerText = "1";
    statBestFit.innerText = "0.0";
    statAvgFit.innerText = "0.0";
    statTimer.innerText = "5.0s";
    txtDna.value = "";

    await resetServerTrainer();
    rebuildSandboxGrid();
  }
});

const trainingListContainer = document.getElementById("training-list-container") as HTMLDivElement;
const dropdownTrigger = document.getElementById("training-dropdown-trigger") as HTMLButtonElement;
const dropdownCurrentLabel = document.getElementById("training-dropdown-current") as HTMLSpanElement;
const txtNewTraining = document.getElementById("txt-new-training") as HTMLInputElement;
const btnCreateTraining = document.getElementById("btn-create-training") as HTMLButtonElement;

async function populateRunSelector() {
  const runs = await fetchServerRuns();
  if (trainingListContainer) {
    trainingListContainer.innerHTML = "";
  }
  
  let defaultExists = false;
  let activeExistsInDb = false;

  const listItems: { id: string; label: string; isNew?: boolean }[] = [];

  runs.forEach(run => {
    if (run.run_id === "default_run") defaultExists = true;
    if (run.run_id === runId) activeExistsInDb = true;

    listItems.push({
      id: run.run_id,
      label: `${run.run_id} (Gen ${run.max_gen}, F: ${run.max_fit.toFixed(0)})`
    });
  });

  if (!activeExistsInDb && runId !== "default_run") {
    listItems.push({
      id: runId,
      label: `${runId} (New)`,
      isNew: true
    });
  }

  if (!defaultExists) {
    listItems.unshift({
      id: "default_run",
      label: "default_run (New)"
    });
  }

  // Update trigger button label to reflect active selection
  const activeItem = listItems.find(item => item.id === runId);
  if (dropdownCurrentLabel && activeItem) {
    dropdownCurrentLabel.innerText = activeItem.label;
  }

  // Render each item as a clickable div with a delete button
  listItems.forEach(item => {
    const row = document.createElement("div");
    row.className = `training-item ${item.id === runId ? "active" : ""}`;
    
    // Label span (selects session on click)
    const labelSpan = document.createElement("span");
    labelSpan.innerText = item.label;
    labelSpan.style.flex = "1";
    labelSpan.style.overflow = "hidden";
    labelSpan.style.textOverflow = "ellipsis";
    labelSpan.style.whiteSpace = "nowrap";
    labelSpan.addEventListener("click", () => {
      // Close dropdown
      if (trainingListContainer) {
        trainingListContainer.style.display = "none";
      }

      if (item.id === runId) return;
      runId = item.id;
      isRunning = false;
      btnStart.innerText = "Start Training";
      btnStart.classList.add("btn-primary");
      btnStart.classList.remove("btn-danger");
      epochTicks = 0;
      currentTrial = 1;
      statTimer.innerText = "5.0s";
      rebuildSandboxGrid().then(() => {
        populateRunSelector();
      });
    });
    row.appendChild(labelSpan);

    // Delete "✕" button (only show for sessions other than default_run)
    if (item.id !== "default_run") {
      const deleteSpan = document.createElement("span");
      deleteSpan.className = "training-delete-btn";
      deleteSpan.innerText = "✕";
      deleteSpan.title = "Delete this training session permanently";
      deleteSpan.addEventListener("click", async (e) => {
        e.stopPropagation(); // prevent selecting the row!
        const confirmDelete = confirm(`Are you sure you want to permanently DELETE the training session '${item.id}'? This will completely wipe all its generation records and cannot be undone!`);
        if (confirmDelete) {
          const wasActive = item.id === runId;
          
          if (wasActive) {
            isRunning = false;
            btnStart.innerText = "Start Training";
            btnStart.classList.add("btn-primary");
            btnStart.classList.remove("btn-danger");
            currentGeneration = 1;
            highestFitness = 0.0;
            epochTicks = 0;
            currentTrial = 1;
            statGen.innerText = "1";
            statBestFit.innerText = "0.0";
            statAvgFit.innerText = "0.0";
            statTimer.innerText = "5.0s";
            txtDna.value = "";
          }

          // SQLite delete call via our reset API
          try {
            await fetch(`${BACKEND_BASE}/api/trainer/reset`, {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ runId: item.id })
            });
          } catch (err) {
            console.error("[Trainer API] Error deleting training:", err);
          }

          if (wasActive) {
            runId = "default_run";
          }

          await rebuildSandboxGrid();
          await populateRunSelector();
        }
      });
      row.appendChild(deleteSpan);
    }

    if (trainingListContainer) {
      trainingListContainer.appendChild(row);
    }
  });
}

// Toggle dropdown visibility on trigger click
if (dropdownTrigger) {
  dropdownTrigger.addEventListener("click", (e) => {
    e.stopPropagation();
    if (trainingListContainer) {
      const isOpen = trainingListContainer.style.display === "block";
      trainingListContainer.style.display = isOpen ? "none" : "block";
    }
  });
}

// Close dropdown on clicking outside
document.addEventListener("click", () => {
  if (trainingListContainer) {
    trainingListContainer.style.display = "none";
  }
});

btnCreateTraining.addEventListener("click", () => {
  const name = txtNewTraining.value.trim().replace(/[^a-zA-Z0-9_]/g, "");
  if (!name) {
    alert("Please enter a valid alphanumeric training name!");
    return;
  }
  
  runId = name;
  txtNewTraining.value = "";
  
  isRunning = false;
  btnStart.innerText = "Start Training";
  btnStart.classList.add("btn-primary");
  btnStart.classList.remove("btn-danger");
  epochTicks = 0;
  statTimer.innerText = "5.0s";
  
  rebuildSandboxGrid().then(() => {
    populateRunSelector();
  });
});

btnCopyDna.addEventListener("click", () => {
  if (!txtDna.value) return;
  navigator.clipboard.writeText(txtDna.value);
  applyServerChampion(txtDna.value);
});

// Real-time slider update hooks
sliderGridSize.addEventListener("input", () => {
  N = parseInt(sliderGridSize.value);
  lblGridSize.innerText = N.toString();
  rebuildSandboxGrid();
});

sliderSpeedup.addEventListener("input", () => {
  warpSpeed = parseInt(sliderSpeedup.value);
  lblSpeedup.innerText = warpSpeed + "x";
});

sliderEliteRatio.addEventListener("input", () => {
  eliteRatio = parseInt(sliderEliteRatio.value) / 100;
  lblEliteRatio.innerText = sliderEliteRatio.value + "%";
});

sliderMutation.addEventListener("input", () => {
  genomeMutationRate = parseInt(sliderMutation.value) / 100;
  lblMutation.innerText = sliderMutation.value + "%";
});

sliderInflow.addEventListener("input", () => {
  randomInflowRate = parseInt(sliderInflow.value) / 100;
  lblInflow.innerText = sliderInflow.value + "%";
});

sliderHof.addEventListener("input", () => {
  randomHofRate = parseInt(sliderHof.value) / 100;
  lblHof.innerText = sliderHof.value + "%";
});

chkMultiTrial.addEventListener("change", () => {
  isMultiTrial = chkMultiTrial.checked;
  currentTrial = 1;
});

// Startup Boot: Pause background substrate simulation & load grid
toggleBackgroundSimulation(false);
rebuildSandboxGrid().then(() => {
  populateRunSelector();
});

// Restore background simulation on page close
window.addEventListener("unload", () => {
  toggleBackgroundSimulation(true);
});
