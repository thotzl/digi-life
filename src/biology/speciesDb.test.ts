import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { 
  initDb, 
  saveSpecies, 
  getSpeciesById, 
  getAliveSpecies, 
  getAllSpecies, 
  markSpeciesAsExtinct, 
  clearDb,
  getSavedSimulationState,
  saveSimulationState,
  clearSimulationState
} from './speciesDb';

describe('Database and Resumption API Client', () => {
  const originalFetch = globalThis.fetch;

  beforeEach(() => {
    globalThis.fetch = vi.fn();
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
  });

  it('should initialize DB correctly', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => []
    } as Response);

    await expect(initDb()).resolves.not.toThrow();
    expect(globalThis.fetch).toHaveBeenCalledWith('/api/species/alive');
  });

  it('should save a species record correctly', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ success: true })
    } as Response);

    const mockRecord = {
      id: "ABC",
      name: "Mock species",
      genome: "ABC",
      antisense: "ZYX",
      parentSpeciesId: null,
      status: "alive" as const,
      peakPopulation: 1,
      birthTime: Date.now(),
      generation: 1,
      carnivory: 0.1
    };

    await expect(saveSpecies(mockRecord)).resolves.not.toThrow();
    expect(globalThis.fetch).toHaveBeenCalledWith('/api/species', expect.objectContaining({
      method: 'POST',
      body: JSON.stringify(mockRecord)
    }));
  });

  it('should get species by ID correctly', async () => {
    const mockRecord = { id: "ABC", name: "Mock species" };
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => mockRecord
    } as Response);

    const res = await getSpeciesById("ABC");
    expect(res).toEqual(mockRecord);
    expect(globalThis.fetch).toHaveBeenCalledWith('/api/species?id=ABC');
  });

  it('should fetch alive species correctly', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => []
    } as Response);

    const res = await getAliveSpecies();
    expect(res).toEqual([]);
    expect(globalThis.fetch).toHaveBeenCalledWith('/api/species/alive');
  });

  it('should fetch all species correctly', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => []
    } as Response);

    const res = await getAllSpecies();
    expect(res).toEqual([]);
    expect(globalThis.fetch).toHaveBeenCalledWith('/api/species');
  });

  it('should mark species as extinct correctly', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true
    } as Response);

    await expect(markSpeciesAsExtinct("ABC")).resolves.not.toThrow();
    expect(globalThis.fetch).toHaveBeenCalledWith('/api/species/extinct?id=ABC', { method: 'POST' });
  });

  it('should clear DB correctly', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true
    } as Response);

    await expect(clearDb()).resolves.not.toThrow();
    expect(globalThis.fetch).toHaveBeenCalledWith('/api/species/clear', { method: 'POST' });
  });

  it('should get saved simulation state correctly', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ empty: false })
    } as Response);

    const res = await getSavedSimulationState();
    expect(res).toEqual({ empty: false });
    expect(globalThis.fetch).toHaveBeenCalledWith('/api/simulation/state');
  });

  it('should save simulation state correctly', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true
    } as Response);

    await expect(saveSimulationState({ creatures: [] })).resolves.not.toThrow();
    expect(globalThis.fetch).toHaveBeenCalledWith('/api/simulation/save', expect.objectContaining({
      method: 'POST',
      body: JSON.stringify({ creatures: [] })
    }));
  });

  it('should clear simulation state correctly', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true
    } as Response);

    await expect(clearSimulationState()).resolves.not.toThrow();
    expect(globalThis.fetch).toHaveBeenCalledWith('/api/simulation/clear', { method: 'POST' });
  });
});
