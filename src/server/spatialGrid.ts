import { CreatureAgent, FoodSpore } from "../shared/types";

export class SpatialGrid {
  public readonly cellSize = 80;
  public readonly cols = 240; // 19200 / 80
  public readonly rows = 135; // 10800 / 80

  private creatureBuckets: CreatureAgent[][];
  private foodBuckets: FoodSpore[][];

  constructor() {
    const totalCells = this.cols * this.rows;
    this.creatureBuckets = Array.from({ length: totalCells }, () => []);
    this.foodBuckets = Array.from({ length: totalCells }, () => []);
  }

  public clear() {
    const totalCells = this.cols * this.rows;
    for (let i = 0; i < totalCells; i++) {
      this.creatureBuckets[i].length = 0;
      this.foodBuckets[i].length = 0;
    }
  }

  private getCellIndex(x: number, y: number): number {
    const c = Math.max(0, Math.min(this.cols - 1, Math.floor(x / this.cellSize)));
    const r = Math.max(0, Math.min(this.rows - 1, Math.floor(y / this.cellSize)));
    return c * this.rows + r;
  }

  public insertCreature(agent: CreatureAgent) {
    const idx = this.getCellIndex(agent.px, agent.py);
    this.creatureBuckets[idx].push(agent);
  }

  public insertFood(pellet: FoodSpore) {
    const idx = this.getCellIndex(pellet.x, pellet.y);
    this.foodBuckets[idx].push(pellet);
  }

  /**
   * Returns all creatures within a bounding box centered at (x, y) with a given range.
   */
  public getNearbyCreatures(x: number, y: number, range: number): CreatureAgent[] {
    const startCol = Math.max(0, Math.floor((x - range) / this.cellSize));
    const endCol = Math.min(this.cols - 1, Math.floor((x + range) / this.cellSize));
    const startRow = Math.max(0, Math.floor((y - range) / this.cellSize));
    const endRow = Math.min(this.rows - 1, Math.floor((y + range) / this.cellSize));

    const result: CreatureAgent[] = [];
    for (let c = startCol; c <= endCol; c++) {
      for (let r = startRow; r <= endRow; r++) {
        const idx = c * this.rows + r;
        const bucket = this.creatureBuckets[idx];
        for (let i = 0; i < bucket.length; i++) {
          result.push(bucket[i]);
        }
      }
    }
    return result;
  }

  /**
   * Returns all food pellets within a bounding box centered at (x, y) with a given range.
   */
  public getNearbyFood(x: number, y: number, range: number): FoodSpore[] {
    const startCol = Math.max(0, Math.floor((x - range) / this.cellSize));
    const endCol = Math.min(this.cols - 1, Math.floor((x + range) / this.cellSize));
    const startRow = Math.max(0, Math.floor((y - range) / this.cellSize));
    const endRow = Math.min(this.rows - 1, Math.floor((y + range) / this.cellSize));

    const result: FoodSpore[] = [];
    for (let c = startCol; c <= endCol; c++) {
      for (let r = startRow; r <= endRow; r++) {
        const idx = c * this.rows + r;
        const bucket = this.foodBuckets[idx];
        for (let i = 0; i < bucket.length; i++) {
          result.push(bucket[i]);
        }
      }
    }
    return result;
  }
}
