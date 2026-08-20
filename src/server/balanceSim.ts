import fs from "fs";
import path from "path";
import { parseGenome, generateRandomGenome, getComplementaryString, executeBrain, mutateGenome } from "../biology/dna";
import { CreatureAgent, FoodSpore } from "../shared/types";
import { generateWorld, getVectoredCurrentAt, getBiomeAt, checkObstacleCollision } from "../shared/mapGenerator";
import { SpatialGrid } from "./spatialGrid";
import { applyCreaturePhysics } from "../shared/physics";
import { computeSensoryInputs } from "../shared/sensory";

// Dimensions
const logicalWidth = 19200;
const logicalHeight = 10800;
const frameScale = 1.0;

// Load Config
const CONFIG_PATH = path.resolve("./config.json");

interface ConfigRules {
  creatureMaxAgeTicks: number;
  creatureMatureAgeFloor: number;
  lamarckianAssimilationAgeFloor: number;
  lamarckianAssimilationChance: number;
  bmrBaseScale: number;
  adrenalineMaxMultiplier: number;
  adrenalineIncreaseRate: number;
  adrenalineDecayRate: number;
  adrenalineMetabolicSurchargeScale: number;
  thermalStressPenaltyScale: number;
  bioluminescenceFlashCost: number;
  photosynthesisEnergyGain: number;
  biomeHazardDamageScale: number;
  biomeAlgaeSporeSpawnThreshold: number;
  grazingRadiusMultiplier: number;
  grazingRadiusOffset: number;
  grazingEfficiencyHerbivoreScale: number;
  bitingCarnivoryThreshold: number;
  bitingRadiusMultiplier: number;
  bitingRadiusOffset: number;
  bitingEnergyDamage: number;
  bitingBaseEnergyGain: number;
  bitingEfficiencyCarnivoreScale: number;
  reproductionStomachThresholdFloor: number;
  reproductionSplitLossRatio: number;
  reproductionRecoilVelocityScale: number;
  thrustBaseMultiplier: number;
  steerTorqueBaseMultiplier: number;
  predatorSavageThrustThreshold: number;
  predatorSavageThrustMultiplier: number;
  dragForwardCoefficient: number;
  dragForwardStiffnessDecay: number;
  dragLateralCoefficient: number;
  receptorBallastScale: number;
  elasticWallRestitution: number;
  hebbianLearningRateBase: number;
  hebbianLearningStiffnessDecay: number;
  hebbianForgettingDecay: number;
  decompositionSporeMin: number;
  decompositionSporeMax: number;
  decompositionSizeRatio: number;
  restockFounderGeneInheritChance: number;
  restockInitialStomachRatio: number;
}

interface FullConfig {
  seed: string;
  targetPopulation: number;
  foodSporeCount: number;
  maxCreatures: number;
  basalMetabolicRateMultiplier: number;
  rules: ConfigRules;
}

function loadConfig(): FullConfig {
  return JSON.parse(fs.readFileSync(CONFIG_PATH, "utf8"));
}

function saveConfig(cfg: FullConfig) {
  fs.writeFileSync(CONFIG_PATH, JSON.stringify(cfg, null, 2), "utf8");
}

