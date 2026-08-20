import { CreatureAgent } from "./types";

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
