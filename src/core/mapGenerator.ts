export interface ProceduralObstacle {
  id: number;
  x: number;
  y: number;
  radius: number; // max bounding radius
  type: 'rock' | 'coral' | 'hazard_vent';
  color: string;
  vertices: { x: number; y: number; r: number; angle: number }[];
}

export interface CurrentVent {
  id: number;
  x: number;
  y: number;
  radius: number;
  forceType: 'push' | 'pull' | 'vortex';
  strength: number;
}

export interface BiomeArea {
  id: string;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  sporeSpawnRate: number;
  sporeEnergyValue: number;
  hazardDamage: number;
  color: string;
}

export interface ClimatePoint {
  biomeId: string;
  name: string;
  targetToxicity: number;
  targetViscosity: number;
  sporeSpawnRate: number;
  sporeEnergyValue: number;
  hazardDamage: number;
  color: string;
}

export interface ProceduralWorld {
  seed: string;
  width: number;
  height: number;
  obstacles: ProceduralObstacle[];
  vents: CurrentVent[];
  biomes: BiomeArea[];
}

export const CLIMATE_MATRIX: ClimatePoint[] = [
  { 
    biomeId: 'abyssal_barrens', 
    name: '💀 Abyssal Barrens',
    targetToxicity: 0.15, 
    targetViscosity: 0.15,
    sporeSpawnRate: 0.18,
    sporeEnergyValue: 30.0,
    hazardDamage: 0.0,
    color: 'rgba(15, 23, 42, 0.45)' // deep slate
  },
  { 
    biomeId: 'algae_shallows', 
    name: '🌿 Algae Shallows',
    targetToxicity: 0.20, 
    targetViscosity: 0.80,
    sporeSpawnRate: 1.40,
    sporeEnergyValue: 15.0,
    hazardDamage: 0.0,
    color: 'rgba(21, 128, 61, 0.16)' // vibrant green
  },
  { 
    biomeId: 'acid_pool', 
    name: '🧪 Sulphuric Shallows',
    targetToxicity: 0.80, 
    targetViscosity: 0.20,
    sporeSpawnRate: 0.65,
    sporeEnergyValue: 45.0,
    hazardDamage: 0.08,
    color: 'rgba(234, 179, 8, 0.14)' // sulphuric yellow
  },
  { 
    biomeId: 'cybernetic_vents', 
    name: '🔥 Cybernetic Vents',
    targetToxicity: 0.80, 
    targetViscosity: 0.80,
    sporeSpawnRate: 0.90,
    sporeEnergyValue: 50.0,
    hazardDamage: 0.04,
    color: 'rgba(168, 85, 247, 0.16)' // vent hot purple
  }
];

// --------------------------------------------------------------------------
// Seed hashing & PRNG algorithms
// --------------------------------------------------------------------------
export function hashStringToInt(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = (hash << 5) - hash + str.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
}

export function createPRNG(seed: string): () => number {
  let a = hashStringToInt(seed);
  return function() {
    let t = a += 0x6D2B79F5;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  }
}

// --------------------------------------------------------------------------
// Toroidal Fractal Noise
// --------------------------------------------------------------------------
export function sampleToroidalNoise(
  x: number, 
  y: number, 
  width: number, 
  height: number, 
  seed: string, 
  octaveOffset = ''
): number {
  const rand = createPRNG(seed + octaveOffset);
  
  let val = 0.0;
  let totalAmp = 0.0;
  let amp = 1.0;
  let freq = 1;

  // 4 Octaves of periodic toroidal sines/cosines
  for (let oct = 0; oct < 4; oct++) {
    const phaseX = rand() * Math.PI * 2;
    const phaseY = rand() * Math.PI * 2;

    const wx = (x / width) * Math.PI * 2 * freq + phaseX;
    const wy = (y / height) * Math.PI * 2 * freq + phaseY;

    val += (Math.sin(wx) * Math.cos(wy)) * amp;
    
    totalAmp += amp;
    amp *= 0.55; // Persistence
    freq *= 2;   // Lacunarity
  }

  // Normalize from [-totalAmp, totalAmp] to [0.0, 1.0]
  return (val / totalAmp) * 0.5 + 0.5;
}