// --------------------------------------------------------------------------
// Decoupled Simulation Runner
// --------------------------------------------------------------------------
export function runSimulation(cfg: FullConfig, totalTicks = 10000): {
  success: boolean;
  score: number;
  finalPopulation: number;
  finalSpores: number;
  starvationDeaths: number;
  oldAgeDeaths: number;
  avgLifespan: number;
  maxGeneration: number;
  extinct: boolean;
} {
  const rules = cfg.rules;
  const bmrMultiplier = cfg.basalMetabolicRateMultiplier || 1.0;
  const world = generateWorld(cfg.seed || "ALIFE_BASIN_77A", 19200, 10800, cfg.rules);

  let creatures: CreatureAgent[] = [];
  let foodPellets: FoodSpore[] = [];
  let nextAgentId = 1;
  let highestGeneration = 1;

  let starvationDeaths = 0;
  let oldAgeDeaths = 0;
  let totalDeadAge = 0;
  let totalDeaths = 0;

  // Spore biological spawn helper
  function spawnSporeBiologically(pellet: FoodSpore) {
    const modeRoll = Math.random();
    if (modeRoll < 0.15 && world.vents.length > 0) {
      const ventIdx = Math.floor(Math.random() * world.vents.length);
      const vent = world.vents[ventIdx];
      pellet.x = vent.x + (Math.random() * 40 - 20);
      pellet.y = vent.y + (Math.random() * 40 - 20);
      const spewAngle = Math.random() * Math.PI * 2;
      const spewSpeed = 1.5 + Math.random() * 2.5;
      pellet.vx = Math.cos(spewAngle) * spewSpeed;
      pellet.vy = Math.sin(spewAngle) * spewSpeed;
    } else {
      let accepted = false;
      let attempts = 0;
      while (!accepted && attempts < 25) {
        attempts++;
        const tx = Math.random() * logicalWidth;
        const ty = Math.random() * logicalHeight;
        const biome = getBiomeAt(world, tx, ty);
        if (biome) {
          const acceptChance = biome.sporeSpawnRate / 1.4;
          if (Math.random() < acceptChance) {
            pellet.x = tx;
            pellet.y = ty;
            pellet.vx = (Math.random() * 0.3 - 0.15);
            pellet.vy = (Math.random() * 0.3 - 0.15);
            accepted = true;
          }
        }
      }
      if (!accepted) {
        pellet.x = Math.random() * logicalWidth;
        pellet.y = Math.random() * logicalHeight;
        pellet.vx = (Math.random() * 0.3 - 0.15);
        pellet.vy = (Math.random() * 0.3 - 0.15);
      }
    }
  }

  // Populate initial spores
  const initialSporeCount = cfg.foodSporeCount || 300;
  for (let i = 0; i < initialSporeCount; i++) {
    const p: FoodSpore = { x: 0, y: 0, vx: 0, vy: 0 };
    spawnSporeBiologically(p);
    foodPellets.push(p);
  }

  // Main Loop
  for (let tick = 0; tick < totalTicks; tick++) {
    // A. Move spores
    foodPellets.forEach(pellet => {
      const current = getVectoredCurrentAt(world, pellet.x, pellet.y);
      pellet.vx = (pellet.vx + current.vx * frameScale) * 0.95;
      pellet.vy = (pellet.vy + current.vy * frameScale) * 0.95;
      pellet.x += pellet.vx * frameScale;
      pellet.y += pellet.vy * frameScale;

      const pelletRadius = 8;
      const pelletRestitution = rules.elasticWallRestitution !== undefined ? rules.elasticWallRestitution : 0.5;
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

      const collision = checkObstacleCollision(world, pellet.x, pellet.y, 8);
      if (collision.collided) {
        pellet.x += collision.normalX * collision.overlap;
        pellet.y += collision.normalY * collision.overlap;
        const dot = pellet.vx * collision.normalX + pellet.vy * collision.normalY;
        pellet.vx = (pellet.vx - 2.0 * dot * collision.normalX) * 0.5;
        pellet.vy = (pellet.vy - 2.0 * dot * collision.normalY) * 0.5;
      }
    });

    // Spore vegetative reproduction
    const maxSporeCeiling = (cfg.foodSporeCount || 300) * 1.5;
    if (foodPellets.length < maxSporeCeiling && Math.random() < 0.05) {
      const idx = Math.floor(Math.random() * foodPellets.length);
      const parentSpore = foodPellets[idx];
      if (parentSpore) {
        const biome = getBiomeAt(world, parentSpore.x, parentSpore.y);
        if (biome && biome.sporeSpawnRate >= (rules.biomeAlgaeSporeSpawnThreshold || 1.0)) {
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

    // Wide phase spatial grid
    const grid = new SpatialGrid();
    foodPellets.forEach(pellet => grid.insertFood(pellet));
    creatures.forEach(agent => grid.insertCreature(agent));

    const deadAgentIds: number[] = [];
    const newbornAgents: CreatureAgent[] = [];

    // B. Creatures physics and metabolism
    creatures.forEach(agent => {
      agent.age += 1;

      // 1. Photosynthesis
      const hue = agent.phenotype.primaryColor.h;
      const isGreenPrey = (agent.phenotype.carnivory < (rules.bitingCarnivoryThreshold || 0.35)) && (hue >= 75 && hue <= 175);
      const inLightZone = agent.py < logicalHeight * 0.35;
      if (isGreenPrey && inLightZone) {
        agent.energy = Math.min(agent.phenotype.stomachCapacity, agent.energy + (rules.photosynthesisEnergyGain || 0.15));
      }

      // 2. Temp stress
      const localTemp = 38.0 - (agent.py / logicalHeight) * 26.0;
      const tempMin = agent.phenotype.thermalToleranceMin;
      const tempMax = agent.phenotype.thermalToleranceMax;
      const thermalStress = localTemp < tempMin ? (tempMin - localTemp) : (localTemp > tempMax ? (localTemp - tempMax) : 0.0);
      if (thermalStress > 0.1) {
        agent.energy -= thermalStress * (rules.thermalStressPenaltyScale || 0.0012) * frameScale;
      }

      // 3. Metabolic tax
      const metabolicSurcharge = 1.0 + (agent.adrenaline - 1.0) * (rules.adrenalineMetabolicSurchargeScale || 1.5);
      agent.energy -= agent.phenotype.basalMetabolicRate * (rules.bmrBaseScale || 0.005) * metabolicSurcharge * bmrMultiplier;

      // Death check
      if (agent.energy <= 0 || agent.age >= (rules.creatureMaxAgeTicks || 2700)) {
        deadAgentIds.push(agent.id);
        totalDeaths++;
        totalDeadAge += agent.age;
        if (agent.energy <= 0) starvationDeaths++;
        else oldAgeDeaths++;

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
        return;
      }

      // Brain controller
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

      const clockVal = 0.5 + 0.5 * Math.sin(tick * 0.1);
      const inputs = computeSensoryInputs(agent, clockVal, grid, logicalWidth, logicalHeight);
      const brainRes = executeBrain(agent.phenotype.brain, inputs, agent.neuronStates, agent.neuronActivations);
      const outputs = brainRes.outputs;

      const outThrust = outputs[0];
      const outLeft = outputs[1];
      const outFlash = outputs[3];

      if (outFlash > 0.5) {
        agent.energy -= (rules.bioluminescenceFlashCost || 0.05) * outFlash;
      }

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

      // Synapse Learning
      const learningRate = (rules.hebbianLearningRateBase || 0.00015) * (1.0 - stiffness * (rules.hebbianLearningStiffnessDecay || 0.85));
      const forgettingDecay = rules.hebbianForgettingDecay || 0.0000032;
      agent.phenotype.brain.synapses.forEach((syn: any) => {
        const preVal = agent.neuronActivations[syn.fromNode];
        const postVal = Math.max(0.0, agent.neuronActivations[syn.toNode]);
        syn.weight = Math.max(-2.5, Math.min(2.5, syn.weight + learningRate * (preVal * postVal) - forgettingDecay * syn.weight));
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

      const currentBiome = getBiomeAt(world, agent.px, agent.py);
      if (currentBiome && currentBiome.hazardDamage > 0) {
        agent.energy = Math.max(0.0, agent.energy - currentBiome.hazardDamage * bmrMultiplier * (rules.biomeHazardDamageScale || 1.0) * frameScale);
      }

      // Grazing Spores
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
            agent.energy = Math.min(agent.phenotype.stomachCapacity, agent.energy + baseEnergy * herbivoreEfficiency * (rules.grazingEfficiencyHerbivoreScale || 1.25));
            agent.hasEaten = true;
          }
        }
      });

      // Fight Biting
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
          }
        });
      }

      // Mitosis Reproduction
      const canReproduce = agent.age >= Math.max(rules.creatureMatureAgeFloor || 600, agent.phenotype.matureAge) && agent.hasEaten;
      const reachedReproThreshold = agent.energy >= agent.phenotype.stomachCapacity * Math.max(rules.reproductionStomachThresholdFloor || 0.65, agent.phenotype.reproThreshold);

      if (canReproduce && reachedReproThreshold && creatures.length < (cfg.maxCreatures || 45)) {
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
            }
          }
          childGenome = tempGenome;
        }

        const childMutationRate = 1.0 - agent.phenotype.repairFidelity;
        if (Math.random() < childMutationRate) {
          childGenome = mutateGenome(childGenome).newGenome;
        }

        const childAntisense = getComplementaryString(childGenome);
        const childPhenotype = parseGenome(childGenome, childAntisense, [...agent.phenotype.methylations]);

        const childPx = agent.px + Math.random() * 32 - 16;
        const childPy = agent.py + Math.random() * 32 - 16;

        newbornAgents.push({
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
        });
      }
    });

    creatures = creatures.filter(agent => !deadAgentIds.includes(agent.id));
    creatures.push(...newbornAgents);

    // RESTOCK FOUNDERS
    const targetPopulation = cfg.targetPopulation !== undefined ? cfg.targetPopulation : 25;
    while (creatures.length < targetPopulation) {
      const g = generateRandomGenome(256);
      const anti = getComplementaryString(g);
      const pheno = parseGenome(g, anti);

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
        energy: pheno.stomachCapacity * (rules.restockInitialStomachRatio !== undefined ? rules.restockInitialStomachRatio : 0.60),
        age: 0,
        generation: 1,
        adrenaline: 1.0,
        hasEaten: false,
        neuronStates: [],
        neuronActivations: []
      });
    }

    // Physical Collisions Phase
    grid.clear();
    foodPellets.forEach(pellet => grid.insertFood(pellet));
    creatures.forEach(agent => grid.insertCreature(agent));

    // Creature-to-Creature collision
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

          const totalMass = mass1 + mass2;
          const push1 = overlap * (mass2 / totalMass);
          const push2 = overlap * (mass1 / totalMass);

          agent1.px -= nx * push1;
          agent1.py -= ny * push1;
          agent2.px += nx * push2;
          agent2.py += ny * push2;

          if (agent1.px < r1) agent1.px = r1;
          else if (agent1.px > logicalWidth - r1) agent1.px = logicalWidth - r1;
          if (agent1.py < r1) agent1.py = r1;
          else if (agent1.py > logicalHeight - r1) agent1.py = logicalHeight - r1;

          if (agent2.px < r2) agent2.px = r2;
          else if (agent2.px > logicalWidth - r2) agent2.px = logicalWidth - r2;
          if (agent2.py < r2) agent2.py = r2;
          else if (agent2.py > logicalHeight - r2) agent2.py = logicalHeight - r2;

          const rvx = agent2.vx - agent1.vx;
          const rvy = agent2.vy - agent1.vy;
          const vnorm = rvx * nx + rvy * ny;

          if (vnorm < 0) {
            const e = 0.5;
            const J = -(1 + e) * vnorm / ((1 / mass1) + (1 / mass2));
            agent1.vx -= (J * nx) / mass1;
            agent1.vy -= (J * ny) / mass1;
            agent2.vx += (J * nx) / mass2;
            agent2.vy += (J * ny) / mass2;
          }
        }
      }
    }

    // Creature-to-Food collision
    creatures.forEach(agent => {
      const r = agent.phenotype.spinalHarmonics.meanRadius;
      const nearbyFood = grid.getNearbyFood(agent.px, agent.py, r + 20);
      nearbyFood.forEach(pellet => {
        const dx = pellet.x - agent.px;
        const dy = pellet.y - agent.py;
        const d = Math.sqrt(dx * dx + dy * dy);
        const minDist = r + 8;

        if (d < minDist && d > 0.1) {
          const overlap = minDist - d;
          const nx = dx / d;
          const ny = dy / d;

          pellet.x += nx * overlap;
          pellet.y += ny * overlap;

          if (pellet.x < 8) pellet.x = 8;
          else if (pellet.x > logicalWidth - 8) pellet.x = logicalWidth - 8;
          if (pellet.y < 8) pellet.y = 8;
          else if (pellet.y > logicalHeight - 8) pellet.y = logicalHeight - 8;

          pellet.vx = agent.vx + nx * 2.0;
          pellet.vy = agent.vy + ny * 2.0;
        }
      });
    });

    creatures.forEach(c => {
      highestGeneration = Math.max(highestGeneration, c.generation);
    });
  }

  const extinct = creatures.length === 0;
  const extinctPenalty = extinct ? 10000 : 0;

  // Stability scoring: target is around 25 population. Large fluctuations are penalized
  const finalPop = creatures.length;
  const popDeviation = Math.abs(finalPop - cfg.targetPopulation);

  // Lifespan score: we want a good amount of old age deaths (senescence) instead of purely hunger starvation
  const hungerRatio = totalDeaths > 0 ? starvationDeaths / totalDeaths : 1.0;
  const averageDeadAge = totalDeaths > 0 ? totalDeadAge / totalDeaths : 0;

  // Evolutionary Golden Ratio: we want healthy selective pressure.
  // - Hunger ratio should be between 50% and 75% (representing selective pressure).
  // - Old age ratio should be between 25% and 50% (representing high survival capacity).
  // - We also want higher generations (meaning active evolution!).
  let evolutionaryBonus = 0;
  if (hungerRatio >= 0.50 && hungerRatio <= 0.75) {
    evolutionaryBonus += 15.0; // Perfect selective pressure!
  } else {
    // Penalty for extreme starvation or extreme lack of selective pressure
    evolutionaryBonus -= Math.abs(hungerRatio - 0.62) * 20.0;
  }

  // Generation bonus (evolutionary progression)
  const genBonus = Math.min(25.0, highestGeneration * 5.0);

  // Let's formulate a highly nuanced score:
  let score = 100.0 - popDeviation * 2.5 + evolutionaryBonus + genBonus + (averageDeadAge / rules.creatureMaxAgeTicks) * 15.0;
  score = Math.max(0, score - extinctPenalty);

  return {
    success: score > 50 && !extinct,
    score,
    finalPopulation: finalPop,
    finalSpores: foodPellets.length,
    starvationDeaths,
    oldAgeDeaths,
    avgLifespan: averageDeadAge,
    maxGeneration: highestGeneration,
    extinct
  };
}

