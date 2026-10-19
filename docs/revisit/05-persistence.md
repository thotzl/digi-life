# 05 — Persistenz (SQLite)

Schema: `src/database.rs` — 6 Tabellen: `sessions`, `species_records`, `simulation_history`, `simulation_state`, `trainer_genomes`, `creature_catalogue`.

---

## RV-D01 · P0 · DB-Speicherort → siehe RV-B03

## RV-D02 · P1 · Migrationen sind ad hoc und laufen bei jedem Command

`init_db` wird bei jedem Tauri-Command ausgeführt (RV-A04). Migrationen bestehen aus `CREATE TABLE IF NOT EXISTS` + einem manuellen `PRAGMA table_info`-Check für zwei Spalten; Fehler bei `ALTER TABLE` werden verworfen (`let _ =`). Kein Schema-Versionsfeld.

**Ziel:** `PRAGMA user_version` + nummerierte Migrationen, einmal beim Start in einer Transaktion (Skill `core-database-safety`).

## RV-D03 · P1 · Unbegrenztes Wachstum

- `species_records`: eine Zeile pro einzigartigem Genom (RV-B10). Nach RV-B01-Fix potenziell tausende Zeilen/Stunde, und jede wird pro Step gescannt (RV-P01).
- `trainer_genomes`: pro Generation `elite_count` Zeilen inkl. JSON-Synapsengewichten (hunderte Floats), nie bereinigt.
- `simulation_history`: alle 10 s eine Zeile, nie bereinigt (unkritisch, aber ohne Nutzer — keine UI liest sie).
- `simulation_state`: alle 10 s komplettes JSON aller Kreaturen inkl. Phänotyp+Brain — ok als Upsert, aber teuer im Engine-Thread.

**Ziel:** Retention-Regeln (HoF Top-N pro Run behalten, Rest löschen), `simulation_history` entweder nutzen (Charts) oder entfernen.

## RV-D04 · P2 · Inkonsistente Zeitstempel

| Tabelle/Spalte | Einheit |
| :--- | :--- |
| `species_records.birth_time/extinction_time` | Sekunden (Variable heißt `now_millis`) |
| `trainer_genomes.created_at` | Sekunden |
| `simulation_state.updated_at` | Millisekunden (Engine) **und** Sekunden (`apply_champion`) |
| `creature_catalogue.created_at`, `sessions.created_at` | SQLite-Text `CURRENT_TIMESTAMP` |

**Ziel:** Ein Typ (Unix-ms als INTEGER), eine Helper-Funktion.

## RV-D05 · P2 · Fehler werden flächig verschluckt

- Engine: praktisch alle `conn.execute` mit `let _ =` (≈ 25 Stellen).
- API: `for row in rows { if let Ok(item) = row ... }` — fehlerhafte Zeilen verschwinden still.
- `serde_json::from_str(...).unwrap_or_default()` für Methylierungen/Synapsen — korrupte Daten führen zu stillem Verlust gelernter Gewichte.

**Ziel:** Fehler loggen (mindestens `eprintln!` mit Kontext), API-Commands geben Fehler zurück.

## RV-D06 · P3 · Toter/irreführender Zustand

- `apply_champion` schreibt `progenitor_genome` in `simulation_state` — wird nirgends gelesen.
- `sessions` existiert nur, um einen Foreign Key zu erfüllen (`desktop-session-001` hart codiert).
- Bei `RESET_EVOLUTION` wird `simulation_state` komplett gelöscht, inkl. `progenitor_genome`.
