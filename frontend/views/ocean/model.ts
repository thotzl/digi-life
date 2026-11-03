// Ocean client model (TCK-160): the local copy of creatures and spores built from backend events, plus the pure
// helpers of the Ocean view (hit testing, bite flashes). No DOM here, so it is unit-tested.
import type { CreatureAgent } from "../../../bindings/CreatureAgent";
import type { CreaturePhenotype } from "../../../bindings/CreaturePhenotype";
import type { FoodSpore } from "../../../bindings/FoodSpore";
import type { OceanInitState } from "../../../bindings/OceanInitState";
import type { OceanTelemetryTick } from "../../../bindings/OceanTelemetryTick";

/** Click radius around a creature centre (world units). */
export const CLICK_RADIUS = 120;
/** Frames a bite flash stays visible. */
export const BITE_FLASH_FRAMES = 15;

/** Hover radius of a creature body (world units), from its own phenotype. */
export function hitRadius(phenotype: CreaturePhenotype): number {
  return phenotype.spinalHarmonics.meanRadius * 2.6 * 0.5 + 10;
}

export function isOverCreature(creatures: readonly CreatureAgent[], x: number, y: number): boolean {
  return creatures.some((c) => Math.hypot(c.px - x, c.py - y) < hitRadius(c.phenotype));
}

/** Nearest creature within `CLICK_RADIUS` of (x, y), or `null`. */
export function creatureAt(creatures: readonly CreatureAgent[], x: number, y: number): CreatureAgent | null {
  let best: CreatureAgent | null = null;
  let bestDist = CLICK_RADIUS;
  for (const c of creatures) {
    const d = Math.hypot(c.px - x, c.py - y);
    if (d < bestDist) {
      bestDist = d;
      best = c;
    }
  }
  return best;
}

/** A bite flash at a world position; drawn inside the camera transform like the creatures. */
export interface BiteFlash {
  x: number;
  y: number;
  age: number;
}

export function biteFlash(event: { x: number; y: number }): BiteFlash {
  return { x: event.x, y: event.y, age: 0 };
}

/** Ages the flashes by one frame and drops the expired ones (in place). */
export function ageFlashes(flashes: BiteFlash[]): void {
  for (let i = flashes.length - 1; i >= 0; i--) {
    flashes[i].age += 1;
    if (flashes[i].age > BITE_FLASH_FRAMES) flashes.splice(i, 1);
  }
}

export class OceanModel {
  creatures: CreatureAgent[] = [];
  foodPellets: FoodSpore[] = [];
  highestGeneration = 1;

  applyInit(data: OceanInitState) {
    this.highestGeneration = data.highestGeneration;
    this.foodPellets = data.foodPellets;
    this.creatures = [...data.creatures];
  }

  /** Adds newborns, moves the known creatures in place (they keep their phenotype) and drops the dead. */
  applyTelemetry(tick: OceanTelemetryTick) {
    this.highestGeneration = tick.highestGeneration;
    for (const born of tick.newCreatures) {
      if (!this.creatures.some((c) => c.id === born.id)) this.creatures.push(born);
    }
    const byId = new Map(this.creatures.map((c) => [c.id, c]));
    const alive: CreatureAgent[] = [];
    for (const tele of tick.creatures) {
      const local = byId.get(tele.id);
      if (!local) continue;
      local.px = tele.px;
      local.py = tele.py;
      local.vx = tele.vx;
      local.vy = tele.vy;
      local.headingAngle = tele.headingAngle;
      local.omegaRot = tele.omegaRot;
      local.energy = tele.energy;
      local.age = tele.age;
      local.generation = tele.generation;
      local.hasEaten = tele.hasEaten;
      alive.push(local);
    }
    // Keep the local order (draw order) of the survivors
    const aliveIds = new Set(alive.map((c) => c.id));
    this.creatures = this.creatures.filter((c) => aliveIds.has(c.id));
  }

  find(id: number): CreatureAgent | undefined {
    return this.creatures.find((c) => c.id === id);
  }
}