// --------------------------------------------------------------------------
// Core Generator
// --------------------------------------------------------------------------
export function generateWorld(seed: string, width = 19200, height = 10800, rules?: any): ProceduralWorld {
  const rand = createPRNG(seed);

  // A. Generate Organic Biome Grid (Cell-Size 80 units - highly precise organic micro-climates!)
  const biomes: BiomeArea[] = [];
  const cellSize = 80;
  const cols = width / cellSize;
  const rows = height / cellSize;

  for (let c = 0; c < cols; c++) {
    for (let r = 0; r < rows; r++) {
      const bx = c * cellSize;
      const by = r * cellSize;

      // Sample noise at center of cell
      const tx = bx + cellSize / 2;
      const ty = by + cellSize / 2;

      const toxicity = sampleToroidalNoise(tx, ty, width, height, seed, 'toxicity');
      const viscosity = sampleToroidalNoise(tx, ty, width, height, seed, 'viscosity');

      let bestClimate = CLIMATE_MATRIX[0];
      let minDistance = Infinity;

      for (const climate of CLIMATE_MATRIX) {
        const dT = toxicity - climate.targetToxicity;
        const dV = viscosity - climate.targetViscosity;
        const distance = Math.sqrt(dT*dT + dV*dV);

        if (distance < minDistance) {
          minDistance = distance;
          bestClimate = climate;
        }
      }

      biomes.push({
        id: bestClimate.biomeId,
        name: bestClimate.name,
        x: bx,
        y: by,
        width: cellSize,
        height: cellSize,
        sporeSpawnRate: bestClimate.sporeSpawnRate,
        sporeEnergyValue: bestClimate.sporeEnergyValue,
        hazardDamage: bestClimate.hazardDamage,
        color: bestClimate.color
      });
    }
  }

  // B. Generate Circular solid obstacles (Reefs / Rock barriers) deterministically
  const obstacles: ProceduralObstacle[] = [];
  const numObstacles = 18 + Math.floor(rand() * 10);
  
  for (let i = 0; i < numObstacles; i++) {
    const radius = 180 + rand() * 260;
    const x = radius + rand() * (width - radius * 2);
    const y = radius + rand() * (height - radius * 2);

    const typeRoll = rand();
    let type: 'rock' | 'coral' = 'rock';
    let color = 'rgba(100, 116, 139, 0.8)';
    if (typeRoll > 0.6) {
      type = 'coral';
      color = 'rgba(244, 63, 94, 0.75)';
    }

    // Generate jagged, non-circular polygon vertices
    const vertices: { x: number; y: number; r: number; angle: number }[] = [];
    const numVertices = 5 + Math.floor(rand() * 4); // 5 to 8 vertices
    for (let j = 0; j < numVertices; j++) {
      const angle = (j / numVertices) * Math.PI * 2;
      const deform = 0.65 + rand() * 0.5; // [65% to 115% of base radius]
      const r = radius * deform;
      vertices.push({
        x: x + r * Math.cos(angle),
        y: y + r * Math.sin(angle),
        r,
        angle
      });
    }

    obstacles.push({ id: i + 1, x, y, radius, type, color, vertices });
  }

  // C. Generate Thermal Current Vents
  const vents: CurrentVent[] = [];
  const baseCount = (rules && rules.ventsBaseCount !== undefined) ? rules.ventsBaseCount : 6;
  const randCount = (rules && rules.ventsRandomCountRange !== undefined) ? rules.ventsRandomCountRange : 4;
  const numVents = baseCount + Math.floor(rand() * randCount);
  
  for (let i = 0; i < numVents; i++) {
    const radius = 600 + rand() * 800;
    const x = radius + rand() * (width - radius * 2);
    const y = radius + rand() * (height - radius * 2);

    const forceRoll = rand();
    let forceType: 'push' | 'pull' | 'vortex' = 'push';
    
    const forceDisabled = rules && rules.disableVentsForce === true;
    let strength = forceDisabled ? 0.0 : (0.08 + rand() * 0.16);

    if (forceRoll > 0.35 && forceRoll <= 0.70) {
      forceType = 'pull';
    } else if (forceRoll > 0.70) {
      forceType = 'vortex';
      if (!forceDisabled) {
        strength *= 1.4;
      }
    }

    vents.push({ id: i + 1, x, y, radius, forceType, strength });
  }

  return { seed, width, height, obstacles, vents, biomes };
}

