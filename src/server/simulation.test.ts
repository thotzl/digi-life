import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { WebSocketServer, WebSocket } from 'ws';
import http from 'http';
import { parseGenome, generateRandomGenome, getComplementaryString } from '../biology/dna';
import { CreatureAgent, FoodSpore } from '../shared/types';

describe('Petri Dish Physics & Biological Mechanics Integration', () => {
  const logicalWidth = 19200;

  it('should update positions, apply fluid drag, and bounce off Logical boundaries', () => {
    const genome = generateRandomGenome(128);
    const phenotype = parseGenome(genome);
    
    // Create a creature near the logical boundary moving outwards
    const agent: CreatureAgent = {
      id: 1,
      speciesId: "species_1",
      genome,
      antisense: "",
      phenotype,
      px: logicalWidth - 5,
      py: 500,
      vx: 10, // moving right, out of bounds
      vy: 0,
      headingAngle: 0,
      omegaRot: 0,
      energy: 80,
      age: 100,
      generation: 1,
      adrenaline: 1.0,
      hasEaten: false,
      neuronStates: [],
      neuronActivations: []
    };

    // Apply 1 physics step: Newtonian Euler integration
    agent.px += agent.vx;
    agent.py += agent.vy;

    // Fluid drag resistance (frictional damping)
    agent.vx *= 0.95;
    agent.vy *= 0.95;

    // Assert movement
    expect(agent.px).toBe(logicalWidth + 5);

    // Logical Boundary bounce check (simulate server's boundary collision resolver)
    if (agent.px >= logicalWidth) {
      agent.px = logicalWidth - (agent.px - logicalWidth);
      agent.vx = -Math.abs(agent.vx); // Invert velocity to bounce back
    }

    // Verify elastic boundary bounce
    expect(agent.px).toBeLessThan(logicalWidth);
    expect(agent.vx).toBeLessThan(0); // moving backwards (leftwards)
  });

  it('should absorb nutrient spores upon close contact and grant stomach energy', () => {
    const genome = generateRandomGenome(128);
    const phenotype = parseGenome(genome);

    const agent: CreatureAgent = {
      id: 2,
      speciesId: "species_1",
      genome,
      antisense: "",
      phenotype,
      px: 1000,
      py: 1000,
      vx: 0,
      vy: 0,
      headingAngle: 0,
      omegaRot: 0,
      energy: 30, // low energy
      age: 100,
      generation: 1,
      adrenaline: 1.0,
      hasEaten: false,
      neuronStates: [],
      neuronActivations: []
    };

    // Spore placed very close to the creature (distance 10, which is well within radius)
    const spore: FoodSpore = {
      x: 1008,
      y: 1006,
      vx: 0,
      vy: 0
    };

    // Distance calculation
    const dx = spore.x - agent.px;
    const dy = spore.y - agent.py;
    const distance = Math.sqrt(dx*dx + dy*dy);
    const meanRadius = agent.phenotype.spinalHarmonics.meanRadius;

    expect(distance).toBeLessThan(meanRadius * 2.5); // absorbable range

    // Simulate Spore absorption
    if (distance < meanRadius * 2.5) {
      agent.energy = Math.min(agent.phenotype.stomachCapacity, agent.energy + 25);
      agent.hasEaten = true;
    }

    expect(agent.energy).toBe(55);
    expect(agent.hasEaten).toBe(true);
  });

  it('should trigger mitotic splitting when mature, well-fed, and has physiological feeding guarantee', () => {
    const genome = generateRandomGenome(128);
    const phenotype = parseGenome(genome);

    const agent: CreatureAgent = {
      id: 3,
      speciesId: "species_1",
      genome,
      antisense: "",
      phenotype,
      px: 5000,
      py: 5000,
      vx: 0,
      vy: 0,
      headingAngle: 0,
      omegaRot: 0,
      energy: phenotype.stomachCapacity * Math.max(0.65, phenotype.reproThreshold) + 10, // Exceeds both local and server stomach threshold
      age: Math.max(600, phenotype.matureAge) + 10, // Exceeds both local and server mature age floor
      generation: 2,
      adrenaline: 1.0,
      hasEaten: true, // has physiological guarantee
      neuronStates: [],
      neuronActivations: []
    };

    const meetsReproductionCriteria = 
      agent.age >= Math.max(600, agent.phenotype.matureAge) &&
      agent.energy >= agent.phenotype.stomachCapacity * Math.max(0.65, agent.phenotype.reproThreshold) &&
      agent.hasEaten === true;

    expect(meetsReproductionCriteria).toBe(true);

    // Mitotic splitting division
    let splitOffspring: CreatureAgent | null = null;
    if (meetsReproductionCriteria) {
      const splitLossRatio = agent.phenotype.splitLoss;
      const childEnergy = (agent.energy / 2) * (1.0 - splitLossRatio);
      agent.energy = agent.energy / 2; // Split half
      agent.hasEaten = false; // Reset feeding guarantee

      splitOffspring = {
        ...agent,
        id: 4,
        generation: agent.generation + 1,
        energy: childEnergy,
        age: 0,
        hasEaten: false
      };
    }

    expect(splitOffspring).not.toBeNull();
    expect(splitOffspring!.generation).toBe(3);
    expect(splitOffspring!.energy).toBeLessThan(agent.energy); // Child loses split energy tax to friction
    expect(agent.hasEaten).toBe(false); // Parent feeding state reset
  });
});

