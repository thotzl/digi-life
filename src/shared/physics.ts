import { CreatureAgent } from "./types";
import { generateRandomGenome, parseGenome, getComplementaryString } from "../biology/dna";

/**
 * Physically Equipped, Neurally Naive (PE-NN) Progenitor Factory
 */
export function generatePEN_Progenitor(): string {
  let attempts = 0;
  while (attempts < 1500) {
    attempts++;
    const genome = generateRandomGenome(256);
    const phenotype = parseGenome(genome, getComplementaryString(genome));
    
    // Body size must be balanced (Radius 16 to 28)
    const r = phenotype.spinalHarmonics.meanRadius;
    if (r < 16 || r > 28) continue;

    // Must have exactly 1 or 2 forward-facing algae food chemoreceptors / photoreceptors
    // Angle 0 is straight forward, 90 is right, 270/315 is left.
    // Ensure both left and right forward quadrants are matched to prevent clockwise bias!
    const foodSensors = phenotype.organelles.filter(patch => {
      const angle = patch.angle; // 0° to 360°
      const isForward = angle <= 45 || angle >= 315; // Symmetric front quadrant
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

/**
 * Applies biomorphic flexion kinematics, movement thrust, fluid drag, 
 * wall boundary bounces, and obstacle collisions to a creature agent.
 */
export function applyCreaturePhysics(
  agent: CreatureAgent,
  netThrustForce: number,
  outBending: number,
  worldWidth: number,
  worldHeight: number,
  checkObstacleCollisionFn?: (px: number, py: number, r: number) => { collided: boolean; normalX: number; normalY: number; overlap: number }
): { hitWall: boolean } {
  const pheno = agent.phenotype;
  const meanRadius = pheno.spinalHarmonics.meanRadius;
  const stiffness = pheno.stiffness;
  const mass = 1.0 + (meanRadius ** 2) * 0.01;

  // 1. Biomorphic flexion (body bending) steering kinematics:
  const maxFlexion = 1.2; // approx 68 degrees max bend
  const targetBending = outBending * (maxFlexion / Math.max(0.2, stiffness));

  // Smooth muscle stiffness body bending interpolation (stiffness-modulated leading model)
  agent.bendAngle = (agent.bendAngle || 0.0) * (1.0 - stiffness * 0.3) + targetBending * (stiffness * 0.3);
  agent.headingAngle += agent.omegaRot;
  agent.omegaRot = agent.bendAngle / 12.0;

  // 2. Fluid drag & thrust forces
  const vForward = agent.vx * Math.cos(agent.headingAngle) + agent.vy * Math.sin(agent.headingAngle);
  const fx = netThrustForce * Math.cos(agent.headingAngle);
  const fy = netThrustForce * Math.sin(agent.headingAngle);

  const receptorBallast = pheno.organelles.length * 0.18;
  const dragForward = (meanRadius * 0.015 + receptorBallast) * (1.0 - stiffness * 0.3);

  const dragForceForward = -dragForward * vForward;
  const ax = (fx + dragForceForward * Math.cos(agent.headingAngle)) / mass;
  const ay = (fy + dragForceForward * Math.sin(agent.headingAngle)) / mass;

  agent.vx = (agent.vx + ax) * 0.94;
  agent.vy = (agent.vy + ay) * 0.94;

  // Lock-on heading movement (No slip!)
  const netSpeed = agent.vx * Math.cos(agent.headingAngle) + agent.vy * Math.sin(agent.headingAngle);
  agent.vx = netSpeed * Math.cos(agent.headingAngle);
  agent.vy = netSpeed * Math.sin(agent.headingAngle);

  // Integrate position
  agent.px += agent.vx;
  agent.py += agent.vy;

  // 3. Wall boundary collisions (Bounces with Trainer-standard 50% restitution)
  const r = meanRadius;
  const wallRestitution = 0.5; // 50% kinetic preservation
  let hitWall = false;

  if (agent.px < r) {
    agent.px = r;
    agent.vx = -Math.abs(agent.vx) * wallRestitution;
    hitWall = true;
  } else if (agent.px > worldWidth - r) {
    agent.px = worldWidth - r;
    agent.vx = -Math.abs(agent.vx) * wallRestitution;
    hitWall = true;
  }

  if (agent.py < r) {
    agent.py = r;
    agent.vy = -Math.abs(agent.vy) * wallRestitution;
    hitWall = true;
  } else if (agent.py > worldHeight - r) {
    agent.py = worldHeight - r;
    agent.vy = -Math.abs(agent.vy) * wallRestitution;
    hitWall = true;
  }

  // 4. Spatial obstacles collisions
  if (checkObstacleCollisionFn) {
    const col = checkObstacleCollisionFn(agent.px, agent.py, r);
    if (col.collided) {
      agent.px += col.normalX * col.overlap;
      agent.py += col.normalY * col.overlap;
      const dot = agent.vx * col.normalX + agent.vy * col.normalY;
      // Abprall an Felsen mit 45% Schwungverbleib (Ozean/Trainer standard)
      agent.vx = (agent.vx - 2.0 * dot * col.normalX) * 0.45;
      agent.vy = (agent.vy - 2.0 * dot * col.normalY) * 0.45;
    }
  }

  return { hitWall };
}

/**
 * Resolves physical push-impulses and boundaries for food spores when a creature collides with them.
 */
export function resolveSporeCollision(
  agent: CreatureAgent,
  pellet: any,
  worldWidth: number,
  worldHeight: number
): void {
  const meanRadius = agent.phenotype.spinalHarmonics.meanRadius;
  const dx = pellet.x - agent.px;
  const dy = pellet.y - agent.py;
  const d = Math.sqrt(dx * dx + dy * dy);
  const minDist = meanRadius + 8; // Spore radius = 8

  if (d < minDist && d > 0.1) {
    const overlap = minDist - d;
    const nx = dx / d;
    const ny = dy / d;

    // Push spore away
    pellet.x += nx * overlap;
    pellet.y += ny * overlap;

    // Clip spore to boundaries
    if (pellet.x < 8) pellet.x = 8;
    else if (pellet.x > worldWidth - 8) pellet.x = worldWidth - 8;
    if (pellet.y < 8) pellet.y = 8;
    else if (pellet.y > worldHeight - 8) pellet.y = worldHeight - 8;

    // Impart velocity push
    pellet.vx = agent.vx + nx * 2.0;
    pellet.vy = agent.vy + ny * 2.0;
  }
}
