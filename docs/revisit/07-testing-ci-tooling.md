# 07 — Tests, CI, Tooling

## Ist-Zustand

| Ebene | Stand |
| :--- | :--- |
| Rust-Unit-Tests | ~20 in `dna.rs` (12), `brain.rs` (2), `database.rs` (2); `engine.rs`, `physics.rs`, `scenarios.rs`, `trainer_engine.rs`, `api.rs` ohne Tests |
| Frontend-Tests | 0 Dateien (`vitest.config.ts` sucht `frontend/**/*.test.ts`) |
| Integration/Headless | keine |
| CI | `release.yml` baut bei **jedem Push auf master** Release-Installer für 4 Plattformen; kein `cargo test`, kein `tsc`, kein `clippy` |
| Lokale Toolchain (dieser Rechner) | Node OK, **kein Rust** |

---

## RV-T01 · P0 · Keine Verifikations-Pipeline — **Status: ✔ f04a035 (TCK-134)**

Ein Push kann einen Draft-Release mit kaputtem Rust-Code erzeugen, ohne dass irgendein Test lief.

**Ziel:** `ci.yml` auf Push/PR: `cargo fmt --check`, `cargo clippy -- -D warnings` (nach Baseline), `cargo test`, `git diff --exit-code bindings/`, `npm ci && npm run build`. `release.yml` nur auf Tags `v*` (Branch-Trigger entfernen).

## RV-T02 · P1 · Tests prüfen Implementierungsdetails statt Verhalten — **Status: teilweise ✔ (Repro-/Golden-Master-Tests in TCK-135..162) · Rest → Pool TCK-161**

- `brain.rs::test_execute_brain_determinism` füttert 4 Inputs ("3 sensors + 1 clock") in ein Gehirn, das ≥ 7 Inputs erwartet — testet ein veraltetes Modell und prüft keine Determinismus-Eigenschaft (kein Vergleich zweier Läufe).
- `database::tests` testen SQLite selbst (Insert/Select), nicht die Repository-Logik.
- Kein Test hätte RV-B01, RV-B02, RV-B05 gefunden.

**Ziel — Verhaltenstests, die echte Fehlerklassen fangen:**
1. Physik: Wandreflexion an allen 4 Wänden (RV-B02).
2. Ocean-Step: nach Mitose ist `creatures.len()` um 1 gestiegen und das Kind wird im nächsten Step bewegt (RV-B01).
3. Brain: identische Inputs → bitgleiche Outputs über zwei unabhängige Läufe; Output-Range je Aktivierungstyp.
4. Trainer: `finish_tick` reflektiert tatsächlichen Fresszeitpunkt (RV-B05).
5. Config: jeder Key in `config.json` deserialisiert; keine unbekannten Keys (RV-S01).
6. Property-Test (`proptest`): `parse_genome` panict für kein ASCII-Genom 0..1024 Zeichen; `mutate_genome` hält Länge in [128, 512].

Stryker erst einsetzen, wenn es überhaupt FE-Tests gibt.

## RV-T03 · P1 · Kein Headless-Messwerkzeug für Balancing — **Status: → Pool TCK-158**

Balancing-Entscheidungen (RV-S02, S03, S07) brauchen reproduzierbare Messreihen: N Ticks, Seed fix, Kennzahlen (Population, Geschwindigkeitsverteilung, Fressrate, Species-Diversität). Voraussetzung: RV-A01 (Engine ohne Tauri) + seedbarer RNG (aktuell überall `thread_rng()` → nicht reproduzierbar).

**Ziel:** `cargo run --bin headless -- --seed X --ticks N --report json` ersetzt `cli_trainer`; Skill `simulation-analytics` darauf umschreiben.

## RV-T04 · P1 · Nicht-deterministischer RNG — **Status: → Pool TCK-158**

`rand::thread_rng()` an ~20 Stellen. Weder Ocean noch Trainer sind mit gleichem Seed reproduzierbar; `config.seed` ist ungenutzt. Ohne Determinismus sind Regressions- und Balancing-Tests nur statistisch möglich.

**Ziel:** Ein `StdRng`/`ChaCha8Rng` pro Simulation, aus Config-Seed initialisiert, als Parameter durchgereicht.

## RV-T05 · P2 · Toolchain/Plattform — **Status: teilweise ✔ f04a035 (Toolchain, TCK-134) · Dependency-/Lint-Updates → Pool TCK-161**

- Entwicklung lief offenbar unter Linux (`/home/torsten/...` in Tickets/Skills), aktueller Rechner ist Windows ohne Rust. Setup-Anleitung für Windows fehlt (rustup + MSVC Build Tools + WebView2).
- `Cargo.toml`: Paketname `src-tauri`, Version `0.1.0` (App ist 1.12.2), Edition 2024; `rusqlite 0.29` und `rand 0.8` sind veraltet — Update erst nach grüner Baseline.
- `package.json`: `typescript ^5.0.0`, `vite ^5`, `vitest ^1.6` — npm meldet Deprecations transitiver Pakete. Kein akuter Handlungsbedarf.
- `npm run types:generate` ist ein Alias für `cargo test` — Typgenerierung hängt am Erfolg aller Tests.
