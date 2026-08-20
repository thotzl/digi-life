import { WebSocketServer, WebSocket } from "ws";
import http from "http";
import fs from "fs";
import path from "path";
import { parseGenome, generateRandomGenome, getComplementaryString, executeBrain, mutateGenome } from "../biology/dna";
import { CreatureAgent, FoodSpore, SpeciesRecord } from "../shared/types";

import { readDb, writeDb, readState, writeState, clearState, saveTrainerGeneration, getTrainerPopulation, getTrainerHallOfFame, getAllTrainingsChampions, clearTrainerHistory, getTrainerRuns } from "./db";
import { generateWorld, getVectoredCurrentAt, getBiomeAt, checkObstacleCollision } from "../shared/mapGenerator";
import { SpatialGrid } from "./spatialGrid";

// Read central configuration
const CONFIG_PATH = path.resolve("./config.json");
const config = JSON.parse(fs.readFileSync(CONFIG_PATH, "utf8"));
const rules = config.rules || {};
const bmrMultiplier = config.basalMetabolicRateMultiplier || 1.0;
const world = generateWorld(config.seed || "ALIFE_BASIN_77A", 19200, 10800, config.rules);

// --------------------------------------------------------------------------
// Headless Simulation State
// --------------------------------------------------------------------------
let creatures: CreatureAgent[] = [];
let foodPellets: FoodSpore[] = [];
let biteImpacts: { x: number; y: number; age: number }[] = [];
let nextAgentId = 1;
let highestGeneration = 1;
let cachedAliveSpecies: SpeciesRecord[] = [];
let selectedAgentId: number | null = null;

// Tank Dimensions matching full-screen client boundaries (upscaled 10x!)
const logicalWidth = 19200;
const logicalHeight = 10800;
const frameScale = 1.0; // static simulation step scale (dt)

// WebSocket clients registry
const connectedClients = new Set<WebSocket>();

// --------------------------------------------------------------------------
// Core Simulation Initialization
// --------------------------------------------------------------------------
function registerSpeciesIfNew(genome: string, parentSpeciesId: string | null, generation: number) {
  const db = readDb();
  const recordExist = db.find(r => r.id === genome);
  if (!recordExist) {
    const anti = getComplementaryString(genome);
    const phenotype = parseGenome(genome, anti);
    db.push({
      id: genome,
      name: phenotype.latinName,
      genome: genome,
      antisense: anti,
      parentSpeciesId: parentSpeciesId,
      status: "alive",
      peakPopulation: 1,
      birthTime: Date.now(),
      generation: generation,
      carnivory: phenotype.carnivory
    });
    writeDb(db);
    cachedAliveSpecies = db.filter(rec => rec.status === "alive");
    
    // Broadcast database change event so the clients reload roster
    broadcast({ type: "DATABASE_CHANGED" });
  }
}

function spawnSporeBiologically(pellet: FoodSpore) {
  // A. Determine spawning mode:
  // 15% chance to erupt from a thermal vent core as a geothermal spore
  // 85% chance to spawn in an ecologically suitable biome via rejection sampling
  const modeRoll = Math.random();

  if (modeRoll < 0.15 && world.vents.length > 0) {
    const ventIdx = Math.floor(Math.random() * world.vents.length);
    const vent = world.vents[ventIdx];
    
    // Spawn at vent center with a slight random offset
    pellet.x = vent.x + (Math.random() * 40 - 20);
    pellet.y = vent.y + (Math.random() * 40 - 20);
    
    // Spew outwards with high kinetic speed
    const spewAngle = Math.random() * Math.PI * 2;
    const spewSpeed = 1.5 + Math.random() * 2.5;
    pellet.vx = Math.cos(spewAngle) * spewSpeed;
    pellet.vy = Math.sin(spewAngle) * spewSpeed;
  } 
  else {
    let accepted = false;
    let attempts = 0;
    while (!accepted && attempts < 25) {
      attempts++;
      const tx = Math.random() * logicalWidth;
      const ty = Math.random() * logicalHeight;

      const biome = getBiomeAt(world, tx, ty);
      if (biome) {
        // High spawnRate increases likelihood of acceptance
        const acceptChance = biome.sporeSpawnRate / 1.4; // Normalized by max rate
        if (Math.random() < acceptChance) {
          pellet.x = tx;
          pellet.y = ty;
          pellet.vx = (Math.random() * 0.3 - 0.15);
          pellet.vy = (Math.random() * 0.3 - 0.15);
          accepted = true;
        }
      }
    }
    // Fallback if sampling takes too long (safeguard)
    if (!accepted) {
      pellet.x = Math.random() * logicalWidth;
      pellet.y = Math.random() * logicalHeight;
      pellet.vx = (Math.random() * 0.3 - 0.15);
      pellet.vy = (Math.random() * 0.3 - 0.15);
    }
  }
}

function initSimulation() {
  const state = readState();
  const db = readDb();
  cachedAliveSpecies = db.filter(rec => rec.status === "alive");

  if (state && state.creatures && state.creatures.length > 0) {
    // Restore persistent session
    creatures = state.creatures.map((c: any) => ({
      id: c.id,
      speciesId: c.speciesId,
      genome: c.genome,
      antisense: c.antisense,
      phenotype: parseGenome(c.genome, c.antisense, c.phenotype?.methylations || []),
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
      neuronStates: c.neuronStates || [],
      neuronActivations: c.neuronActivations || []
    }));
    foodPellets = state.foodPellets || [];
    nextAgentId = state.nextAgentId || 1;
    highestGeneration = state.highestGeneration || 1;
    console.log(`[SERVER] Simulation restored. creaturesCount=${creatures.length}, foodCount=${foodPellets.length}`);
  } else {
    // Fallback: spawn 15 fresh founder cells (Founders)
    creatures = [];
    foodPellets = [];
    nextAgentId = 1;
    highestGeneration = 1;

    // Spores layout (proportionally upscaled to support 10x larger field!)
    const sporeCount = config.foodSporeCount || 300;
    for (let i = 0; i < sporeCount; i++) {
      const pellet: FoodSpore = { x: 0, y: 0, vx: 0, vy: 0 };
      spawnSporeBiologically(pellet);
      foodPellets.push(pellet);
    }

    // Spawn initial creatures from alive database, or random genomes!
    const numToSpawn = 15;
    for (let k = 0; k < numToSpawn; k++) {
      let g = generateRandomGenome(256);
      let gen = 1;

      if (cachedAliveSpecies.length > 0) {
        const idx = Math.floor(Math.random() * cachedAliveSpecies.length);
        const record = cachedAliveSpecies[idx];
        g = record.genome;
        gen = record.generation;
      }

      const anti = getComplementaryString(g);
      const pheno = parseGenome(g, anti);

      registerSpeciesIfNew(g, null, gen);

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
        energy: pheno.stomachCapacity * 0.6,
        age: 0,
        generation: gen,
        adrenaline: 1.0,
        hasEaten: false,
        neuronStates: [],
        neuronActivations: []
      });
    }
    console.log(`[SERVER] Fresh Founders initialized. creaturesCount=15`);
    saveSimulationStateOnDisk();
  }
}

