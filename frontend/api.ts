import { invoke } from "@tauri-apps/api/core";

export async function safeInvoke(cmd: string, args?: any): Promise<any> {
  try {
    if (typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__ && typeof invoke !== "undefined") {
      return await invoke(cmd, args);
    }
  } catch (e) {
    console.warn(`[Tauri API] Bypassed or failed native command '${cmd}':`, e);
  }
  return Promise.resolve(null);
}

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
      const pheno = await safeInvoke("get_fossil_phenotype", { genome });
      if (pheno) {
        cachePhenotype(genome, pheno);
      }
      return pheno;
    } catch (e) {
      console.error("[Tauri API] Error loading phenotype from Rust backend:", e);
      return null;
    } finally {
      pendingPhenotypes.delete(genome);
    }
  })();
  pendingPhenotypes.set(genome, request);
  return request;
}
