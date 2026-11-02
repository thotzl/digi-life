// Typed IPC client (TCK-157): the only module that talks to Tauri. Actions, events, command results and errors are
// generated Rust types from `bindings/` (contract: wiki/contracts/ipc-message.md).
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { CatalogueCreature } from "../../bindings/CatalogueCreature";
import type { ClientAction } from "../../bindings/ClientAction";
import type { CreaturePhenotype } from "../../bindings/CreaturePhenotype";
import type { IpcError } from "../../bindings/IpcError";
import type { SimMode } from "../../bindings/SimMode";
import type { SpeciesEntry } from "../../bindings/SpeciesEntry";
import type { TrainerRunInfo } from "../../bindings/TrainerRunInfo";
import type { UiEvent } from "../../bindings/UiEvent";
import { showToast } from "./toast";

export type { ClientAction, IpcError, SimMode, UiEvent };

/** Actions sent with `send`. `SET_MODE` is not among them: the mode changes only through `setMode` (router). */
export type Action = Exclude<ClientAction, { type: "SET_MODE" }>;

/** Tauri commands: arguments (camelCase, Tauri maps them to the Rust parameter names) and result. */
export interface Commands {
  handle_client_action: { args: { action: ClientAction }; result: null };
  set_mode: { args: { mode: SimMode }; result: SimMode };
  get_registered_species: { args: void; result: SpeciesEntry[] };
  get_fossil_phenotype: { args: { genome: string }; result: CreaturePhenotype };
  get_trainer_runs: { args: void; result: TrainerRunInfo[] };
  clear_trainer_history: { args: { runId: string }; result: null };
  get_catalogue_creatures: { args: void; result: CatalogueCreature[] };
  save_to_catalogue: {
    args: { id: string; name: string; genome: string; source: string; fitness: number; methylations: number[]; synapseWeights: number[] };
    result: null;
  };
  delete_from_catalogue: { args: { id: string }; result: null };
  rename_catalogue_creature: { args: { id: string; newName: string }; result: null };
  spawn_catalogue_creature_to_ocean: { args: { id: string; loadLearnedSynapses: boolean }; result: null };
  add_catalogue_creature_to_training: { args: { creatureId: string; runId: string; generation: number }; result: null };
}
export type Command = keyof Commands;

/** Rejection of `call`: a typed backend error, or `transport` (Tauri could not run the command, e.g. bad arguments). */
export type CallError = IpcError | { kind: "transport" | "unavailable"; message: string };

export type IpcErrorHandler = (command: Command, error: CallError) => void;

let errorHandler: IpcErrorHandler = (command, error) => showToast(`${command}: ${error.message}`, "error");

/** Replaces the error hook (default: toast). Every failed `call` and `send` reports through it once. */
export function onIpcError(handler: IpcErrorHandler) {
  errorHandler = handler;
}

function tauriAvailable(): boolean {
  return typeof window !== "undefined" && !!(window as any).__TAURI_INTERNALS__;
}

function isIpcError(e: unknown): e is IpcError {
  return typeof e === "object" && e !== null && typeof (e as any).kind === "string" && typeof (e as any).message === "string";
}

/**
 * Runs a Tauri command. Resolves with its typed result; on failure reports the error through the error hook (toast)
 * and rejects with the `CallError`, so callers only catch for control flow. Outside Tauri (plain `vite dev`) it
 * rejects with `unavailable` without a toast.
 */
export async function call<C extends Command>(
  command: C,
  ...args: Commands[C]["args"] extends void ? [] : [Commands[C]["args"]]
): Promise<Commands[C]["result"]> {
  if (!tauriAvailable()) {
    throw { kind: "unavailable", message: "Tauri backend not available" } satisfies CallError;
  }
  try {
    return await invoke<Commands[C]["result"]>(command, args[0] as Record<string, unknown> | undefined);
  } catch (e) {
    const error: CallError = isIpcError(e) ? e : { kind: "transport", message: String(e) };
    console.error(`[IPC] ${command} failed:`, error);
    errorHandler(command, error);
    throw error;
  }
}

/**
 * Sends an action to the engine thread. Resolves `true` once the backend accepted it, `false` if it was rejected
 * (already reported through the error hook); never rejects.
 */
export function send(action: Action): Promise<boolean> {
  return call("handle_client_action", { action }).then(
    () => true,
    () => false,
  );
}

/**
 * Switches the backend mode. Only `navigateTo` (core/router.ts) calls this, plus the trainer's reset to Ocean on
 * unload (invariant 1). Resolves with the queued mode, `null` if the call failed (reported through the error hook).
 */
export function setMode(mode: SimMode): Promise<SimMode | null> {
  return call("set_mode", { mode }).catch(() => null);
}

type UiEventHandler = (event: UiEvent) => void;
const handlers = new Set<UiEventHandler>();
let subscribed = false;

/**
 * Subscribes to the backend events (one `simulation-state` listener for the whole app, dispatching typed `UiEvent`s).
 * Returns the unsubscribe function.
 */
export function onUiEvent(handler: UiEventHandler): () => void {
  handlers.add(handler);
  if (!subscribed && tauriAvailable()) {
    subscribed = true;
    listen<UiEvent>("simulation-state", (event) => {
      for (const h of handlers) {
        h(event.payload);
      }
    }).catch((err) => {
      subscribed = false;
      console.error("[IPC] subscribing to 'simulation-state' failed:", err);
      showToast(`Event stream unavailable: ${err}`, "error");
    });
  }
  return () => handlers.delete(handler);
}
