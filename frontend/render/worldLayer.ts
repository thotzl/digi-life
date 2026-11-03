// World drawing shared by the Ocean and the trainer tiles (TCK-160): biome background, current vents, rock
// obstacles and food spores. Geometry and size come from the backend world (`INIT_STATE.world`); nothing here is
// generated or hard-coded per world.
import type { FoodSpore } from "../../bindings/FoodSpore";
import type { ProceduralWorld } from "../../bindings/ProceduralWorld";

/** Spore type id of meat (plants are type 1). */
export const MEAT_SPORE = 2;

export function sporeColor(spore: FoodSpore): string {
  return spore.typeId === MEAT_SPORE ? "#ef4444" : "#10b981";
}

export type SporeStyle =
  /** Ocean: one disc with a shadow-blur glow */
  | { kind: "glow"; radius: number }
  /** Trainer tiles: a translucent halo instead of a blur pass (cheaper with many tiles) */
  | { kind: "halo"; radius: number; haloRadius: number };

export function drawSpores(ctx: CanvasRenderingContext2D, spores: readonly FoodSpore[], style: SporeStyle) {
  for (const spore of spores) {
    const color = sporeColor(spore);
    ctx.fillStyle = color;
    if (style.kind === "glow") {
      ctx.beginPath();
      ctx.arc(spore.x, spore.y, style.radius, 0, Math.PI * 2);
      ctx.shadowBlur = 8;
      ctx.shadowColor = color;
      ctx.fill();
      ctx.shadowBlur = 0;
    } else {
      ctx.globalAlpha = 0.25;
      ctx.beginPath();
      ctx.arc(spore.x, spore.y, style.haloRadius, 0, Math.PI * 2);
      ctx.fill();
      ctx.globalAlpha = 1;
      ctx.beginPath();
      ctx.arc(spore.x, spore.y, style.radius, 0, Math.PI * 2);
      ctx.fill();
    }
  }
}

/** Static Ocean world: biome image (one pixel per biome cell), vents and obstacles, drawn in world coordinates. */
export class WorldLayer {
  private biomeImage: HTMLCanvasElement | null = null;
  private world: ProceduralWorld | null = null;

  get width(): number | null {
    return this.world?.width ?? null;
  }

  get height(): number | null {
    return this.world?.height ?? null;
  }

  setWorld(world: ProceduralWorld) {
    this.world = world;
    this.biomeImage = null;
    const cell = world.biomes[0];
    if (!cell || typeof document === "undefined") return;
    // Biome cells are a regular grid of `cell.width` x `cell.height` world units (`generate_world`)
    const cols = Math.max(1, Math.round(world.width / cell.width));
    const rows = Math.max(1, Math.round(world.height / cell.height));
    const image = document.createElement("canvas");
    image.width = cols;
    image.height = rows;
    const ctx = image.getContext("2d");
    if (!ctx) return;
    for (const biome of world.biomes) {
      ctx.fillStyle = biome.color;
      ctx.fillRect(Math.floor(biome.x / cell.width), Math.floor(biome.y / cell.height), 1, 1);
    }
    this.biomeImage = image;
  }

  /** Draws biomes, vents and obstacles; the context is in world coordinates. */
  draw(ctx: CanvasRenderingContext2D) {
    const world = this.world;
    if (!world) return;
    if (this.biomeImage) {
      ctx.drawImage(this.biomeImage, 0, 0, world.width, world.height);
    }
    for (const vent of world.vents) {
      ctx.beginPath();
      ctx.arc(vent.x, vent.y, vent.radius, 0, Math.PI * 2);
      ctx.fillStyle = "rgba(239, 68, 68, 0.035)";
      ctx.fill();
    }
    for (const obs of world.obstacles) {
      if (obs.vertices.length === 0) continue;
      ctx.beginPath();
      ctx.moveTo(obs.vertices[0].x, obs.vertices[0].y);
      for (let j = 1; j < obs.vertices.length; j++) {
        ctx.lineTo(obs.vertices[j].x, obs.vertices[j].y);
      }
      ctx.closePath();
      ctx.fillStyle = obs.color;
      ctx.fill();
      ctx.strokeStyle = "#0b1329";
      ctx.lineWidth = 14;
      ctx.stroke();
    }
  }
}

/** Trainer chamber background: radial gradient and a 100-unit grid over a square room of `size` world units. */
export function drawChamber(ctx: CanvasRenderingContext2D, size: number) {
  const c = size / 2.0;
  const grad = ctx.createRadialGradient(c, c, 0, c, c, size * 0.7);
  grad.addColorStop(0, "#0b1329");
  grad.addColorStop(1, "#020617");
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, size, size);

  ctx.strokeStyle = "rgba(0, 242, 254, 0.025)";
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  for (let x = 0; x <= size; x += 100) {
    ctx.moveTo(x, 0);
    ctx.lineTo(x, size);
  }
  for (let y = 0; y <= size; y += 100) {
    ctx.moveTo(0, y);
    ctx.lineTo(size, y);
  }
  ctx.stroke();
}
