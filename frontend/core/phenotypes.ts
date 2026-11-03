// Phenotype cache: decoded phenotypes per genome from `get_fossil_phenotype` (moved from api.ts, TCK-157)
import { signal } from "@preact/signals-core";

import type { CreaturePhenotype } from "../../bindings/CreaturePhenotype";
import { call } from "./ipc";

export const phenotypeCache = new Map<string, CreaturePhenotype>();
/** Bumped whenever a phenotype enters the cache; lists that colour by phenotype re-render on it. */
export const phenotypeVersion = signal(0);
// Bounded: a long trainer run produces new genomes every generation
const PHENOTYPE_CACHE_LIMIT = 512;
// One request per genome: renderers ask every telemetry frame until the first answer arrives
const pendingPhenotypes = new Map<string, Promise<CreaturePhenotype | null>>();
// A failed genome is not asked again before this delay (renderers ask every frame; TCK-160)
export const PHENOTYPE_RETRY_MS = 10_000;
const failedPhenotypes = new Map<string, number>();

function cachePhenotype(genome: string, pheno: CreaturePhenotype) {
  if (phenotypeCache.size >= PHENOTYPE_CACHE_LIMIT && !phenotypeCache.has(genome)) {
    phenotypeCache.delete(phenotypeCache.keys().next().value as string);
  }
  phenotypeCache.set(genome, pheno);
  phenotypeVersion.value++;
}

/** Cached phenotype of `genome`, or `undefined` (then a request is started unless one failed recently). */
export function peekPhenotype(genome: string): CreaturePhenotype | undefined {
  const cached = phenotypeCache.get(genome);
  if (!cached) void getPhenotype(genome);
  return cached;
}

/** Phenotype of `genome` (cached); `null` if the backend call failed (already reported through the IPC error hook)
 * or failed less than `PHENOTYPE_RETRY_MS` ago. */
export async function getPhenotype(genome: string, now = Date.now()): Promise<CreaturePhenotype | null> {
  const cached = phenotypeCache.get(genome);
  if (cached) return cached;
  const pending = pendingPhenotypes.get(genome);
  if (pending) return pending;
  const failedAt = failedPhenotypes.get(genome);
  if (failedAt !== undefined && now - failedAt < PHENOTYPE_RETRY_MS) return null;
  const request = (async () => {
    try {
      const pheno = await call("get_fossil_phenotype", { genome });
      failedPhenotypes.delete(genome);
      cachePhenotype(genome, pheno);
      return pheno;
    } catch {
      failedPhenotypes.set(genome, now);
      return null;
    } finally {
      pendingPhenotypes.delete(genome);
    }
  })();
  pendingPhenotypes.set(genome, request);
  return request;
}
