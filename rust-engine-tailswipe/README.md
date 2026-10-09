# Tail Swipe for the Rust engine (RFC 0013)

Made 2026-10-09 for Brady's postboard Alurentell tests ("add tail swipe to the game engine and test out some postboard games with it vs UR").

## Files
- `0013-tail-swipe.md`: the RFC (changes, rules basis, impact, test results).
- `tailswipe-core.patch`: `patch -p1` inside `rust-engine/` (IR: `Effect::Fight`, `Cond::InMainPhase`; the card in `legacy.cards.ron`). Dry-run clean against the live `/mnt/project-files/rust-engine` and against the repo's `rust-engine/`.
- `scenarios/cards/tail-swipe.yaml`: 5 scenarios written by the implementer (not a separate spec writer), checked with `tools/spec2json.py` and `specrun`.

## Status
Not applied to the live engine by this PR. Games with the card were played from a scratch build of live + this patch.
