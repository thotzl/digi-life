---
id: TCK-145
title: Predation Energy Gain and Bite Cooldown
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-B07
---

# TCK-145: Predation Energy Gain and Bite Cooldown

> Pool ticket: requires a user decision before implementation. Findings (RV-B07) below.

## Description
Biting damages the victim but gives the attacker no energy (`bitingBaseEnergyGain` unused); no bite cooldown.

## Proposed Scope
Decide whether predation should feed the attacker and with which formula.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-B07 · P1 · verified (key unused)** — `src/server/engine.rs` (bite block); `bitingBaseEnergyGain` is never read. Predation gives the attacker no energy; only the victim loses 35. A predator can bite the same victim every tick (no cooldown) and emit several bite events per tick. Carnivory pays off in the Ocean only via meat spores.
- Fix sketch: gain = `biting_base_energy_gain * carnivory * biting_efficiency_carnivore_scale`; bite cooldown per attacker/victim pair.