// --------------------------------------------------------------------------
// Mathematical & Mechanical Physics utility functions
// --------------------------------------------------------------------------

export function getVectoredCurrentAt(world: ProceduralWorld, px: number, py: number): { vx: number; vy: number } {
  let fx = 0;
  let fy = 0;

  for (const vent of world.vents) {
    let dx = px - vent.x;
    let dy = py - vent.y;

    if (dx > world.width / 2) dx -= world.width;
    if (dx < -world.width / 2) dx += world.width;
    if (dy > world.height / 2) dy -= world.height;
    if (dy < -world.height / 2) dy += world.height;

    const dist = Math.sqrt(dx * dx + dy * dy);
    
    if (dist < vent.radius && dist > 1) {
      const factor = (1.0 - dist / vent.radius);

      if (vent.forceType === 'push') {
        fx += (dx / dist) * vent.strength * factor;
        fy += (dy / dist) * vent.strength * factor;
      } 
      else if (vent.forceType === 'pull') {
        fx -= (dx / dist) * vent.strength * factor;
        fy -= (dy / dist) * vent.strength * factor;
      } 
      else if (vent.forceType === 'vortex') {
        fx += (-dy / dist) * vent.strength * factor;
        fy += (dx / dist) * vent.strength * factor;
      }
    }
  }

  return { vx: fx, vy: fy };
}

/**
 * Highly optimized O(1) grid index biome lookup
 */
export function getBiomeAt(world: ProceduralWorld, px: number, py: number): BiomeArea | null {
  const x = (px + world.width) % world.width;
  const y = (py + world.height) % world.height;

  const cellSize = 80;
  const c = Math.floor(x / cellSize);
  const r = Math.floor(y / cellSize);
  
  const numRows = world.height / cellSize;
  const idx = c * numRows + r;

  return world.biomes[idx] || null;
}

export interface ObstacleCollisionResponse {
  collided: boolean;
  overlap: number;
  normalX: number;
  normalY: number;
}

export function checkObstacleCollision(
  world: ProceduralWorld, 
  px: number, 
  py: number, 
  radius: number
): ObstacleCollisionResponse {
  for (const obs of world.obstacles) {
    let dx = px - obs.x;
    let dy = py - obs.y;

    if (dx > world.width / 2) dx -= world.width;
    if (dx < -world.width / 2) dx += world.width;
    if (dy > world.height / 2) dy -= world.height;
    if (dy < -world.height / 2) dy += world.height;

    const dist = Math.sqrt(dx * dx + dy * dy);
    
    // Quick outer bounding sphere filter (massively optimizes performance)
    if (dist > obs.radius + radius + 150) {
      continue;
    }

    // Precise polar angle boundary interpolation
    let approachAngle = Math.atan2(dy, dx);
    if (approachAngle < 0) approachAngle += Math.PI * 2;

    let rAtAngle = obs.radius; // fallback bounding
    const numV = obs.vertices.length;

    for (let j = 0; j < numV; j++) {
      const v1 = obs.vertices[j];
      const v2 = obs.vertices[(j + 1) % numV];
      
      const a1 = v1.angle;
      let a2 = v2.angle;
      if (a2 < a1) a2 += Math.PI * 2; // wrap 2pi

      let targetAngle = approachAngle;
      if (targetAngle < a1 && j === numV - 1) targetAngle += Math.PI * 2;

      if (targetAngle >= a1 && targetAngle <= a2) {
        // Linearly interpolate the radius of the jagged polygon at this specific angle
        const t = (targetAngle - a1) / (a2 - a1);
        rAtAngle = v1.r + t * (v2.r - v1.r);
        break;
      }
    }

    const minDist = rAtAngle + radius;

    if (dist < minDist && dist > 0.1) {
      return {
        collided: true,
        overlap: minDist - dist,
        normalX: dx / dist,
        normalY: dy / dist
      };
    }
  }

  return { collided: false, overlap: 0, normalX: 0, normalY: 0 };
}