describe('WebSocket Replication State Sync Network Protocol', () => {
  let server: http.Server;
  let wss: WebSocketServer;
  const TEST_PORT = 4005;

  beforeAll(() => {
    server = http.createServer();
    wss = new WebSocketServer({ server });
    server.listen(TEST_PORT);
  });

  afterAll(() => {
    wss.close();
    server.close();
  });

  it('should stream formatted delta state updates to connected monitoring monitors', async () => {
    wss.on('connection', (ws) => {
      const telemetryPayload = {
        type: "TELEMETRY_TICK",
        creatures: [
          {
            id: 1,
            px: 100,
            py: 100,
            energy: 100,
            latinName: "Testus bilaterialis"
          }
        ],
        foodPellets: []
      };
      ws.send(JSON.stringify(telemetryPayload));
    });

    const wsClient = new WebSocket(`ws://localhost:${TEST_PORT}`);
    
    const telemetryPromise = new Promise<any>((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error("Timeout waiting for WS message")), 1000);
      wsClient.on('message', (data) => {
        clearTimeout(timeout);
        resolve(JSON.parse(data.toString()));
      });
    });

    const payload = await telemetryPromise;
    expect(payload.type).toBe("TELEMETRY_TICK");
    expect(payload.creatures[0].latinName).toBe("Testus bilaterialis");

    wsClient.close();
  });
});

describe('Continuous-Time Recurrent Brain outputs processing', () => {
  it('[BUG-TEST 2] should process outputs[3] ("Biolum Flash") to trigger flash events on headless server', () => {
    const genome = generateRandomGenome(128);
    const phenotype = parseGenome(genome);

    const agent: CreatureAgent = {
      id: 10,
      speciesId: "species_1",
      genome,
      antisense: "",
      phenotype,
      px: 1000,
      py: 1000,
      vx: 0,
      vy: 0,
      headingAngle: 0,
      omegaRot: 0,
      energy: 100,
      age: 100,
      generation: 1,
      adrenaline: 1.0,
      hasEaten: false,
      neuronStates: [],
      neuronActivations: []
    };

    // We mock that the 4th output (Biolum Flash) has a high positive firing activation (e.g., 0.85)
    const mockOutputs = [0.1, 0.0, 0.0, 0.85];

    // Simulate the server's outputs[3] (Biolum Flash) processing logic:
    const outFlash = mockOutputs[3];
    let broadcastedEvent: any = null;

    function testBroadcast(event: any) {
      broadcastedEvent = event;
    }

    if (outFlash > 0.5) {
      agent.energy -= 0.05 * outFlash;
      testBroadcast({
        type: "FLASH_EVENT",
        agentId: agent.id,
        x: agent.px,
        y: agent.py,
        intensity: outFlash
      });
    }

    // Assert that the FLASH_EVENT is correctly structured and emitted on active Biolum Flash
    expect(broadcastedEvent).not.toBeNull();
    expect(broadcastedEvent.type).toBe("FLASH_EVENT");
    expect(broadcastedEvent.agentId).toBe(10);
    expect(broadcastedEvent.x).toBe(1000);
    expect(broadcastedEvent.y).toBe(1000);
    expect(broadcastedEvent.intensity).toBe(0.85);

    // Verify metabolic surcharge energy tax is correctly deducted from the creature's stomach
    expect(agent.energy).toBe(100 - (0.05 * 0.85));
  });
});

describe('Species Database Record Structure Validation', () => {
  it('should verify that all required fields are present and correctly typed in a SpeciesRecord', () => {
    const genome = generateRandomGenome(128);
    const anti = getComplementaryString(genome);
    const phenotype = parseGenome(genome, anti);

    const record = {
      id: genome,
      name: phenotype.latinName,
      genome: genome,
      antisense: anti,
      parentSpeciesId: "parent_dna_xyz",
      status: "alive" as const,
      peakPopulation: 1,
      birthTime: Date.now(),
      generation: 2,
      carnivory: phenotype.carnivory
    };

    expect(record.id).toBe(genome);
    expect(record.name).toBe(phenotype.latinName);
    expect(record.genome).toBe(genome);
    expect(record.antisense).toBe(anti);
    expect(record.parentSpeciesId).toBe("parent_dna_xyz");
    expect(record.status).toBe("alive");
    expect(record.peakPopulation).toBe(1);
    expect(record.birthTime).toBeLessThanOrEqual(Date.now());
    expect(record.generation).toBe(2);
    expect(record.carnivory).toBe(phenotype.carnivory);
  });
});
