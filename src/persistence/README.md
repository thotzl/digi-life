# persistence — the only module with SQL

`init_db` and migrations (`schema.rs`), repository traits (`repos.rs`) implemented by `Db` (one module per table group) and `MemoryDb` (`memory.rs`), DTOs (`dto.rs`), contract tests (`tests.rs`).

- Guide: add a table: [`wiki/guides/add-table.md`](../../wiki/guides/add-table.md)
- Contract: persistence repository: [`wiki/contracts/persistence-repo.md`](../../wiki/contracts/persistence-repo.md)
- Rules: [`wiki/architecture/invariants.md`](../../wiki/architecture/invariants.md); module list: [`wiki/architecture/module-map.md`](../../wiki/architecture/module-map.md)
