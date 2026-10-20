# 03 — Performance

Kontext: Der Loop läuft mit 60 Hz und führt bei Warp bis zu 35 Physik-Steps pro Frame aus (≈ 2.100 Steps/s). Jede Operation im Step-Body wird also mit bis zu 2.100/s multipliziert. Aktuell ist die Population klein (25–45), weil Nachkommen nicht leben (RV-B01) — **nach dem Fix von RV-B01 werden die folgenden Punkte akut**.

Alle Befunde `[statisch]`; vor Optimierung Baseline messen (Ticks/s bei Warp 35, Population 45).

---

## RV-P01 · P1 · Vollständiger Species-Table-Scan in jedem Physik-Step — **Status: ✔ ef30b6e (TCK-139)**

**Ort:** `src/server/engine.rs:1445-1504`

`SELECT id, status, peak_population FROM species_records` pro Step, danach für jede Species `creatures.iter().filter(...).count()` → O(S × C) plus SQLite-IO, bis 2.100×/s. `species_records` wächst mit jeder Mutation (RV-B10) und wird nie bereinigt.

**Fix:** Species-Zählung in-memory (`HashMap<SpeciesId, count>`), Status-Änderungen als Events sammeln, DB-Writes gebündelt in einer Transaktion alle N Sekunden.

## RV-P02 · P1 · Spatial Grid wird jeden Step neu allokiert — **Status: ✔ ef30b6e (TCK-139)**

**Ort:** `engine.rs:1016` — `SpatialGrid::new(19200, 10800, 80)` = 240 × 135 = 32.400 Zellen × 2 `Vec`s pro Step.

**Fix:** Grid einmal anlegen, pro Step `clear()` (existiert bereits).

## RV-P03 · P1 · Tiefe Klone kompletter Agenten im Hot Loop — **Status: ✔ ef30b6e (TCK-139)**

- `engine.rs:1028` — für **jede Spore** werden die nahen Kreaturen per `Vec::contains` gefiltert und **vollständig geklont** (inkl. Genom-String, Phänotyp, Brain-Topologie mit hunderten Synapsen). 300 Sporen × k Klone pro Step.
- `engine.rs:1035` — `creatures.clone()` komplett pro Step für `alive_clones`.
- `scenarios.rs:127/311` — `vec![sb.agent.clone()]` pro Sandbox pro Step nur für Sporen-Push.

**Fix:** `step_food_spore_physics` nimmt `&[(x, y, vx, vy, radius)]` bzw. Indizes; Snapshot nur der Positionsdaten (`Vec<AgentKinematics>`).

## RV-P04 · P2 · Nächste-Nahrung-Suche linear über alle Sporen — **Status: → Pool TCK-156 (bewusst zurückgestellt: Tie-Break-Risiko)**

**Ort:** `engine.rs:1050-1093` — jede Kreatur scannt alle Sporen zweimal (Pflanze/Fleisch) und alle Kreaturen. O(C × F + C²). Grid existiert, wird dafür nicht genutzt; `get_nearby_food` liefert IDs und danach `food_pellets.iter().position(id)` → wieder O(F).

**Fix:** Grid speichert Indizes statt IDs; Ringsuche mit wachsendem Radius bis Sensorreichweite.

## RV-P05 · P1 · IPC-Flut aus dem Hot Loop — **Status: ✔ ef30b6e (TCK-139, Ocean) · 27c95ec (TCK-140, Trainer-Welt)**

- `EAT_EVENT`, `BITE_EVENT`, `LOG_EVENT`, `DATABASE_CHANGED` werden **innerhalb** der Step-Schleife emittiert → bei Warp 35 potenziell hunderte Events/Frame. Jedes `DATABASE_CHANGED` löst im Frontend `fetchRosterRecords()` → neuer Tauri-Command → neue DB-Connection aus.
- Trainer-Telemetrie sendet mit 25 Hz pro Sandbox die komplette `ProceduralWorld` (18–28 Hindernisse mit Vertex-Listen), obwohl der Trainer keine Hindernisse nutzt (`&[]` in `scenarios.rs`), sowie das komplette Genom. Bei N = 16: ~16 Welten × 25/s.
- `FOOD_TICK` sendet alle Sporen mit 2 Hz (ok), `INIT_STATE` sendet alle Kreaturen inkl. Phänotyp+Brain (ok, einmalig).

**Fix:** Events im Step in einen Buffer sammeln, einmal pro Frame gebündelt und dedupliziert senden (`DATABASE_CHANGED` max. 1×/s). Welt nur bei Init/Änderung senden.

## RV-P06 · P2 · `mutate_genome` kompiliert jedes Mal den vollen Phänotyp — **Status: ✔ 27c95ec (TCK-140)**

**Ort:** `src/biology/dna.rs:429` — `parse_genome` (inkl. Brain-Synthese mit O(Sources × Destinations × Thalamus-Blöcke)-Reflex-Check, `dna.rs:1260-1304`) nur um zwei Raten zu lesen. `init_rust_sandbox` ruft `mutate_genome` ~15× pro Mutant (256 × 0.06), also ~15 volle Compiles + 1 finaler pro Sandbox pro Generation.

**Fix:** `fn read_evo_rates(genome) -> (f32, f32)` separat; Reflex-Check im Brain-Compiler über ein `HashSet<(from,to)>` statt verschachtelter Schleife.

## RV-P07 · P2 · DB-Abfragen in `while`-Restock-Schleife — **Status: ✔ ef30b6e (TCK-139)**

**Ort:** `engine.rs:1508-1601` — pro fehlender Kreatur wird der komplette Catalogue neu abgefragt und deserialisiert.

**Fix:** Catalogue einmal pro Restock-Durchgang laden (oder cachen, invalidieren bei `save/delete`).

## RV-P08 · P3 · Allokationen pro Brain-Tick — **Status: offen, nur mit Profiler-Beleg**

`execute_brain_with_learning` allokiert `outputs` pro Aufruf; `compute_sensory_inputs` allokiert `inputs`. Bei < 50 Agenten irrelevant, erst bei Massen-Training (CLI) relevant. Nur mit Profiler-Beleg angehen.
