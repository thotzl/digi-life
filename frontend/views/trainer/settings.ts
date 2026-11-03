// Trainer hyperparameters as the UI holds them (TCK-160). The backend state (`TRAINER_STATE_CHANGED`, sent when the
// trainer mode is entered) is the source of truth. The UI sends partial updates: a control sends only the field it
// changed (TCK-161), so a change made before the backend state arrived is not lost and no stale value from another
// control (or a client default at boot) can overwrite the backend.
import type { TrainerAction } from "../../../bindings/TrainerAction";
import type { TrainerState } from "../../../bindings/TrainerState";

export type HyperparamsAction = Extract<TrainerAction, { type: "UPDATE_TRAINER_HYPERPARAMS" }>;

export class TrainerSettings {
  /** True once a backend trainer state was applied */
  synced = false;
  /** Visible sandboxes (grid tiles, telemetry); the grid shows the first N sandboxes (elites first) */
  N = 16;
  population = 100;
  warpSpeed = 1;
  eliteRatio = 0.25;
  // Mutation operator calls per mutant / hall-of-fame child: uniform in [mutationOpsMin, mutationOpsMax] (TCK-174)
  mutationOpsMin = 1;
  mutationOpsMax = 3;
  mutationOpsLimit = 8;
  inflowRate = 0.15;
  hofRate = 0.15;
  multiTrial = true;
  lamarckian = false;
  runId = "default_run";

  /** Copies a backend state; returns whether N changed (the grid must be rebuilt). */
  apply(state: TrainerState): boolean {
    const gridChanged = state.N !== this.N;
    this.synced = true;
    this.population = state.population;
    this.N = state.N;
    this.warpSpeed = state.warpSpeed;
    this.eliteRatio = state.eliteRatio;
    this.mutationOpsMin = state.mutationOpsMin;
    this.mutationOpsLimit = state.mutationOpsLimit;
    this.mutationOpsMax = state.mutationRate;
    this.inflowRate = state.inflowRate;
    this.hofRate = state.hofRate;
    this.multiTrial = state.multiTrial;
    this.lamarckian = state.lamarckian;
    this.runId = state.runId;
    return gridChanged;
  }

  mutationOpsLabel(max = this.mutationOpsMax): string {
    return max > this.mutationOpsMin ? `${this.mutationOpsMin}–${max}` : `${max}`;
  }

  /** A hyperparameter update carrying only the given fields (absent fields stay unchanged in the backend). */
  update(fields: Omit<HyperparamsAction, "type">): HyperparamsAction {
    return { type: "UPDATE_TRAINER_HYPERPARAMS", ...fields };
  }

}
