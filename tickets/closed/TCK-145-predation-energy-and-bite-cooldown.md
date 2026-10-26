---
id: TCK-145
title: Predation Energy Gain and Bite Cooldown
status: closed
priority: medium
decision: accepted
assigned: unassigned
created_at: 2026-10-19
revisit: RV-B07
epic_phase: Training Track
---

# TCK-145: Predation Energy Gain and Bite Cooldown

> Decision 2026-10-21 (training track, epic TCK-171; default from the session's recommendation, user may veto): the attacker gains `biting_base_energy_gain * carnivory * biting_efficiency_carnivore_scale`; bite cooldown per attacker/victim pair from config. Needed for carnivore viability in the Ocean even though the first two trainer rooms have no live prey. Shares the energy function of TCK-178. Findings (RV-B07) below.

## Description
Biting damages the victim but gives the attacker no energy (`bitingBaseEnergyGain` unused); no bite cooldown.

## Proposed Scope
Decide whether predation should feed the attacker and with which formula.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-B07 · P1 · verified (key unused)** — `src/server/engine.rs` (bite block); `bitingBaseEnergyGain` is never read. Predation gives the attacker no energy; only the victim loses 35. A predator can bite the same victim every tick (no cooldown) and emit several bite events per tick. Carnivory pays off in the Ocean only via meat spores.
- Fix sketch: gain = `biting_base_energy_gain * carnivory * biting_efficiency_carnivore_scale`; bite cooldown per attacker/victim pair.

## Verification (P6, v1.12.20)
- Attacker gain `bitingBaseEnergyGain * carnivory * bitingEfficiencyCarnivoreScale`, capped by the damage dealt (`bitingEnergyDamage`) and by what the victim still has after earlier bites in the same tick, so bites move energy and never create it (lead decision: with the defaults the formula alone reached 43.75 > 35 damage).
- At most one bite per attacker per tick (nearest eligible victim, `bite_target`), per-pair cooldown `biteCooldownTicks` (60, default without measurement), cooldown map cleared on `RESET_EVOLUTION`; a bite sets `has_eaten`. RV-B07 resolved. Tests for food/bite energy and bite-target selection.
- Death messages: "hunted down by predators" only after bite damage in that tick (before, every death with 0 < energy <= 0.1 was reported as predation); test `test_death_without_bite_is_not_reported_as_predation`.
- Ocean check (90 s at max warp): 13 bites, 5 predation deaths, 142 starvation/senescence deaths.
- Golden master `0xe0a2bfd1c7310306` -> `0x9d206dfd8124f808` (phenotype field `basal_metabolic_rate` removed); `MODEL_VERSION` 3 -> 4; CI reference regenerated. `cargo test` 113 passed (1 ignored), 0 warnings; `npm run build`, vitest green.
