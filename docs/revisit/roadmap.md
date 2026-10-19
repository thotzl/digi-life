# Roadmap — Stabilisieren → Umbauen → Weiterentwickeln

Leitlinie: Jede Phase endet mit einem Hard-Stop und expliziter Freigabe. Kein Feature-Ticket (TCK-111/118/123/113) startet vor Abschluss von Phase 0.

---

## Phase 0 — Stabilisieren (Korrektheit + Release-Fähigkeit)

Ziel: Die Simulation tut, was sie vorgibt; Release-Builds funktionieren; jeder Fix ist durch einen Test abgesichert. **Minimal-invasiv** — keine Strukturumbauten außer wo für Testbarkeit zwingend.

| Schritt | Inhalt | IDs |
| :--- | :--- | :--- |
| 0.1 | Rust-Toolchain auf Windows installieren; Baseline `cargo test`, `cargo clippy` dokumentieren | T05 |
| 0.2 | CI-Workflow (Test + Build), `release.yml` nur auf Tags | T01 |
| 0.3 | Warnhinweis-Banner in die 3 falschesten Skills (WebSocket/JSON) | X01 (Teil) |
| 0.4 | Wandreflexion fixen + 4-Wände-Test | B02 |
| 0.5 | DB-Pfad → `app_data_dir`, Einmal-Migration der alten DB; Config per `include_str!` + Override, Parse-Fehler laut | B03, B04 |
| 0.6 | Trainer `epoch_ticks` inkrementieren + Test | B05 |
| 0.7 | **Mitose-Kinder in Simulation aufnehmen** + `max_creatures`-Cap; gleichzeitig Species-Zählung in-memory statt DB-Scan pro Step | B01, P01 |
| 0.8 | Grid-Reuse, Agenten-Klone aus Hot Loop entfernen, Events pro Frame bündeln | P02, P03, P05 |
| 0.9 | Unbekannte Actions loggen; `SPAWN_FOOD` implementieren oder UI entfernen | B09 |
| 0.10 | Manueller Smoke-Test der App (Ocean läuft, Kinder sichtbar, Trainer-Generation, Catalogue speichern/laden) | — |

**DoD Phase 0:** `cargo test` + `npm run build` grün in CI; Ocean zeigt Generationszuwachs durch echte Nachkommen; Release-Build auf Windows startet, DB liegt in AppData; keine DB-Query im Physik-Step.

**Hard-Stop:** `Phase 0 complete. Waiting for explicit 'GO PHASE 1'.`

---

## Phase 1 — Gezielter Umbau (nur was Phase 2 blockiert)

Ziel: Engine testbar und erweiterbar machen, Vertrag Frontend↔Backend typisieren, Messwerkzeug für Balancing. Jeder Punkt ist durch ein konkretes Phase-2-Feature begründet.

| Schritt | Inhalt | Begründet durch | IDs |
| :--- | :--- | :--- | :--- |
| 1.1 | Engine in `OceanSim`/`TrainerSim`-Structs mit `step()` + `EventSink`; `CreatureFactory` | TCK-123 (Szenarien), Headless-Runner | A01 |
| 1.2 | Seedbarer RNG durchreichen | Reproduzierbare Tests/Balancing | T04 |
| 1.3 | Typisierte `Action`/`SimEvent`-Enums, ts-rs-Export, Frontend nutzt `bindings/` | TCK-111 (Creator braucht stabilen Vertrag) | A02, A03, F01 |
| 1.4 | Repository-Schicht, einmalige Migration mit `user_version`, WAL, Retention | TCK-123 (HoF-Export), Datenwachstum | A04, D02–D05 |
| 1.5 | Headless-Runner `--seed --ticks --report` ersetzt `cli_trainer` | Balancing S02/S03/S07 | T03, S11 |
| 1.6 | Verhaltenstests gemäß RV-T02-Liste | Regressionen bei Phase 2 | T02 |
| 1.7 | Config-Bereinigung: jeden toten Key implementieren oder löschen (**Entscheidung User pro Key**), `deny_unknown_fields` | SSOT-Mandat | S01, B07, B08 |
| 1.8 | Species-Modell: Distanz-basierte Speziation, ID ≠ Genom (**Design-Entscheidung User**) | Roster/Analytics sinnvoll | B10 |
| 1.9 | Gene-Spans vom Backend; Legacy-Locus-Logik im Frontend entfernen; `innerHTML` → `textContent`; CSP | TCK-111 Editor zeigt Gene korrekt | F02, F03 |
| 1.10 | Skills, README, HANDOFF, active-task neu schreiben; Tickets aufräumen | Agent-Zuverlässigkeit | X01–X05 |

**DoD Phase 1:** Engine-Logik ohne Tauri testbar; Frontend importiert Typen aus `bindings/`; Headless-Report reproduzierbar mit Seed; Skills beschreiben den Ist-Zustand.

**Hard-Stop:** `Phase 1 complete. Waiting for explicit 'GO PHASE 2'.`

---

## Phase 2 — Weiterentwicklung

Erst hier beginnen inhaltliche Features. Reihenfolge-Vorschlag nach Abhängigkeit:

1. **Modell-Balancing mit Messreihen** (RV-S02 Schub, RV-S03 BMR, RV-S07 Ökologie, RV-S04 Hebbian-Normalisierung, RV-B06 Motion-Vision-Kanal). Erst messen, dann Konstanten — Ergebnisse in `docs/` festhalten.
2. **TCK-123 Curriculum/Multi-Scenario + Champion-Seeding** — Plugin-Trait existiert, nach 1.1 sauber erweiterbar; RV-S08 (Spawn-Distanzen ins Plugin) ist Vorbedingung.
3. **TCK-111 Creator/Genom-Editor** — nach 1.3/1.9; Scope anpassen: Speicherung in SQLite-Catalogue statt IndexedDB, Live-Preview über `get_fossil_phenotype`.
4. **TCK-118 Diploide Genetik** — tiefer Eingriff in `parse_genome`; braucht Property-Tests (RV-T02 #6) und Headless-Vergleich vorher/nachher.
5. **Ideen aus `ideas.md`** (Swimbot-Oszillatoren, Hormone/Endokrinologie als Ersatz für das tote Adrenalin-System, Biogenesis-Metabolismus) — einzeln als Tickets, jeweils mit Messkriterium.
6. **TCK-113 Mobile** — zuletzt; setzt stabile Performance (Phase 0.8) und Touch-fähige Views voraus.

---

## Bewusst NICHT tun

- Kein Rewrite in ein ECS-Framework oder anderes Frontend-Framework (React/Svelte) — Größe rechtfertigt es nicht, das Problem ist Struktur, nicht Tooling.
- Keine Dependency-Major-Updates vor grüner CI-Baseline.
- Kein Kommentar-/Formatierungs-Pass als eigene Aufgabe (RV-X06 nur beim Anfassen).
- Keine Performance-Mikrooptimierung (RV-P08) ohne Profiler-Beleg.
