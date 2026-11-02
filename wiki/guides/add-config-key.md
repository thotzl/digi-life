---
sources: [wiki/contracts/config-section.md]
---

# Guide: add a config key

Read first: [[wiki/contracts/config-section]] and [[wiki/architecture/invariants]].

## Steps

1. Pick the section whose code reads the key (`core`, `ocean`, `trainer`, `genome`, `brain`, `metabolism`,
   `physics`). A new subsystem with several keys gets its own section (contract: Registration point).
2. Add a `pub` field to the section struct (file in the contract's Signature list) with a doc comment: meaning, unit,
   ticket. snake_case in Rust; the JSON key is its camelCase form.
3. Add the key with its value to the same section of `config.json`. No default in Rust.
4. Read it where it is used through the config passed in (`config.<section>.<field>` or the section parameter). If
   the function has no config yet, add a parameter; never call `AppConfig::global()` outside `main.rs` and
   `src/bin/trainer_runner.rs`.
5. Tests that need the value use `crate::config::test_config()`.
6. List the key in its section in `wiki/architecture/configuration.md` in the same commit.

## Done when

- `cargo test config::tests` is green (every key read, round trip, unknown keys rejected).
- `cargo test` (golden master unchanged unless the ticket says the model changes) and the three trainer reference
  runs of `.github/workflows/ci.yml` match, or the ticket states the intended change.
- `node scripts/check-links.mjs` is green.
