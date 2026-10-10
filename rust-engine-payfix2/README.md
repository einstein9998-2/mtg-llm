# payfix2 on main (RFC 0016)

Made 2026-10-10 so that main matches the engine the Tron and Alurentell matches run on. The changes themselves come from the "Match game log for review" thread (`match-logs/tooling/pay-named-sources-then-pool.patch`, `sunbaked-canyon-mana-ability.patch`).

## Files
- `0016-payfix2-reconcile.md`: the RFC (what changes, impact, tests).
- `payfix2-core.patch`: `patch -p1` inside `rust-engine/`. Four files: `crates/mtg-core/src/legal.rs`, `crates/mtg-cards/cards/boros.cards.ron`, `crates/mtg-spec/src/runner.rs`, `own-scenarios/scenarios/own.yaml`. Apply it after the tail-swipe (#20), tron-sideboard (#21) and paypick (#22) packages: it replaces the `legal.rs` hunk of #22. Checked by applying all three to an untouched copy of the live engine, then this patch (clean).
- `goldens-real/`: the two real-pool records that this change re-records (`r00`, `r07`). Copy over `rust-engine/goldens-real/`.
- `scenarios/`: the seven match-thread scenarios (already in `own.yaml` inside the patch).

## Results
Spec (all packages) 1352/1352, own 39/39, workspace tests green except the existing 8-versus-9-decks `legacy_fuzz` assertion, 6000-game fuzz clean, `real_pool_goldens_replay_bit_identically` passes with the two re-recorded records.

## Status
Not applied to the live engine by this PR (the live folder is not writable from this thread). The binaries built from this stack plus the match-thread harness are in `/mnt/project-files/rust-engine-tron-payfix2/`.
