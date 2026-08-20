import { CreatureAgent } from "./types";

/**
 * Computes all sensory inputs (Visual, Olfactory, Vibrational, Thermal, Proprioceptive, Wall warning)
 * using the leading Trainer-tuned formulas and exact SSOT matching thresholds.
 */
export function computeSensoryInputs(
  agent: CreatureAgent,
  clockVal: number,
  grid: any, // passed as SpatialGrid instance to prevent import loops
  worldWidth: number,
  worldHeight: number
): number[] {
  const K = agent.phenotype.organelles.length;
  const inputs: number[] = Array(K + 1).fill(0.0);
  inputs[K] = clockVal;

  agent.phenotype.organelles.forEach((patch, idx) => {
    const range = patch.scale * 550.0; // Trainer leading visual reach
    const alpha = (patch.angle - 90) * (Math.PI / 180);
    const halfCone = Math.max(0.1, patch.bandwidth * 1.5); // Leading field of view cone (1.5x)
    const aff = patch.spectralAffinity;
    const organPower = patch.scale * (1.1 - patch.bandwidth);

    let maxStimulus = 0.0;

    // A. Food / Algae Scan (Herbivore sensors)
    const nearbyFood = grid.getNearbyFood(agent.px, agent.py, range);
    nearbyFood.forEach((pellet: any) => {
      const dx = pellet.x - agent.px;
      const dy = pellet.y - agent.py;
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
            // Olfactory/Smell Scan: chlorophyll plant scent is at 0.35, perfectly in the olfactory range [0.25, 0.65]!
            match = Math.max(0, 1.0 - Math.abs(aff - 0.35) / (patch.bandwidth * 1.8 + 0.12));
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

    // B. Peer / Prey Scan (Carnivore sensors - detecting other active creatures)
    const nearbyPeers = grid.getNearbyCreatures(agent.px, agent.py, range);
    nearbyPeers.forEach((other: any) => {
      if (other.id === agent.id) return;
      const dx = other.px - agent.px;
      const dy = other.py - agent.py;
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
      if (agent.px < wallWarningZone) boundaryPressure = 1.0 - agent.px / wallWarningZone;
      else if (agent.px > worldWidth - wallWarningZone) boundaryPressure = 1.0 - (worldWidth - agent.px) / wallWarningZone;
      if (agent.py < wallWarningZone) boundaryPressure = Math.max(boundaryPressure, 1.0 - agent.py / wallWarningZone);
      else if (agent.py > worldHeight - wallWarningZone) boundaryPressure = Math.max(boundaryPressure, 1.0 - (worldHeight - agent.py) / wallWarningZone);

      if (boundaryPressure > 0.0) {
        maxStimulus = Math.max(maxStimulus, boundaryPressure * organPower);
      }
    }

    // D. Proprioceptive centrifugal touch
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
