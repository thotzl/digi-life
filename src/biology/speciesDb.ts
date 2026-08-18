export interface SpeciesRecord {
  id: string;             // Sense DNA string serves as the unique ID
  name: string;           // Procedural scientific name (Latin)
  genome: string;         // The Sense DNA
  antisense: string;      // The complementary strand
  parentSpeciesId: string | null; // Parent Species DNA ID (Lineage / Stammbaum!)
  status: "alive" | "extinct";    // Evolved state
  peakPopulation: number; // Highest concurrent specimen count reached
  birthTime: number;      // Creation epoch
  extinctionTime?: number; // Sinking epoch
  generation: number;     // Ancestral depth
  carnivory: number;      // Predation bias
}

/**
 * Initializes the persistent file database connection (REST API verification check).
 */
export async function initDb(): Promise<void> {
  try {
    // Simply check that the dev server REST endpoint is responsive
    const res = await fetch('/api/species/alive');
    if (!res.ok) {
      throw new Error(`REST API check failed: ${res.statusText}`);
    }
  } catch (err) {
    console.warn("REST API Database not ready yet, waiting for Vite Dev Server:", err);
  }
}

/**
 * Saves a species record to the physical local file database.
 */
export async function saveSpecies(record: SpeciesRecord): Promise<void> {
  const res = await fetch('/api/species', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(record)
  });
  if (!res.ok) {
    throw new Error(`Failed to save species: ${res.statusText}`);
  }
}

/**
 * Fetches a single species record by its ID from the local file.
 */
export async function getSpeciesById(id: string): Promise<SpeciesRecord | null> {
  const res = await fetch(`/api/species?id=${encodeURIComponent(id)}`);
  if (!res.ok) {
    throw new Error(`Failed to fetch species by ID: ${res.statusText}`);
  }
  return res.json();
}

/**
 * Fetches all species records that are currently "alive" inside the physical local file.
 */
export async function getAliveSpecies(): Promise<SpeciesRecord[]> {
  const res = await fetch('/api/species/alive');
  if (!res.ok) {
    throw new Error(`Failed to fetch alive species: ${res.statusText}`);
  }
  return res.json();
}

/**
 * Fetches all species records (both alive and extinct fossils) from the database file.
 */
export async function getAllSpecies(): Promise<SpeciesRecord[]> {
  const res = await fetch('/api/species');
  if (!res.ok) {
    throw new Error(`Failed to fetch all species: ${res.statusText}`);
  }
  return res.json();
}

/**
 * Marks a species as extinct in the physical local file.
 */
export async function markSpeciesAsExtinct(id: string): Promise<void> {
  const res = await fetch(`/api/species/extinct?id=${encodeURIComponent(id)}`, {
    method: 'POST'
  });
  if (!res.ok) {
    throw new Error(`Failed to mark species as extinct: ${res.statusText}`);
  }
}

/**
 * Clears the entire physical database file (resets local evolution history).
 */
export async function clearDb(): Promise<void> {
  const res = await fetch('/api/species/clear', {
    method: 'POST'
  });
  if (!res.ok) {
    throw new Error(`Failed to clear database: ${res.statusText}`);
  }
}

/**
 * Fetches the saved running simulation state (creatures, spores, metadata) from disk.
 */
export async function getSavedSimulationState(): Promise<any> {
  const res = await fetch('/api/simulation/state');
  if (!res.ok) {
    throw new Error(`Failed to fetch simulation state: ${res.statusText}`);
  }
  return res.json();
}

/**
 * Saves the current active simulation state securely to disk.
 */
export async function saveSimulationState(state: any): Promise<void> {
  const res = await fetch('/api/simulation/save', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(state)
  });
  if (!res.ok) {
    throw new Error(`Failed to save simulation state: ${res.statusText}`);
  }
}

/**
 * Deletes the saved simulation state from disk.
 */
export async function clearSimulationState(): Promise<void> {
  const res = await fetch('/api/simulation/clear', {
    method: 'POST'
  });
  if (!res.ok) {
    throw new Error(`Failed to clear simulation state: ${res.statusText}`);
  }
}
