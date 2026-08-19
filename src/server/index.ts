import { WebSocketServer, WebSocket } from "ws";
import http from "http";
import fs from "fs";
import path from "path";
import { parseGenome, generateRandomGenome, getComplementaryString, executeBrain, mutateGenome } from "../biology/dna";
import { CreatureAgent, FoodSpore, SpeciesRecord } from "../shared/types";

import { readDb, writeDb, readState, writeState, clearState } from "./db";

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
    for (let i = 0; i < 300; i++) {
      foodPellets.push({
        x: Math.random() * logicalWidth,
        y: Math.random() * logicalHeight,
        vx: (Math.random() * 0.3 - 0.15),
        vy: (Math.random() * 0.3 - 0.15)
      });
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

function simulationTick() {
  tickCount++;

  // A. Move and drift food spores
  foodPellets.forEach(pellet => {
    pellet.x += pellet.vx;
    pellet.y += pellet.vy;

    // Soft elastic boundary bouncing
    const pMargin = 6;
    if (pellet.x < pMargin) { pellet.x = pMargin; pellet.vx = -pellet.vx; }
    else if (pellet.x > logicalWidth - pMargin) { pellet.x = logicalWidth - pMargin; pellet.vx = -pellet.vx; }

    if (pellet.y < pMargin) { pellet.y = pMargin; pellet.vy = -pellet.vy; }
    else if (pellet.y > logicalHeight - pMargin) { pellet.y = logicalHeight - pMargin; pellet.vy = -pellet.vy; }
  });

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
    const isGreenPrey = (agent.phenotype.carnivory < 0.35) && (hue >= 75 && hue <= 175);
    const inLightZone = agent.py < logicalHeight * 0.35;
    if (isGreenPrey && inLightZone) {
      agent.energy = Math.min(agent.phenotype.stomachCapacity, agent.energy + 0.15);
    }

    // 2. Spatial Temperature Stratification and Thermal Limits stress penalty
    const localTemp = 38.0 - (agent.py / logicalHeight) * 26.0;
    const tempMin = agent.phenotype.thermalToleranceMin;
    const tempMax = agent.phenotype.thermalToleranceMax;
    const thermalStress = localTemp < tempMin ? (tempMin - localTemp) : (localTemp > tempMax ? (localTemp - tempMax) : 0.0);
    
    if (thermalStress > 0.1) {
      agent.energy -= thermalStress * 0.0012 * frameScale;
    }

    // 3. Adrenaline sprint tax
    const metabolicSurcharge = 1.0 + (agent.adrenaline - 1.0) * 1.5;
    agent.energy -= agent.phenotype.basalMetabolicRate * 0.005 * metabolicSurcharge;

    // Starvation / Senescence death
    if (agent.energy <= 0 || agent.age >= 2700) {
      deadAgentIds.push(agent.id);

      // Decompose corpse into food pellets
      const L_dead = agent.phenotype.spinalHarmonics.baseLength;
      const r_dead = agent.phenotype.spinalHarmonics.meanRadius;
      const numPellets = Math.max(1, Math.min(5, Math.floor((L_dead * r_dead) / 1200)));

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

    let thrustMag = stiffness * (pulse * 1000 * pulse * 1000) * 6.0;
    const wavePhase = agent.phenotype.wavePhase;
    const etaSwim = Math.max(0.1, Math.min(3.2, (baseLength / (meanRadius * 3.5)) * Math.max(0.01, Math.sin(wavePhase)) * stiffness));
    thrustMag *= etaSwim;

    const limbsCount = agent.phenotype.organelles.filter(n => n.expressionStyle >= 0.72).length;
    thrustMag *= (1.0 + limbsCount * 0.12);
    thrustMag *= (1.0 + agent.phenotype.spinalHarmonics.parapodiaAmp * 1.0);

    // Collect Senses
    const clockVal = 0.5 + 0.5 * Math.sin(Date.now() * 0.0012 + agent.id);
    const inputs = computeHeadlessSensoryInputs(agent, clockVal);

    const brainRes = executeBrain(agent.phenotype.brain, inputs, agent.neuronStates, agent.neuronActivations);
    const outputs = brainRes.outputs;

    const outThrust = outputs[0];
    const outLeft = outputs[1];
    const outRight = outputs[2];
    const outFlash = outputs[3];

    if (outFlash > 0.5) {
      agent.energy -= 0.05 * outFlash;
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
      agent.adrenaline = Math.min(1.8, agent.adrenaline + 0.06 * threatPerception);
    } else {
      agent.adrenaline = Math.max(1.0, agent.adrenaline - 0.015);
    }

    let fx = 0;
    let fy = 0;
    let torque = 0;

    if (outThrust > 0.0) {
      const predatorSavageMultiplier = agent.phenotype.carnivory >= 0.55 ? 1.45 : 1.0;
      fx += outThrust * thrustMag * predatorSavageMultiplier * agent.adrenaline * Math.cos(agent.headingAngle);
      fy += outThrust * thrustMag * predatorSavageMultiplier * agent.adrenaline * Math.sin(agent.headingAngle);
    }
    torque = (outRight - outLeft) * stiffness * 5.8 * agent.adrenaline;

    // Hebbian Recurrent Synaptic Learning
    const learningRate = 0.00015 * (1.0 - stiffness * 0.85);
    const forgettingDecay = 0.0000032; // exactly 1 hour half-life
    const b = agent.phenotype.brain;

    b.synapses.forEach((syn: any) => {
      const preVal = agent.neuronActivations[syn.fromNode];
      const postVal = Math.max(0.0, agent.neuronActivations[syn.toNode]);
      
      let weight = syn.weight;
      weight += learningRate * (preVal * postVal) - forgettingDecay * weight;
      weight = Math.max(-2.5, Math.min(2.5, weight));
      syn.weight = weight;
    });

    // Integrated drag friction and accelerations
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

    agent.vx *= 0.94;
    agent.vy *= 0.94;
    agent.omegaRot *= 0.88;

    agent.px += agent.vx * frameScale;
    agent.py += agent.vy * frameScale;

    // Elastic margin boundaries rebounding
    const margin = (meanRadius * 1.5) * 0.5 + 10;
    if (agent.px < margin) { agent.px = margin; agent.vx = -agent.vx * 0.45; agent.omegaRot = -agent.omegaRot * 0.5; }
    else if (agent.px > logicalWidth - margin) { agent.px = logicalWidth - margin; agent.vx = -agent.vx * 0.45; agent.omegaRot = -agent.omegaRot * 0.5; }

    if (agent.py < margin) { agent.py = margin; agent.vy = -agent.vy * 0.45; agent.omegaRot = -agent.omegaRot * 0.5; }
    else if (agent.py > logicalHeight - margin) { agent.py = logicalHeight - margin; agent.vy = -agent.vy * 0.45; agent.omegaRot = -agent.omegaRot * 0.5; }

    // 5. Unified Spore Grazing and Combat Biting collisions!
    // A. Graze Spores
    const eatRadius = meanRadius * 1.5 * 0.5 + 4;
    foodPellets.forEach(pellet => {
      let dx = pellet.x - agent.px;
      let dy = pellet.y - agent.py;
      if (dx > logicalWidth / 2) dx -= logicalWidth;
      if (dx < -logicalWidth / 2) dx += logicalWidth;
      if (dy > logicalHeight / 2) dy -= logicalHeight;
      if (dy < -logicalHeight / 2) dy += logicalHeight;

      const d = Math.sqrt(dx*dx + dy*dy);
      if (d <= eatRadius) {
        pellet.x = Math.random() * logicalWidth;
        pellet.y = Math.random() * logicalHeight;

        const herbivoreEfficiency = 1.0 - agent.phenotype.carnivory;
        if (herbivoreEfficiency > 0.05) {
          const energyGain = 45.0 * herbivoreEfficiency * 1.25;
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

    // B. Fight Biting
    if (agent.phenotype.carnivory >= 0.35) {
      const biteRange = meanRadius * 1.6 * 0.5 + 5.0;
      creatures.forEach(victim => {
        if (victim.id === agent.id) return;
        if (victim.speciesId === agent.speciesId) return;

        let dx = victim.px - agent.px;
        let dy = victim.py - agent.py;
        if (dx > logicalWidth / 2) dx -= logicalWidth;
        if (dx < -logicalWidth / 2) dx += logicalWidth;
        if (dy > logicalHeight / 2) dy -= logicalHeight;
        if (dy < -logicalHeight / 2) dy += logicalHeight;

        const dist = Math.sqrt(dx*dx + dy*dy);
        if (dist <= biteRange) {
          victim.energy = Math.max(0.0, victim.energy - 50.0);

          const carnivoreEfficiency = agent.phenotype.carnivory;
          const energyGain = 45.0 * carnivoreEfficiency * 1.25;
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
    const canReproduce = agent.age >= Math.max(600, agent.phenotype.matureAge) && agent.hasEaten;
    const reachedReproThreshold = agent.energy >= agent.phenotype.stomachCapacity * Math.max(0.65, agent.phenotype.reproThreshold);

    if (canReproduce && reachedReproThreshold && creatures.length < 45) {
      const splitLoss = agent.phenotype.splitLoss;
      const energyPool = agent.energy;

      const parentEnergyAfter = energyPool * 0.4 * (1.0 - splitLoss);
      const childEnergyAfter = energyPool * 0.4 * (1.0 - splitLoss);
      agent.energy = parentEnergyAfter;
      agent.hasEaten = false;

      const recoilVelocity = splitLoss * 15.0;
      agent.vx += recoilVelocity * Math.cos(agent.headingAngle + Math.PI);
      agent.vy += recoilVelocity * Math.sin(agent.headingAngle + Math.PI);

      let childGenome = agent.genome;
      let isMutated = false;
      let isLamarckian = false;

      // Lamarckian Assimilation
      if (agent.age > 1200) {
        let mCopy = [...agent.phenotype.methylations];
        let tempGenome = childGenome;
        const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        for (let i = 16; i <= 31; i++) {
          if (mCopy[i] !== 0 && Math.random() < 0.25) {
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

      const child: CreatureAgent = {
        id: nextAgentId++,
        speciesId: childGenome,
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
  const targetPopulation = 25;
  while (creatures.length < targetPopulation) {
    let g = generateRandomGenome(256);
    let gen = 1;

    if (cachedAliveSpecies.length > 0 && Math.random() < 0.60) {
      const idx = Math.floor(Math.random() * cachedAliveSpecies.length);
      const record = cachedAliveSpecies[idx];
      g = record.genome;
      gen = record.generation;
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
      energy: pheno.stomachCapacity * 0.6,
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

// --------------------------------------------------------------------------
// Headless Sensory Inputs Scanners
// --------------------------------------------------------------------------
function computeHeadlessSensoryInputs(agent: CreatureAgent, clockVal: number): number[] {
  const K = agent.phenotype.organelles.length;
  const inputs: number[] = Array(K + 1).fill(0.0);
  inputs[K] = clockVal;

  agent.phenotype.organelles.forEach((patch, idx) => {
    const aff = patch.spectralAffinity;
    const organPower = patch.scale * (1.1 - patch.bandwidth);
    const range = patch.scale * 350.0;
    const alpha = (patch.angle - 90) * (Math.PI / 180);
    const halfCone = Math.max(0.1, patch.bandwidth * 1.5);

    let maxStimulus = 0.0;

    // Spores scan
    foodPellets.forEach(pellet => {
      let dx = pellet.x - agent.px;
      let dy = pellet.y - agent.py;
      if (dx > logicalWidth / 2) dx -= logicalWidth;
      if (dx < -logicalWidth / 2) dx += logicalWidth;
      if (dy > logicalHeight / 2) dy -= logicalHeight;
      if (dy < -logicalHeight / 2) dy += logicalHeight;

      const dist = Math.sqrt(dx*dx + dy*dy);
      if (dist <= range) {
        let angleRel = Math.atan2(dy, dx) - agent.headingAngle;
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

    // Peers scan
    creatures.forEach(other => {
      if (other.id === agent.id) return;

      let dx = other.px - agent.px;
      let dy = other.py - agent.py;
      if (dx > logicalWidth / 2) dx -= logicalWidth;
      if (dx < -logicalWidth / 2) dx += logicalWidth;
      if (dy > logicalHeight / 2) dy -= logicalHeight;
      if (dy < -logicalHeight / 2) dy += logicalHeight;

      const dist = Math.sqrt(dx*dx + dy*dy);
      if (dist <= range) {
        let angleRel = Math.atan2(dy, dx) - agent.headingAngle;
        while (angleRel > Math.PI) angleRel -= Math.PI * 2;
        while (angleRel < -Math.PI) angleRel += Math.PI * 2;

        let deltaBeta = angleRel - alpha;
        while (deltaBeta > Math.PI) deltaBeta -= Math.PI * 2;
        while (deltaBeta < -Math.PI) deltaBeta += Math.PI * 2;

        if (Math.abs(deltaBeta) <= halfCone) {
          const targetVisual = other.phenotype.primaryColor.h / 360;
          const targetSmell = (other.phenotype.basalMetabolicRate % 100) / 100;
          const targetVibration = (other.phenotype.pulseSpeed * 1000) % 1.0;
          const targetHeat = (other.phenotype.carnivory >= 0.55) ? 0.85 * other.adrenaline : 0.15;

          let match = 0.0;
          if (aff >= 0.8) {
            match = Math.max(0, 1.0 - Math.abs(aff - targetVisual) / (patch.bandwidth * 1.8 + 0.12));
          } else if (aff >= 0.25 && aff <= 0.65) {
            match = Math.max(0, 1.0 - Math.abs(aff - targetSmell) / (patch.bandwidth * 1.8 + 0.12));
          } else if (aff < 0.25) {
            match = Math.max(0, 1.0 - Math.abs(aff - targetVibration) / (patch.bandwidth * 1.8 + 0.12));
          } else {
            match = Math.max(0, 1.0 - Math.abs(aff - targetHeat) / (patch.bandwidth * 1.8 + 0.12));
          }

          if (match > 0.05) {
            const strength = match * organPower * (1.0 - dist / range) * Math.cos(deltaBeta);
            maxStimulus = Math.max(maxStimulus, strength);
          }
        }
      }
    });

    // Wall warn touch
    if (aff < 0.25) {
      const wallWarningZone = range * 0.5;
      let boundaryPressure = 0.0;
      
      if (agent.px < wallWarningZone) boundaryPressure = 1.0 - agent.px / wallWarningZone;
      else if (agent.px > logicalWidth - wallWarningZone) boundaryPressure = 1.0 - (logicalWidth - agent.px) / wallWarningZone;

      if (agent.py < wallWarningZone) boundaryPressure = Math.max(boundaryPressure, 1.0 - agent.py / wallWarningZone);
      else if (agent.py > logicalHeight - wallWarningZone) boundaryPressure = Math.max(boundaryPressure, 1.0 - (logicalHeight - agent.py) / wallWarningZone);

      if (boundaryPressure > 0.0) {
        maxStimulus = Math.max(maxStimulus, boundaryPressure * organPower);
      }
    }

    // Centrifugal water rotation touch
    if (aff < 0.25) {
      const speed = Math.sqrt(agent.vx * agent.vx + agent.vy * agent.vy);
      const rotSpeed = Math.abs(agent.omegaRot);
      const proprioceptiveStimulus = Math.min(1.0, speed * 0.15 + rotSpeed * 0.35);
      if (proprioceptiveStimulus > 0.0) {
        maxStimulus = Math.max(maxStimulus, proprioceptiveStimulus * organPower);
      }
    }

    inputs[idx] = Math.max(0.0, Math.min(1.0, maxStimulus));
  });

  return inputs;
}

// --------------------------------------------------------------------------
// Express/HTTP WebSocket Server Setup
// --------------------------------------------------------------------------
const server = http.createServer((req, res) => {
  // REST backend fallbacks
  const url = new URL(req.url || "", `http://${req.headers.host}`);
  const method = req.method;

  if (method === "GET" && url.pathname === "/api/species") {
    const id = url.searchParams.get("id");
    const db = readDb();
    if (!id) {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(JSON.stringify(db));
      return;
    }
    const record = db.find(rec => rec.id === id);
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify(record || null));
    return;
  }

  res.writeHead(404);
  res.end();
});

const wss = new WebSocketServer({ server });

wss.on("connection", (ws) => {
  connectedClients.add(ws);
  console.log(`[SERVER] Browser connected. Active clients=${connectedClients.size}`);

  // Send initial setup
  sendToClient(ws, {
    type: "INIT_STATE",
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