// --------------------------------------------------------------------------
// Command Line Interface & Fuzzy Tuning Engine
// --------------------------------------------------------------------------
function main() {
  const args = process.argv.slice(2);
  const isTuneMode = args.includes("--tune");

  let cfg = loadConfig();

  if (!isTuneMode) {
    console.log("==========================================================================");
    console.log("🧬 Starting Single Decoupled Ecosystem Balancing Run...");
    console.log(`World Seed: ${cfg.seed} | Target Pop: ${cfg.targetPopulation} | Spores: ${cfg.foodSporeCount}`);
    console.log("==========================================================================");

    const start = Date.now();
    const result = runSimulation(cfg, 2000); // 2k Ticks = ~33 seconds sim
    const elapsed = Date.now() - start;

    console.log(`Simulation finished in ${elapsed}ms! (${Math.round(2000 / (elapsed / 1000))} Ticks/sec)`);
    console.log("--------------------------------------------------------------------------");
    console.log(`Ecosystem Status:   ${result.extinct ? "💀 EXTINCT" : "🌿 HEALTHY"}`);
    console.log(`Stability Score:    ${result.score.toFixed(2)} / 100`);
    console.log(`Final Population:   ${result.finalPopulation} creatures (target: ${cfg.targetPopulation})`);
    console.log(`Final Spores:       ${result.finalSpores} food pellets`);
    console.log(`Starvation Deaths:  ${result.starvationDeaths} cells (${Math.round(result.starvationDeaths / (result.starvationDeaths + result.oldAgeDeaths) * 100 || 0)}%)`);
    console.log(`Senescence Deaths:  ${result.oldAgeDeaths} cells (${Math.round(result.oldAgeDeaths / (result.starvationDeaths + result.oldAgeDeaths) * 100 || 0)}%)`);
    console.log(`Average Lifespan:   ${Math.round(result.avgLifespan)} ticks (${Math.round(result.avgLifespan / 60)} seconds)`);
    console.log(`Max Gen Reached:    Gen ${result.maxGeneration}`);
    console.log("==========================================================================");
  } else {
    console.log("==========================================================================");
    console.log("⚡ Starting Autonomous Fuzzy Balancing Tuning...");
    console.log("==========================================================================");

    let bestConfig = JSON.parse(JSON.stringify(cfg));
    let bestResult = runSimulation(bestConfig, 4000);
    console.log(`Initial Score: ${bestResult.score.toFixed(2)} | Avg Lifespan: ${Math.round(bestResult.avgLifespan)} ticks`);

    let iterations = 50;
    let successfulTuning = false;

    for (let iter = 1; iter <= iterations; iter++) {
      // Perturb rules slightly
      const tempConfig = JSON.parse(JSON.stringify(bestConfig));
      const r = tempConfig.rules;

      // Small random mutations to metabolic parameters
      if (Math.random() < 0.4) {
        // Mutate ground BMR scale slightly
        r.bmrBaseScale = Math.max(0.001, Math.min(0.015, r.bmrBaseScale + (Math.random() * 0.001 - 0.0005)));
      }
      if (Math.random() < 0.4) {
        // Mutate maximum age
        r.creatureMaxAgeTicks = Math.round(Math.max(3600, Math.min(10800, r.creatureMaxAgeTicks + (Math.random() * 600 - 300))));
      }
      if (Math.random() < 0.3) {
        // Mutate grazing efficiency
        r.grazingEfficiencyHerbivoreScale = Math.max(0.8, Math.min(2.5, r.grazingEfficiencyHerbivoreScale + (Math.random() * 0.2 - 0.1)));
      }
      if (Math.random() < 0.3) {
        // Mutate photosynthesis gain
        r.photosynthesisEnergyGain = Math.max(0.05, Math.min(0.35, r.photosynthesisEnergyGain + (Math.random() * 0.05 - 0.025)));
      }
      if (Math.random() < 0.3) {
        // Mutate biome hazard scaling (lower means safer and longer lifespans!)
        r.biomeHazardDamageScale = Math.max(0.1, Math.min(1.0, r.biomeHazardDamageScale + (Math.random() * 0.1 - 0.05)));
      }
      if (Math.random() < 0.3) {
        // Mutate reproduction threshold
        r.reproductionStomachThresholdFloor = Math.max(0.45, Math.min(0.75, r.reproductionStomachThresholdFloor + (Math.random() * 0.05 - 0.025)));
      }

      const runRes = runSimulation(tempConfig, 4000);

      // Score improvement check
      if (runRes.score > bestResult.score && !runRes.extinct) {
        bestConfig = tempConfig;
        bestResult = runRes;
        successfulTuning = true;
        console.log(`[ITER ${iter}] New Best Score: ${runRes.score.toFixed(2)} | Avg Lifespan: ${Math.round(runRes.avgLifespan)} ticks | BMR: ${r.bmrBaseScale.toFixed(5)} | Age: ${r.creatureMaxAgeTicks} | Hazard: ${r.biomeHazardDamageScale.toFixed(2)}`);
      }
    }

    if (successfulTuning) {
      saveConfig(bestConfig);
      console.log("==========================================================================");
      console.log("🎉 Calibration Complete!");
      console.log("Optimized parameters successfully written back to config.json.");
      console.log(`Final Stable Score: ${bestResult.score.toFixed(2)}`);
      console.log("==========================================================================");
    } else {
      console.log("==========================================================================");
      console.log("⚠️ No better configurations found. Ecosystem remains at current values.");
      console.log("==========================================================================");
    }
  }
}

if (process.argv[1] === path.resolve(process.argv[1])) {
  main();
}
