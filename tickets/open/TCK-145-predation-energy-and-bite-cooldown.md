---
id: TCK-145
title: Predation Energy Gain and Bite Cooldown
status: open
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
