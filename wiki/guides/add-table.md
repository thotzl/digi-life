---
sources: [wiki/contracts/persistence-repo.md]
---

# Guide: add a table (or a stored field)

Read first: [[wiki/contracts/persistence-repo]] and [[wiki/architecture/invariants]].

## Steps

1. Pick the owning module in `src/persistence/` (`species.rs`, `catalogue.rs`, `trainer.rs`, `ocean.rs`); a new
   subsystem gets its own module, declared in `persistence/mod.rs` and chained into `schema::tables()`.
2. Latest schema: add a `TableDef` to the module's `TABLES` (`CREATE TABLE IF NOT EXISTS` plus indexes) with
   `model_scoped: true` if the rows depend on the genome/brain model, else `false`. A new column on an existing table
   changes that table's `CREATE` in `TABLES`.
3. Migration: write a step `pub(super) fn vN_<what>(conn: &Connection) -> Result<()>` in the module and append
   `Migration { version: N, name, steps }` to `schema::MIGRATIONS` (N = last version + 1). Never edit a released
   migration. Dropping data: say so in the name.
4. DTO: a plain struct in `persistence/dto.rs` (no model types; map them in an adapter next to the model like
   `biology::trainer_store`).
5. Trait methods on the owning trait in `persistence/repos.rs` (or a new trait), implemented for `Db` in the module
   and for `MemoryDb` in `memory.rs`. Repositories set their own timestamps.
6. Callers: commands in `src/server/api.rs` return the error text; engine calls use `log_db_err`. No call per
   physics step.
7. Retention: if the table grows with play time, bound it (config key in the owning section, see
   `wiki/guides/add-config-key.md`) and prune on a timer or event.
8. Tests: extend the contract suite (`persistence::tests::contract`) so both stores run the new methods; if the
   migration transforms existing rows, add them to the v1.12.30 fixture of
   `test_v1_12_30_database_migrates_without_data_loss` and assert they survive.
9. Docs: the table row in `wiki/architecture/persistence.md`.

## Done when

- `cargo test persistence::` is green, including `test_migrated_schema_equals_latest_tables`.
- `cargo test`, `npm test`, `npm run build` and `node scripts/check-links.mjs` are green.
- No SQL outside `src/persistence/`; golden master and the trainer references unchanged.
