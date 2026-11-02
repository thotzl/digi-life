import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

import { call, onIpcError, send, setMode, type CallError, type Command } from "./ipc";

describe("IPC client (TCK-157)", () => {
  const reported: { command: Command; error: CallError }[] = [];

  beforeEach(() => {
    invokeMock.mockReset();
    reported.length = 0;
    onIpcError((command, error) => reported.push({ command, error }));
    (globalThis as any).window = { __TAURI_INTERNALS__: {} };
  });

  afterEach(() => {
    delete (globalThis as any).window;
  });

  it("passes the arguments and resolves with the typed result", async () => {
    invokeMock.mockResolvedValue([{ runId: "r", maxGeneration: 3, maxFitness: 1.5 }]);
    const runs = await call("get_trainer_runs");
    expect(runs[0].maxGeneration).toBe(3);
    expect(invokeMock).toHaveBeenCalledWith("get_trainer_runs", undefined);
    await call("clear_trainer_history", { runId: "r" });
    expect(invokeMock).toHaveBeenLastCalledWith("clear_trainer_history", { runId: "r" });
    expect(reported).toEqual([]);
  });

  it("reports a typed backend error once and rejects with it", async () => {
    const error = { kind: "notFound", message: "creature 'x' not found in the catalogue" };
    invokeMock.mockRejectedValue(error);
    await expect(call("delete_from_catalogue", { id: "x" })).rejects.toEqual(error);
    expect(reported).toEqual([{ command: "delete_from_catalogue", error }]);
  });

  it("maps a non-typed rejection to a transport error", async () => {
    invokeMock.mockRejectedValue("invalid args `id` for command");
    await expect(call("get_fossil_phenotype", { genome: "A" })).rejects.toEqual({ kind: "transport", message: "invalid args `id` for command" });
    expect(reported).toHaveLength(1);
  });

  it("sends actions as objects (no JSON string) and resolves whether they were accepted", async () => {
    invokeMock.mockResolvedValue(null);
    expect(await send({ type: "TOGGLE_SIMULATION", running: false })).toBe(true);
    expect(invokeMock).toHaveBeenCalledWith("handle_client_action", { action: { type: "TOGGLE_SIMULATION", running: false } });
    invokeMock.mockRejectedValue({ kind: "invalidAction", message: "unknown client action type 'X'" });
    expect(await send({ type: "INJECT_URZELLE" })).toBe(false);
    expect(reported).toHaveLength(1);
  });

  it("switches the mode through the set_mode command", async () => {
    invokeMock.mockResolvedValue("trainer");
    expect(await setMode("trainer")).toBe("trainer");
    expect(invokeMock).toHaveBeenCalledWith("set_mode", { mode: "trainer" });
  });

  it("rejects without a toast outside Tauri", async () => {
    delete (globalThis as any).window;
    await expect(call("get_trainer_runs")).rejects.toMatchObject({ kind: "unavailable" });
    expect(invokeMock).not.toHaveBeenCalled();
    expect(reported).toEqual([]);
  });
});
