---
sources: [wiki/contracts/organ-kind.md]
---

# Guide: add an organ kind

Read first: [[wiki/contracts/organ-kind]] and [[wiki/architecture/invariants]].

## Steps

1. `src/biology/organs/<kind>.rs`: write `live_channels`, `sense` (or point to `spectral::*` / `contact::*`) and
   `pub const SPEC: OrganSpec` (names, channel names, `distance`, cost, limb, ballast, Latin suffix and rank).
2. `src/biology/organs/mod.rs`: `pub mod <kind>;`, a variant in `OrganKind` and `OrganKind::ALL`, an arm in
   `OrganKind::spec`, and its region in `OrganKind::classify` (otherwise no genome expresses it).
3. Config keys: none unless the kind has a tunable; then follow `wiki/guides/add-config-key.md`.
4. Tests: `test_live_channels_match_sense` covers the kind automatically; add a unit test for any non-trivial `sense`.
5. Run `cargo test`; it regenerates `bindings/OrganKind.ts` and `bindings/organRegistry.ts` (invariant 7), where the
   frontend's names come from. Add a painter for the kind to `ORGAN_PAINTERS` in `frontend/render/organs.ts`
   (`Record<OrganKind, …>`, so `npm run build` fails without it).
6. Docs: add the kind to the purpose list of the contract and to `wiki/biology/sensing.md`.

## Done when

- `cargo test organs::` passes.
- `cargo test`, `npm test`, `npm run build` and `node scripts/check-links.mjs` are green.
- If genomes can classify into the kind (step 2 region), the golden master and trainer references changed:
  intended model change, new fingerprint and references, `MODEL_VERSION` bump (invariants 8, 9). Otherwise unchanged.
