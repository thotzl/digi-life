---
sources: [wiki/contracts/organ-kind.md]
---

# Guide: add an organ kind

Read first: [[wiki/contracts/organ-kind]] and [[wiki/architecture/invariants]].

## Steps

1. `src/biology/organs/<kind>.rs`: write `live_channels`, `sense` (or point to `spectral::*` / `contact::*`) and
   `pub const SPEC: OrganSpec` (names, channel names, `distance`, cost, limb, ballast, Latin suffix and rank; free
   ranks and the suffix rule: contract, Signature).
2. `src/biology/organs/mod.rs`: `pub mod <kind>;`, a variant in `OrganKind`, an entry in `OrganKind::ALL` (raise
   `N` in `[OrganKind; N]`), an arm in `OrganKind::spec`, and its region in `OrganKind::classify` (otherwise no
   genome expresses it). Choose the region by the contract's rule (Registration point): never split Eye or Nose by
   style, or the structured genome generator breaks `test_structured_genome_parses_to_exactly_the_planned_genes`.
3. Config keys: none unless the kind has a tunable; then follow `wiki/guides/add-config-key.md`.
4. Tests: `test_live_channels_match_sense` covers the kind automatically; add a unit test for any non-trivial `sense`.
5. Run `cargo test`; it regenerates `bindings/OrganKind.ts` and `bindings/organRegistry.ts` (invariant 7), where the
   frontend's names come from. Add a painter for the kind to `ORGAN_PAINTERS` in `frontend/render/organs.ts`
   (`Record<OrganKind, …>`, so `npm run build` fails without it).
6. Model change (only if step 2 gave the kind a region): run the four reference runs, update the golden master and
   the references that differ and bump `MODEL_VERSION` (contract, Test recipe; invariants 8, 9).
7. Docs, same commit: the contract (Purpose list, Signature block `enum OrganKind { … }` and `ALL: [OrganKind; N]`,
   the `classify` region in Registration point), the Organ kind row of "Registered plugins" in
   `wiki/architecture/module-map.md`, and `wiki/biology/sensing.md` (kind list and the `classify` thresholds).

## Done when

- The contract's test recipe passes (`cargo test organs::`, `cargo test biology::genome::`).
- The project gate is green: `cargo fmt --check`, `cargo clippy --all-targets --features runner -- -D warnings`,
  `cargo test`, `npm test`, `npm run lint`, `npm run build`, `node scripts/check-links.mjs`, and the four CI
  reference runs match their committed reports (commands: contract, Test recipe).
- Kind without a `classify` region: golden master and all references unchanged. Kind with a region: new
  `GOLDEN_FINGERPRINT`, the changed references updated, `MODEL_VERSION` bumped, stated in the ticket or commit.
