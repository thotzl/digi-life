---
sources: [tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-145-predation-energy-and-bite-cooldown.md, tickets/closed/TCK-190-thalamus-sum-silent-offset.md, tickets/closed/TCK-171-training-track-epic.md, wiki/biology/metabolism-and-reproduction.md]
---

# ADR-011: Energy Model v2 — Additive, Activity-Based Cost Shared by Ocean and Trainer

- Status: accepted (recorded retroactively on 2026-11-03)
- Date: 2026-10-21
- Tickets: TCK-151 (user decision), TCK-178, TCK-145; recalibration in TCK-190; epic TCK-171 phase P6 (v1.12.20)

## Context

The basal metabolic rate multiplied by synapse and input count (RV-S03): one sense organ raised BMR ×3–×8, the Ocean compensated with a `* 0.45` literal, and selection pressure ran against sense organs. The trainer did not drain energy at all, so champions could starve in the Ocean (TCK-178). Biting gave the attacker no energy and had no cooldown (RV-B07, TCK-145).

## Decision

1. **Additive, activity-based cost** (user, 2026-10-21, after research; TCK-151): `cost per tick = body + Σ organ(size) + a · neurons + b · Σ |weight · source activation|`, plus movement cost by thrust. Rationale given in the ticket: brain energy scales roughly linearly with neuron count (Herculano-Houzel 2011); most signalling energy scales with activity (Attwell & Laughlin 2001), so near-zero weights cost almost nothing without a pruning threshold.
2. **Constants from share targets, not free constants:** brain + senses ≈ 10–20 % of resting cost for a typical creature; absolute scale matched to the old median Ocean drain 0.0243/tick. The `* 0.45` literal is removed.
3. **One energy function for Ocean and trainer** (TCK-178): the trainer drains `trainerMetabolicScale` (1.0) × the shared cost; a creature at 0 energy is starved for the rest of the trial.
4. **Predation moves energy** (TCK-145; default from the session's recommendation, user may veto, 2026-10-21): attacker gain `bitingBaseEnergyGain · carnivory · bitingEfficiencyCarnivoreScale`, capped by the damage dealt and the victim's remaining energy (lead decision: the formula alone gave 43.75 > 35 damage); one bite per attacker per tick; per-pair cooldown `biteCooldownTicks` 60 ("default without measurement").

## Alternatives considered

| Alternative | Why not (source) |
| :--- | :--- |
| Multiplicative BMR (before) | selects against sense organs (RV-S03) |
| Per-synapse price with a pruning threshold (ticket's original scope) | replaced by the activity cost: no threshold edge, exuberant start brain not punished (TCK-151) |
| Connection cost as a selection objective (Clune et al. 2013) | kept as an escalation option of TCK-175, not adopted |
| Metabolic ramp during the curriculum | measured in TCK-176, lowered success; default off (ADR-010) |

## Consequences

- Golden master `0xe0a2bfd1c7310306` → `0x9d206dfd8124f808`, `MODEL_VERSION` 3 → 4.
- Sensor equipment did not collapse under the budget (food-organ share small 0.907 → 0.958, large 0.812 → 0.917); large-room starved share 0 → 0.182.
- The constants depend on the typical brain activity: the tanh change (ADR-013) lowered it, and the constants were re-derived with the documented procedure (scale factor 0.6296; brain + senses share 15.5 %).
- Current model: [[wiki/biology/metabolism-and-reproduction]].