// --------------------------------------------------------------------------
// State Persistence Broadcaster
// --------------------------------------------------------------------------
function saveSimulationStateOnDisk() {
  const payload = {
    creatures: creatures.map(c => ({
      id: c.id,
      speciesId: c.speciesId,
      genome: c.genome,
      antisense: c.antisense,
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
    foodPellets,
    nextAgentId,
    highestGeneration
  };
  writeState(payload);
}

// --------------------------------------------------------------------------
// WebSocket Event Dispatchers
// --------------------------------------------------------------------------
function broadcast(event: any) {
  const msg = JSON.stringify(event);
  connectedClients.forEach(client => {
    if (client.readyState === WebSocket.OPEN) {
      client.send(msg);
    }
  });
}

function sendToClient(ws: WebSocket, event: any) {
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(event));
  }
}

// --------------------------------------------------------------------------
// Headless 60hz Simulation Tick
// --------------------------------------------------------------------------
let tickCount = 0;
let isSimulationRunning = true;

function simulationTick() {
  if (!isSimulationRunning) return;
  tickCount++;

  // A. Move and drift food spores (Hard boundaries & Vector current fields)
  foodPellets.forEach(pellet => {
    const current = getVectoredCurrentAt(world, pellet.x, pellet.y);
    pellet.vx += current.vx * frameScale;
    pellet.vy += current.vy * frameScale;

    // Fluid friction drag on spores
    pellet.vx *= 0.95;
    pellet.vy *= 0.95;

    pellet.x += pellet.vx * frameScale;
    pellet.y += pellet.vy * frameScale;

    // Collide with hard boundaries (elastic bounce)
    const pelletRadius = 8;
    const pelletRestitution = 0.5;
    if (pellet.x < pelletRadius) {
      pellet.x = pelletRadius;
      pellet.vx = -Math.abs(pellet.vx) * pelletRestitution;
    } else if (pellet.x > logicalWidth - pelletRadius) {
      pellet.x = logicalWidth - pelletRadius;
      pellet.vx = -Math.abs(pellet.vx) * pelletRestitution;
    }
    if (pellet.y < pelletRadius) {
      pellet.y = pelletRadius;
      pellet.vy = -Math.abs(pellet.vy) * pelletRestitution;
    } else if (pellet.y > logicalHeight - pelletRadius) {
      pellet.y = logicalHeight - pelletRadius;
      pellet.vy = -Math.abs(pellet.vy) * pelletRestitution;
    }

    // Collide with solid circular obstacles deterministically
    const collision = checkObstacleCollision(world, pellet.x, pellet.y, 8);
    if (collision.collided) {
      pellet.x += collision.normalX * collision.overlap;
      pellet.y += collision.normalY * collision.overlap;
      
      const dot = pellet.vx * collision.normalX + pellet.vy * collision.normalY;
      pellet.vx = (pellet.vx - 2.0 * dot * collision.normalX) * 0.5;
      pellet.vy = (pellet.vy - 2.0 * dot * collision.normalY) * 0.5;
    }
  });

  // C. Vegetative Spore Mitosis multiplication (Meadows cloning)
  const maxSporeCeiling = (config.foodSporeCount || 300) * 1.5;
  if (foodPellets.length < maxSporeCeiling && Math.random() < 0.05) {
    const idx = Math.floor(Math.random() * foodPellets.length);
    const parentSpore = foodPellets[idx];
    if (parentSpore) {
      const biome = getBiomeAt(world, parentSpore.x, parentSpore.y);
      if (biome && biome.sporeSpawnRate >= 1.0) {
        if (Math.random() < 0.04) {
          const spawnX = parentSpore.x + (Math.random() * 120 - 60);
          const spawnY = parentSpore.y + (Math.random() * 120 - 60);
          foodPellets.push({
            x: Math.max(8, Math.min(logicalWidth - 8, spawnX)),
            y: Math.max(8, Math.min(logicalHeight - 8, spawnY)),
            vx: (Math.random() * 0.2 - 0.1),
            vy: (Math.random() * 0.2 - 0.1)
          });
        }
      }
    }
  }

  // Populate SpatialGrid at the start of the frame (before creature loop)
  const grid = new SpatialGrid();
  foodPellets.forEach(pellet => grid.insertFood(pellet));
  creatures.forEach(agent => grid.insertCreature(agent));

  // Decay bite impacts
  biteImpacts = biteImpacts.map(impact => ({ ...impact, age: impact.age + 1 })).filter(impact => impact.age < 12);

  const deadAgentIds: number[] = [];
  const newbornAgents: CreatureAgent[] = [];
  const currentAliveSpeciesThisFrame = new Set<string>();

  // B. Physics & biology loop over all creatures
  creatures.forEach(agent => {
    agent.age += 1;
    currentAliveSpeciesThisFrame.add(agent.speciesId);

    if (agent.adrenaline === undefined) agent.adrenaline = 1.0;

    // 1. Photosynthesis for green creatures in light zone
    const hue = agent.phenotype.primaryColor.h;
    const isGreenPrey = (agent.phenotype.carnivory < (rules.bitingCarnivoryThreshold || 0.35)) && (hue >= 75 && hue <= 175);
    const inLightZone = agent.py < logicalHeight * 0.35;
    if (isGreenPrey && inLightZone) {
      agent.energy = Math.min(agent.phenotype.stomachCapacity, agent.energy + (rules.photosynthesisEnergyGain || 0.15));
    }

    // 2. Spatial Temperature Stratification and Thermal Limits stress penalty
    const localTemp = 38.0 - (agent.py / logicalHeight) * 26.0;
    const tempMin = agent.phenotype.thermalToleranceMin;
    const tempMax = agent.phenotype.thermalToleranceMax;
    const thermalStress = localTemp < tempMin ? (tempMin - localTemp) : (localTemp > tempMax ? (localTemp - tempMax) : 0.0);
    
    if (thermalStress > 0.1) {
      agent.energy -= thermalStress * (rules.thermalStressPenaltyScale || 0.0012) * frameScale;
    }

    // 3. Adrenaline sprint tax
    const metabolicSurcharge = 1.0 + (agent.adrenaline - 1.0) * (rules.adrenalineMetabolicSurchargeScale || 1.5);
    agent.energy -= agent.phenotype.basalMetabolicRate * (rules.bmrBaseScale || 0.005) * metabolicSurcharge * bmrMultiplier;

    // Starvation / Senescence death
    if (agent.energy <= 0 || agent.age >= (rules.creatureMaxAgeTicks || 2700)) {
      deadAgentIds.push(agent.id);

      // Decompose corpse into food pellets
      const L_dead = agent.phenotype.spinalHarmonics.baseLength;
      const r_dead = agent.phenotype.spinalHarmonics.meanRadius;
      const numPellets = Math.max(
        rules.decompositionSporeMin || 1,
        Math.min(rules.decompositionSporeMax || 5, Math.floor((L_dead * r_dead) / (rules.decompositionSizeRatio || 1200)))
      );

      for (let p = 0; p < numPellets; p++) {
        const pIdx = Math.floor(Math.random() * foodPellets.length);
        if (foodPellets[pIdx]) {
          foodPellets[pIdx].x = agent.px + Math.random() * 32 - 16;
          foodPellets[pIdx].y = agent.py + Math.random() * 32 - 16;
          foodPellets[pIdx].vx = (Math.random() * 0.4 - 0.2);
          foodPellets[pIdx].vy = (Math.random() * 0.4 - 0.2);
        }
      }

      broadcast({
        type: "LOG_EVENT",
        message: `Corpse Decomposition! Species #${agent.id} ${agent.energy <= 0 ? "starved" : "died of old age"}. ${numPellets} nutrient spores released.`,
        logType: "mutation"
      });
      return;
    }

    // 4. CTRNN Neural Controller Execution
    const meanRadius = agent.phenotype.spinalHarmonics.meanRadius;
    const baseLength = agent.phenotype.spinalHarmonics.baseLength;
    const stiffness = agent.phenotype.stiffness;
    const pulse = agent.phenotype.pulseSpeed;

    let thrustMag = stiffness * (pulse * 1000 * pulse * 1000) * (rules.thrustBaseMultiplier || 6.0);
    const wavePhase = agent.phenotype.wavePhase;
    const etaSwim = Math.max(0.1, Math.min(3.2, (baseLength / (meanRadius * 3.5)) * Math.max(0.01, Math.sin(wavePhase)) * stiffness));
    thrustMag *= etaSwim;

    const limbsCount = agent.phenotype.organelles.filter(n => n.expressionStyle >= 0.72).length;
    thrustMag *= (1.0 + limbsCount * 0.12);
    thrustMag *= (1.0 + agent.phenotype.spinalHarmonics.parapodiaAmp * 1.0);

    // Collect Senses
    const clockVal = 0.5 + 0.5 * Math.sin(Date.now() * 0.0012 + agent.id);
    const inputs = computeHeadlessSensoryInputs(agent, clockVal, grid);

    const brainRes = executeBrain(agent.phenotype.brain, inputs, agent.neuronStates, agent.neuronActivations);
    const outputs = brainRes.outputs;

    const outThrust = outputs[0];
    const outLeft = outputs[1];
    const outFlash = outputs[3];

    if (outFlash > 0.5) {
      agent.energy -= (rules.bioluminescenceFlashCost || 0.05) * outFlash;
      broadcast({
        type: "FLASH_EVENT",
        agentId: agent.id,
        x: agent.px,
        y: agent.py,
        intensity: outFlash
      });
    }

    // threat perception triggers endocrine adrenaline
    let threatPerception = 0.0;
    agent.phenotype.organelles.forEach((patch, idx) => {
      const sensesDanger = (patch.spectralAffinity < 0.25) || (patch.spectralAffinity > 0.65 && patch.spectralAffinity < 0.8);
      if (sensesDanger) {
        threatPerception = Math.max(threatPerception, inputs[idx]);
      }
    });

    if (threatPerception > 0.1) {
      agent.adrenaline = Math.min(rules.adrenalineMaxMultiplier || 1.8, agent.adrenaline + (rules.adrenalineIncreaseRate || 0.06) * threatPerception);
    } else {
      agent.adrenaline = Math.max(1.0, agent.adrenaline - (rules.adrenalineDecayRate || 0.015));
    }

    // Hebbian Recurrent Synaptic Learning
    const learningRate = (rules.hebbianLearningRateBase || 0.00015) * (1.0 - stiffness * (rules.hebbianLearningStiffnessDecay || 0.85));
    const forgettingDecay = rules.hebbianForgettingDecay || 0.0000032; // exactly 1 hour half-life
    const b = agent.phenotype.brain;

    b.synapses.forEach((syn: any) => {
      const preVal = agent.neuronActivations[syn.fromNode];
      const postVal = Math.max(0.0, agent.neuronActivations[syn.toNode]);
      
      let weight = syn.weight;
      weight += learningRate * (preVal * postVal) - forgettingDecay * weight;
      weight = Math.max(-2.5, Math.min(2.5, weight));
      syn.weight = weight;
    });

    const predatorSavageMultiplier = agent.phenotype.carnivory >= (rules.predatorSavageThrustThreshold || 0.55) ? (rules.predatorSavageThrustMultiplier || 1.45) : 1.0;
    const netThrustForce = outThrust * thrustMag * predatorSavageMultiplier * agent.adrenaline;

    const mass = Math.pow(meanRadius, 1.5) * (baseLength / 25);

    // Calculate custom biological drag friction based on rules config
    const receptorBallast = agent.phenotype.organelles.length * (rules.receptorBallastScale || 0.18);
    const dragForward = (meanRadius * (rules.dragForwardCoefficient || 0.015) + receptorBallast) * (1.0 - stiffness * (rules.dragForwardStiffnessDecay || 0.3));

    // Calculate external environmental current forces
    const current = getVectoredCurrentAt(world, agent.px, agent.py);

    // Apply unified, isomorphic physics kinematics, lateral currents drift, and wall/obstacle collisions SSOT!
    applyCreaturePhysics(agent, netThrustForce, outLeft, mass, dragForward, current.vx, current.vy, logicalWidth, logicalHeight, (px: number, py: number, r: number) => checkObstacleCollision(world, px, py, r));

    // Apply Biome-based Hazard damage and metabolic tax
    const currentBiome = getBiomeAt(world, agent.px, agent.py);
    if (currentBiome && currentBiome.hazardDamage > 0) {
      agent.energy = Math.max(0.0, agent.energy - currentBiome.hazardDamage * bmrMultiplier * (rules.biomeHazardDamageScale || 1.0) * frameScale);
    }

    // 5. Unified Spore Grazing and Combat Biting collisions!
    // A. Graze Spores (optimized via SpatialGrid)
    // Ensure that eating range is always slightly larger than the body push boundary (meanRadius + 8)
    // so that creatures can actually reach and eat food pellets instead of pushing them away forever!
    const baseEatRadius = meanRadius * (rules.grazingRadiusMultiplier || 1.5) * 0.5 + (rules.grazingRadiusOffset || 4.0);
    const eatRadius = Math.max(meanRadius + 10, baseEatRadius);
    const nearbyFood = grid.getNearbyFood(agent.px, agent.py, eatRadius);
    nearbyFood.forEach(pellet => {
      const dx = pellet.x - agent.px;
      const dy = pellet.y - agent.py;

      const d = Math.sqrt(dx*dx + dy*dy);
      if (d <= eatRadius) {
        const consumedBiome = getBiomeAt(world, pellet.x, pellet.y);
        const baseEnergy = consumedBiome ? consumedBiome.sporeEnergyValue : 15.0;

        spawnSporeBiologically(pellet);

        const herbivoreEfficiency = 1.0 - agent.phenotype.carnivory;
        if (herbivoreEfficiency > 0.05) {
          const energyGain = baseEnergy * herbivoreEfficiency * (rules.grazingEfficiencyHerbivoreScale || 1.25);
          agent.energy = Math.min(agent.phenotype.stomachCapacity, agent.energy + energyGain);
          agent.hasEaten = true;

          // Broadcast Eat event for client-side algae explosion sparks!
          broadcast({
            type: "EAT_EVENT",
            agentId: agent.id,
            x: pellet.x,
            y: pellet.y
          });
        }
      }
    });

    // B. Fight Biting (optimized via SpatialGrid)
    if (agent.phenotype.carnivory >= (rules.bitingCarnivoryThreshold || 0.35)) {
      const biteRange = meanRadius * (rules.bitingRadiusMultiplier || 1.6) * 0.5 + (rules.bitingRadiusOffset || 5.0);
      const nearbyPeers = grid.getNearbyCreatures(agent.px, agent.py, biteRange);
      nearbyPeers.forEach(victim => {
        if (victim.id === agent.id) return;
        if (victim.speciesId === agent.speciesId) return;

        const dx = victim.px - agent.px;
        const dy = victim.py - agent.py;

        const dist = Math.sqrt(dx*dx + dy*dy);
        if (dist <= biteRange) {
          victim.energy = Math.max(0.0, victim.energy - (rules.bitingEnergyDamage || 50.0));

          const carnivoreEfficiency = agent.phenotype.carnivory;
          const energyGain = (rules.bitingBaseEnergyGain || 45.0) * carnivoreEfficiency * (rules.bitingEfficiencyCarnivoreScale || 1.25);
          agent.energy = Math.min(agent.phenotype.stomachCapacity, agent.energy + energyGain);
          agent.hasEaten = true;

          biteImpacts.push({ x: victim.px, y: victim.py, age: 0 });

          // Broadcast bite shockwaves to render on browser!
          broadcast({
            type: "BITE_EVENT",
            attackerId: agent.id,
            victimId: victim.id,
            x: victim.px,
            y: victim.py
          });

          broadcast({
            type: "LOG_EVENT",
            message: `⚡ [BITE ATTACK] ${agent.phenotype.latinName.substring(0, 16)} #${agent.id} bites #${victim.id}! (+${Math.round(energyGain)}nJ / -50nJ damage)`,
            logType: "mutation"
          });
        }
      });
    }

    // 6. Mitosis divisions asexual cloning
    const canReproduce = agent.age >= Math.max(rules.creatureMatureAgeFloor || 600, agent.phenotype.matureAge) && agent.hasEaten;
    const reachedReproThreshold = agent.energy >= agent.phenotype.stomachCapacity * Math.max(rules.reproductionStomachThresholdFloor || 0.65, agent.phenotype.reproThreshold);

    if (canReproduce && reachedReproThreshold && creatures.length < (config.maxCreatures || 45)) {
      const splitLoss = agent.phenotype.splitLoss;
      const energyPool = agent.energy;

      const parentEnergyAfter = energyPool * (rules.reproductionSplitLossRatio !== undefined ? rules.reproductionSplitLossRatio : 0.4) * (1.0 - splitLoss);
      const childEnergyAfter = energyPool * (rules.reproductionSplitLossRatio !== undefined ? rules.reproductionSplitLossRatio : 0.4) * (1.0 - splitLoss);
      agent.energy = parentEnergyAfter;
      agent.hasEaten = false;

      const recoilVelocity = splitLoss * (rules.reproductionRecoilVelocityScale || 15.0);
      agent.vx += recoilVelocity * Math.cos(agent.headingAngle + Math.PI);
      agent.vy += recoilVelocity * Math.sin(agent.headingAngle + Math.PI);

      let childGenome = agent.genome;
      let isMutated = false;
      let isLamarckian = false;

      // Lamarckian Assimilation
      if (agent.age > (rules.lamarckianAssimilationAgeFloor || 1200)) {
        let mCopy = [...agent.phenotype.methylations];
        let tempGenome = childGenome;
        const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        for (let i = 16; i <= 31; i++) {
          if (mCopy[i] !== 0 && Math.random() < (rules.lamarckianAssimilationChance || 0.25)) {
            const baseVal = ALPHABET.indexOf(tempGenome[i]);
            const shift = mCopy[i];
            const assimilatedChar = ALPHABET[(baseVal + shift + 26) % 26];
            tempGenome = tempGenome.substring(0, i) + assimilatedChar + tempGenome.substring(i + 1);
            mCopy[i] = 0;
            isLamarckian = true;
          }
        }
        if (isLamarckian) {
          childGenome = tempGenome;
          agent.phenotype.methylations = mCopy;
        }
      }

      // Mitosis mutation
      const childMutationRate = 1.0 - agent.phenotype.repairFidelity;
      if (Math.random() < childMutationRate) {
        const mut = mutateGenome(childGenome);
        childGenome = mut.newGenome;
        isMutated = true;
      }

      const childAntisense = getComplementaryString(childGenome);
      const childPhenotype = parseGenome(childGenome, childAntisense, [...agent.phenotype.methylations]);

      const childPx = agent.px + Math.random() * 32 - 16;
      const childPy = agent.py + Math.random() * 32 - 16;
      const child: CreatureAgent = {
        id: nextAgentId++,
        speciesId: childGenome,
        genome: childGenome,
        antisense: childAntisense,
        phenotype: childPhenotype,
        px: Math.max(childPhenotype.spinalHarmonics.meanRadius, Math.min(logicalWidth - childPhenotype.spinalHarmonics.meanRadius, childPx)),
        py: Math.max(childPhenotype.spinalHarmonics.meanRadius, Math.min(logicalHeight - childPhenotype.spinalHarmonics.meanRadius, childPy)),
        vx: -agent.vx * 0.4 + recoilVelocity * Math.cos(agent.headingAngle),
        vy: -agent.vy * 0.4 + recoilVelocity * Math.sin(agent.headingAngle),
        headingAngle: agent.headingAngle + Math.PI,
        omegaRot: 0,
        energy: childEnergyAfter,
        age: 0,
        generation: agent.generation + 1,
        adrenaline: 1.0,
        hasEaten: false,
        neuronStates: [],
        neuronActivations: []
      };

      newbornAgents.push(child);
      highestGeneration = Math.max(highestGeneration, child.generation);

      // Broadcast newborn details so client can reconstruct its phenotype correctly!
      broadcast({
        type: "CREATURE_SPAWNED",
        creature: {
          id: child.id,
          speciesId: child.speciesId,
          genome: child.genome,
          antisense: child.antisense,
          px: child.px,
          py: child.py,
          vx: child.vx,
          vy: child.vy,
          headingAngle: child.headingAngle,
          omegaRot: child.omegaRot,
          energy: child.energy,
          adrenaline: child.adrenaline,
          age: child.age,
          generation: child.generation,
          hasEaten: child.hasEaten,
          preferredHabitat: childPhenotype.preferredHabitat,
          primaryColor: childPhenotype.primaryColor,
          stiffness: childPhenotype.stiffness,
          meanRadius: childPhenotype.spinalHarmonics.meanRadius,
          baseLength: childPhenotype.spinalHarmonics.baseLength,
          latinName: childPhenotype.latinName
        }
      });

      // Save mutated species into ledger db
      if (isMutated) {
        registerSpeciesIfNew(childGenome, agent.speciesId, child.generation);
      }

      broadcast({
        type: "LOG_EVENT",
        message: `Mitosis Spawn! Species ${childPhenotype.latinName.substring(0, 16)} #${child.id} spawned (descendant of #${agent.id}, Gen: ${child.generation}${isMutated ? ", MUTATED!" : ", clone"}).`,
        logType: isMutated ? "mutation" : "system"
      });

      if (isLamarckian) {
        broadcast({
          type: "LOG_EVENT",
          message: `⚡ [Lamarckism] Species #${agent.id} successfully assimilated learned experiences into the genome of descendant #${child.id}!`,
          logType: "repair"
        });
      }
    }
  });

  // Apply removals & newborns
  creatures = creatures.filter(agent => !deadAgentIds.includes(agent.id));
  creatures.push(...newbornAgents);

  // Maintain population (restocking founder cells)
  const targetPopulation = config.targetPopulation !== undefined ? config.targetPopulation : 25;
  
  // Fetch top-1 all-time champions across all training sessions
  const trainingChamps = getAllTrainingsChampions();

  while (creatures.length < targetPopulation) {
    let g = "";
    let gen = 1;

    const roll = Math.random();
    if (trainingChamps.length > 0 && roll < 0.40) {
      // 40% Chance: Spawn one of our trained champions from the database!
      const champIdx = Math.floor(Math.random() * trainingChamps.length);
      const champ = trainingChamps[champIdx];
      g = champ.genome;
      gen = champ.generation;
    } else if (cachedAliveSpecies.length > 0 && roll < 0.80) {
      // 40% Chance: Clone an already successful species living in the ocean
      const idx = Math.floor(Math.random() * cachedAliveSpecies.length);
      const record = cachedAliveSpecies[idx];
      g = record.genome;
      gen = record.generation;
    } else {
      // 20% Chance: Wild random founder mutation
      g = generateRandomGenome(256);
      gen = 1;
    }

    const anti = getComplementaryString(g);
    const pheno = parseGenome(g, anti);

    registerSpeciesIfNew(g, null, gen);

    const founder: CreatureAgent = {
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
      energy: pheno.stomachCapacity * (rules.restockInitialStomachRatio !== undefined ? rules.restockInitialStomachRatio : 0.60),
      age: 0,
      generation: gen,
      adrenaline: 1.0,
      hasEaten: false,
      neuronStates: [],
      neuronActivations: []
    };

    creatures.push(founder);

    // Broadcast newly spawned founder details to all clients synchronously!
    broadcast({
      type: "CREATURE_SPAWNED",
      creature: {
        id: founder.id,
        speciesId: founder.speciesId,
        genome: founder.genome,
        antisense: founder.antisense,
        px: founder.px,
        py: founder.py,
        vx: founder.vx,
        vy: founder.vy,
        headingAngle: founder.headingAngle,
        omegaRot: founder.omegaRot,
        energy: founder.energy,
        adrenaline: founder.adrenaline,
        age: founder.age,
        generation: founder.generation,
        hasEaten: founder.hasEaten,
        preferredHabitat: pheno.preferredHabitat,
        primaryColor: pheno.primaryColor,
        stiffness: pheno.stiffness,
        meanRadius: pheno.spinalHarmonics.meanRadius,
        baseLength: pheno.spinalHarmonics.baseLength,
        latinName: pheno.latinName
      }
    });
  }

  // C. Physical Collisions Phase (After updating all positions and restocks)
  // 1. Re-populate the spatial grid with final creature and pellet positions
  grid.clear();
  foodPellets.forEach(pellet => grid.insertFood(pellet));
  creatures.forEach(agent => grid.insertCreature(agent));

  // 2. Resolve Creature-to-Creature physical collisions (mass-based push-out & elastic momentum transfer)
  for (let i = 0; i < creatures.length; i++) {
    const agent1 = creatures[i];
    const r1 = agent1.phenotype.spinalHarmonics.meanRadius;
    const mass1 = Math.pow(r1, 1.5) * (agent1.phenotype.spinalHarmonics.baseLength / 25);

    const nearbyPeers = grid.getNearbyCreatures(agent1.px, agent1.py, r1 * 2 + 100);
    for (let j = 0; j < nearbyPeers.length; j++) {
      const agent2 = nearbyPeers[j];
      if (agent1.id === agent2.id) continue;

      const r2 = agent2.phenotype.spinalHarmonics.meanRadius;
      const mass2 = Math.pow(r2, 1.5) * (agent2.phenotype.spinalHarmonics.baseLength / 25);

      const dx = agent2.px - agent1.px;
      const dy = agent2.py - agent1.py;
      const d = Math.sqrt(dx * dx + dy * dy);
      const minDist = r1 + r2;

      if (d < minDist && d > 0.1) {
        const overlap = minDist - d;
        const nx = dx / d;
        const ny = dy / d;

        // Overlap resolution proportional to inverse mass
        const totalMass = mass1 + mass2;
        const push1 = overlap * (mass2 / totalMass);
        const push2 = overlap * (mass1 / totalMass);

        agent1.px -= nx * push1;
        agent1.py -= ny * push1;
        agent2.px += nx * push2;
        agent2.py += ny * push2;

        // Clip back inside boundaries
        if (agent1.px < r1) agent1.px = r1;
        else if (agent1.px > logicalWidth - r1) agent1.px = logicalWidth - r1;
        if (agent1.py < r1) agent1.py = r1;
        else if (agent1.py > logicalHeight - r1) agent1.py = logicalHeight - r1;

        if (agent2.px < r2) agent2.px = r2;
        else if (agent2.px > logicalWidth - r2) agent2.px = logicalWidth - r2;
        if (agent2.py < r2) agent2.py = r2;
        else if (agent2.py > logicalHeight - r2) agent2.py = logicalHeight - r2;

        // Elastic momentum transfer (impact bounce)
        const rvx = agent2.vx - agent1.vx;
        const rvy = agent2.vy - agent1.vy;
        const vnorm = rvx * nx + rvy * ny;

        if (vnorm < 0) {
          const e = 0.5; // coefficient of restitution
          const J = -(1 + e) * vnorm / ((1 / mass1) + (1 / mass2));

          agent1.vx -= (J * nx) / mass1;
          agent1.vy -= (J * ny) / mass1;
          agent2.vx += (J * nx) / mass2;
          agent2.vy += (J * ny) / mass2;
        }
      }
    }
  }

  // 3. Resolve Creature-to-Food physical collisions (pushing spores with bugwave impulse)
  creatures.forEach(agent => {
    const r = agent.phenotype.spinalHarmonics.meanRadius;
    const nearbyFood = grid.getNearbyFood(agent.px, agent.py, r + 20);
    nearbyFood.forEach(pellet => {
      const dx = pellet.x - agent.px;
      const dy = pellet.y - agent.py;
      const d = Math.sqrt(dx * dx + dy * dy);
      const minDist = r + 8; // Spore radius = 8

      if (d < minDist && d > 0.1) {
        const overlap = minDist - d;
        const nx = dx / d;
        const ny = dy / d;

        // Push spore away
        pellet.x += nx * overlap;
        pellet.y += ny * overlap;

        // Clip spore to boundary
        if (pellet.x < 8) pellet.x = 8;
        else if (pellet.x > logicalWidth - 8) pellet.x = logicalWidth - 8;
        if (pellet.y < 8) pellet.y = 8;
        else if (pellet.y > logicalHeight - 8) pellet.y = logicalHeight - 8;

        // Impart velocity push
        pellet.vx = agent.vx + nx * 2.0;
        pellet.vy = agent.vy + ny * 2.0;
      }
    });
  });

  // Update active species tracker and mark extinctions on database
  const activeIdsThisFrame = new Set(creatures.map(c => c.speciesId));
  const db = readDb();
  let dbChanged = false;

  db.forEach(rec => {
    if (rec.status === "alive" && !activeIdsThisFrame.has(rec.id)) {
      rec.status = "extinct";
      rec.extinctionTime = Date.now();
      dbChanged = true;
      broadcast({
        type: "LOG_EVENT",
        message: `✝️ Extinction Event! Species '${rec.name}' has gone completely extinct. The only relics remain in the silent fossil archive.`,
        logType: "mutation"
      });
    } else if (activeIdsThisFrame.has(rec.id)) {
      const currentCount = creatures.filter(c => c.speciesId === rec.id).length;
      if (currentCount > rec.peakPopulation) {
        rec.peakPopulation = currentCount;
        dbChanged = true;
      }
    }
  });

  if (dbChanged) {
    writeDb(db);
    cachedAliveSpecies = db.filter(rec => rec.status === "alive");
  }

  // C. Periodic state save (Every 10 seconds)
  if (tickCount % 600 === 0) {
    saveSimulationStateOnDisk();
  }

  // D. Build and send the Telemetry tick payload to all clients!
  let selectedBrain = null;
  if (selectedAgentId !== null) {
    const selected = creatures.find(c => c.id === selectedAgentId);
    if (selected) {
      selectedBrain = {
        id: selected.id,
        activations: selected.neuronActivations
      };
    }
  }

  const telemetryPayload = {
    type: "TELEMETRY_TICK",
    creatures: creatures.map(c => ({
      id: c.id,
      speciesId: c.speciesId,
      px: c.px,
      py: c.py,
      vx: c.vx,
      vy: c.vy,
      headingAngle: c.headingAngle,
      omegaRot: c.omegaRot,
      energy: c.energy,
      adrenaline: c.adrenaline,
      age: c.age,
      generation: c.generation,
      hasEaten: c.hasEaten,
      preferredHabitat: c.phenotype.preferredHabitat,
      primaryColor: c.phenotype.primaryColor,
      stiffness: c.phenotype.stiffness,
      meanRadius: c.phenotype.spinalHarmonics.meanRadius,
      baseLength: c.phenotype.spinalHarmonics.baseLength,
      latinName: c.phenotype.latinName
    })),
    foodPellets,
    nextAgentId,
    highestGeneration,
    biteImpacts,
    selectedBrain
  };

  broadcast(telemetryPayload);
}

import { applyCreaturePhysics } from "../shared/physics";
import { computeSensoryInputs } from "../shared/sensory";

// --------------------------------------------------------------------------
// Headless Sensory Inputs Scanners
// --------------------------------------------------------------------------
function computeHeadlessSensoryInputs(agent: CreatureAgent, clockVal: number, grid: SpatialGrid): number[] {
  return computeSensoryInputs(agent, clockVal, grid, logicalWidth, logicalHeight);
}

// --------------------------------------------------------------------------
// Express/HTTP WebSocket Server Setup
// --------------------------------------------------------------------------
// Helper to read POST body
const readBody = (req: http.IncomingMessage): Promise<string> => {
  return new Promise((resolve) => {
    let body = "";
    req.on("data", (chunk) => {
      body += chunk.toString();
    });
    req.on("end", () => {
      resolve(body);
    });
  });
};

const server = http.createServer(async (req, res) => {
  // REST backend fallbacks
  const url = new URL(req.url || "", `http://${req.headers.host}`);
  const method = req.method;

  // CORS headers
  const headers = {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
    "Content-Type": "application/json"
  };

  if (method === "OPTIONS") {
    res.writeHead(204, headers);
    res.end();
    return;
  }

  if (method === "GET" && url.pathname === "/api/species") {
    const id = url.searchParams.get("id");
    const db = readDb();
    if (!id) {
      res.writeHead(200, headers);
      res.end(JSON.stringify(db));
      return;
    }
    const record = db.find(rec => rec.id === id);
    res.writeHead(200, headers);
    res.end(JSON.stringify(record || null));
    return;
  }

  // 1. GET simulation active running status
  if (method === "GET" && url.pathname === "/api/simulation/status") {
    res.writeHead(200, headers);
    res.end(JSON.stringify({ running: isSimulationRunning }));
    return;
  }

  // 2. POST toggle simulation active running state
  if (method === "POST" && url.pathname === "/api/simulation/toggle") {
    try {
      const bodyStr = await readBody(req);
      const data = JSON.parse(bodyStr);
      if (data.running !== undefined) {
        isSimulationRunning = data.running;
        console.log(`[SERVER] Main simulation state updated: ${isSimulationRunning ? "RUNNING" : "PAUSED"}`);
        // Broadcast simulation status to all WebSocket clients
        connectedClients.forEach(ws => {
          sendToClient(ws, { type: "SIM_STATE", running: isSimulationRunning });
        });
      }
      res.writeHead(200, headers);
      res.end(JSON.stringify({ success: true, running: isSimulationRunning }));
    } catch (err) {
      res.writeHead(500, headers);
      res.end(JSON.stringify({ error: "Failed to toggle simulation state" }));
    }
    return;
  }

  // 3. GET current active trainer population for a specific runId
  if (method === "GET" && url.pathname === "/api/trainer/population") {
    const runId = url.searchParams.get("runId") || "default_run";
    const limit = parseInt(url.searchParams.get("limit") || "25");
    const pop = getTrainerPopulation(runId, limit);
    res.writeHead(200, headers);
    res.end(JSON.stringify(pop));
    return;
  }

  // 3b. GET historically best trainer genomes (Hall of Fame) for a specific runId
  if (method === "GET" && url.pathname === "/api/trainer/hof") {
    const runId = url.searchParams.get("runId") || "default_run";
    const limit = parseInt(url.searchParams.get("limit") || "10");
    const hof = getTrainerHallOfFame(runId, limit);
    res.writeHead(200, headers);
    res.end(JSON.stringify(hof));
    return;
  }

  // 4. GET list of all unique trainer runs / sessions
  if (method === "GET" && url.pathname === "/api/trainer/runs") {
    const runs = getTrainerRuns();
    res.writeHead(200, headers);
    res.end(JSON.stringify(runs));
    return;
  }

  // 5. POST save current generation elite to trainer_genomes SQLite
  if (method === "POST" && url.pathname === "/api/trainer/generation") {
    try {
      const bodyStr = await readBody(req);
      const data = JSON.parse(bodyStr);
      const runId = data.runId || "default_run";
      saveTrainerGeneration(runId, data.generation, data.population);
      res.writeHead(200, headers);
      res.end(JSON.stringify({ success: true }));
    } catch (err) {
      res.writeHead(500, headers);
      res.end(JSON.stringify({ error: "Failed to save generation" }));
    }
    return;
  }

  // 6. POST reset trainer genomes for a specific runId
  if (method === "POST" && url.pathname === "/api/trainer/reset") {
    try {
      const bodyStr = await readBody(req);
      const data = JSON.parse(bodyStr);
      const runId = data.runId || "default_run";
      clearTrainerHistory(runId);
      res.writeHead(200, headers);
      res.end(JSON.stringify({ success: true }));
    } catch (err) {
      res.writeHead(500, headers);
      res.end(JSON.stringify({ error: "Failed to reset run" }));
    }
    return;
  }

  // 7. POST apply champion to config rules
  if (method === "POST" && url.pathname === "/api/trainer/apply") {
    try {
      const bodyStr = await readBody(req);
      const data = JSON.parse(bodyStr);
      if (data.genome) {
        config.rules = config.rules || {};
        config.rules.progenitorGenome = data.genome;
        fs.writeFileSync(CONFIG_PATH, JSON.stringify(config, null, 2), "utf8");
        console.log(`[SERVER] Champion progenitor genome successfully applied to substrate config!`);
      }
      res.writeHead(200, headers);
      res.end(JSON.stringify({ success: true }));
    } catch (err) {
      res.writeHead(500, headers);
      res.end(JSON.stringify({ error: "Failed to apply champion" }));
    }
    return;
  }

  res.writeHead(404, headers);
  res.end(JSON.stringify({ error: "Not Found" }));
});

const wss = new WebSocketServer({ server });

wss.on("connection", (ws) => {
  connectedClients.add(ws);
  console.log(`[SERVER] Browser connected. Active clients=${connectedClients.size}`);

  // Send initial setup
  sendToClient(ws, {
    type: "INIT_STATE",
    seed: config.seed || "ALIFE_BASIN_77A",
    rules: config.rules,
    running: isSimulationRunning,
    creatures: creatures.map(c => ({
      id: c.id,
      speciesId: c.speciesId,
      genome: c.genome,
      antisense: c.antisense,
      px: c.px,
      py: c.py,
      vx: c.vx,
      vy: c.vy,
      headingAngle: c.headingAngle,
      omegaRot: c.omegaRot,
      energy: c.energy,
      adrenaline: c.adrenaline,
      age: c.age,
      generation: c.generation,
      hasEaten: c.hasEaten,
      preferredHabitat: c.phenotype.preferredHabitat,
      primaryColor: c.phenotype.primaryColor,
      stiffness: c.phenotype.stiffness,
      meanRadius: c.phenotype.spinalHarmonics.meanRadius,
      baseLength: c.phenotype.spinalHarmonics.baseLength,
      latinName: c.phenotype.latinName
    })),
    foodPellets,
    nextAgentId,
    highestGeneration
  });

  ws.on("message", (message) => {
    try {
      const event = JSON.parse(message.toString());
      
      if (event.type === "SELECT_AGENT") {
        selectedAgentId = event.id;
        console.log(`[SERVER] Specimen inspection focus: #${selectedAgentId}`);
      }
      else if (event.type === "DESELECT_AGENT") {
        selectedAgentId = null;
      }
      else if (event.type === "SPAWN_FOOD") {
        foodPellets.push({
          x: event.x,
          y: event.y,
          vx: (Math.random() * 0.4 - 0.2),
          vy: (Math.random() * 0.4 - 0.2)
        });
      }
      else if (event.type === "INJECT_URZELLE") {
        const g = generateRandomGenome(256);
        const anti = getComplementaryString(g);
        const pheno = parseGenome(g, anti);

        registerSpeciesIfNew(g, null, 1);

        const newAgent: CreatureAgent = {
          id: nextAgentId++,
          speciesId: g,
          genome: g,
          antisense: anti,
          phenotype: pheno,
          px: event.x !== undefined ? event.x : Math.random() * logicalWidth,
          py: event.y !== undefined ? event.y : Math.random() * logicalHeight,
          vx: (Math.random() * 0.8 - 0.4),
          vy: (Math.random() * 0.8 - 0.4),
          headingAngle: Math.random() * Math.PI * 2,
          omegaRot: 0,
          energy: pheno.stomachCapacity * 0.8,
          age: 0,
          generation: 1,
          adrenaline: 1.0,
          hasEaten: false,
          neuronStates: [],
          neuronActivations: []
        };

        creatures.push(newAgent);

        // Broadcast newly injected creature to all clients so they can render it!
        broadcast({
          type: "CREATURE_SPAWNED",
          creature: {
            id: newAgent.id,
            speciesId: newAgent.speciesId,
            genome: newAgent.genome,
            antisense: newAgent.antisense,
            px: newAgent.px,
            py: newAgent.py,
            vx: newAgent.vx,
            vy: newAgent.vy,
            headingAngle: newAgent.headingAngle,
            omegaRot: newAgent.omegaRot,
            energy: newAgent.energy,
            adrenaline: newAgent.adrenaline,
            age: newAgent.age,
            generation: newAgent.generation,
            hasEaten: newAgent.hasEaten,
            preferredHabitat: pheno.preferredHabitat,
            primaryColor: pheno.primaryColor,
            stiffness: pheno.stiffness,
            meanRadius: pheno.spinalHarmonics.meanRadius,
            baseLength: pheno.spinalHarmonics.baseLength,
            latinName: pheno.latinName
          }
        });

        broadcast({
          type: "LOG_EVENT",
          message: `Injection Command executed! Completely new, foreign founder cell ${pheno.latinName.substring(0, 16)} #${newAgent.id} released!`,
          logType: "system"
        });
      }
      else if (event.type === "RESET_EVOLUTION") {
        creatures = [];
        foodPellets = [];
        nextAgentId = 1;
        highestGeneration = 1;
        selectedAgentId = null;

        // Wipe files
        clearState();
        writeDb([]);
        cachedAliveSpecies = [];

        // Repopulate
        initSimulation();

        // Broadcast fresh, wiped INIT_STATE to all clients to clear and overwrite browser views!
        broadcast({
          type: "INIT_STATE",
          seed: config.seed || "ALIFE_BASIN_77A",
          creatures: creatures.map(c => ({
            id: c.id,
            speciesId: c.speciesId,
            genome: c.genome,
            antisense: c.antisense,
            px: c.px,
            py: c.py,
            vx: c.vx,
            vy: c.vy,
            headingAngle: c.headingAngle,
            omegaRot: c.omegaRot,
            energy: c.energy,
            adrenaline: c.adrenaline,
            age: c.age,
            generation: c.generation,
            hasEaten: c.hasEaten,
            preferredHabitat: c.phenotype.preferredHabitat,
            primaryColor: c.phenotype.primaryColor,
            stiffness: c.phenotype.stiffness,
            meanRadius: c.phenotype.spinalHarmonics.meanRadius,
            baseLength: c.phenotype.spinalHarmonics.baseLength,
            latinName: c.phenotype.latinName
          })),
          foodPellets,
          nextAgentId,
          highestGeneration
        });

        broadcast({
          type: "LOG_EVENT",
          message: `RESET Command! Biosphere completely evaporated. A new founder epoch has been started.`,
          logType: "system"
        });
      }
    } catch (err) {
      console.error("[SERVER] Failed to parse browser message:", err);
    }
  });

  ws.on("close", () => {
    connectedClients.delete(ws);
    console.log(`[SERVER] Browser disconnected. Active clients=${connectedClients.size}`);
  });
});

// Run server initialization
initSimulation();

// Boot server onfallback port 3002 to avoid Vite collision on port 3001 fallback!
const PORT = 3002;
server.listen(PORT, () => {
  console.log(`\n\x1b[36m============================================================\x1b[0m`);
  console.log(`\x1b[36m🚀 HEADLESS BIOLOGICAL CTRNN SIMULATOR RUNNING ON PORT ${PORT}\x1b[0m`);
  console.log(`\x1b[36m============================================================\x1b[0m\n`);
  
  // Continuous 60hz (16.6ms) physics tick loop!
  setInterval(simulationTick, 16.67);
});
