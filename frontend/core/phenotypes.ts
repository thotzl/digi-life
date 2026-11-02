// Phenotype cache: decoded phenotypes per genome from `get_fossil_phenotype` (moved from api.ts, TCK-157)
import { call } from "./ipc";

export const phenotypeCache = new Map<string, any>();
// Bounded: a long trainer run produces new genomes every generation
const PHENOTYPE_CACHE_LIMIT = 512;
// One request per genome: renderers ask every telemetry frame until the first answer arrives
const pendingPhenotypes = new Map<string, Promise<any>>();

function cachePhenotype(genome: string, pheno: any) {
  if (phenotypeCache.size >= PHENOTYPE_CACHE_LIMIT && !phenotypeCache.has(genome)) {
    phenotypeCache.delete(phenotypeCache.keys().next().value as string);
  }
  phenotypeCache.set(genome, pheno);
}

/** Phenotype of `genome` (cached); `null` if the backend call failed (already reported through the IPC error hook). */
export async function getPhenotype(genome: string): Promise<any> {
  if (phenotypeCache.has(genome)) {
    return phenotypeCache.get(genome);
  }
  const pending = pendingPhenotypes.get(genome);
  if (pending) {
    return pending;
  }
  const request = (async () => {
    try {
      const pheno = await call("get_fossil_phenotype", { genome });
      cachePhenotype(genome, pheno);
      return pheno;
    } catch {
      return null;
    } finally {
      pendingPhenotypes.delete(genome);
    }
  })();
  pendingPhenotypes.set(genome, request);
  return request;
}
