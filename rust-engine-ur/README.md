# UR Cutter alternate for the Rust engine (RFC 0009, APPLIED to the live engine)

Made 2026-10-08 for Brady's alternate UR Cutter 75 (`decks/ur-cutter-alt.txt`). Read `0009-ur-alternate.md` first.

## Files
- `0009-ur-alternate.md`: the RFC (changes, limits, rules basis, impact, results).
- `ur-core.patch`: `patch -p1` inside `rust-engine/` (12 files: Class levels, divided-damage target slots, subtypes, scenario-setup counter fix, spec adapter, `ur.cards.ron`).
- `ur.cards.ron`: the card data (Stormchaser's Talent, Otter Token, Pyrokinesis, Price of Progress); already inside the patch, kept here for reading.
- `scenarios/`: 54 spec scenarios by two separate agents (A 25 Talent; B 29 Pyrokinesis, Price of Progress, Boomerang Basics). Notes in `SCENARIO-NOTES-*.md`, extra Oracle text in `oracle-additions-*.json`, rule quotes in `cr-excerpts-ur-*.md`, the writers' instructions in `SCENARIO-BRIEF.md`.

## Applied
Applied to `/mnt/project-files/rust-engine` on 2026-10-08 (Brady asked to start mixing in the Stormchaser's Talent deck). The card database hash and card count changed: nets and game records made against the previous live pool must be regenerated. Goldens replay bit-identically; `ENGINE_CORE_VERSION` is unchanged.

## Scenarios
`tools/spec2json.py` takes a spec dir with a `scenarios/` folder; symlink the spec folders plus `rust-engine-{tron,storm,orims-chant,ur}/scenarios` into one, then `specrun all.json`.
