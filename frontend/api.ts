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

export async function getPhenotype(genome: string): Promise<any> {
  if (phenotypeCache.has(genome)) {
    return phenotypeCache.get(genome);
  }
  try {
    const pheno = await safeInvoke("get_fossil_phenotype", { genome });
    if (pheno) {
      phenotypeCache.set(genome, pheno);
    }
    return pheno;
  } catch (e) {
    console.error("[Tauri API] Error loading phenotype from Rust backend:", e);
    return null;
  }
}
