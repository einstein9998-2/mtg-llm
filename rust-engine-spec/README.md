# Rust engine spec tests (rulings-as-spec)

Executable scenarios for the Phase 2 Rust engine, written BEFORE the engine, from the Comprehensive Rules (effective 2025-11-14), pinned Oracle text and (where recalled, marked unverified) official rulings. Expectations never come from Forge output or from the engine. The spec-writer thread writes these files; the implementer thread (`rust-engine/`) never edits them and never sees the sealed holdout.

## Layout

| path | what |
|---|---|
| `SCHEMA.md` | scenario format (v1). The long "Addenda from review" section at the end settles format and decision-shape questions; read it fully. |
| `schema/scenario.schema.json`, `tools/validate.py` | structural and semantic validation: `python3 tools/validate.py scenarios` (also checks that every `verified: true` CR quote appears verbatim in `reference/cr-excerpts*.md` and every verified Oracle quote in `reference/oracle.json`). |
| `reference/oracle.json` | pinned Oracle text for 141 cards (the pool of all eight decks except five multi-face cards, plus a few support cards), tokens and the three dungeons. Five multi-face cards have no retrievable text (see `_meta.missing_text`) and have no scenarios. |
| `reference/cr-excerpts*.md` | CR rule text retrieved verbatim, the only source a `verified: true` CR citation may use. |
| `scenarios/` | the VISIBLE set (about 70% of every file). `core/` rules, `cards/` per-card, `matchups/` UR Cutter vs Alurentell, `hidden-info/` observation and mask tests, `exemplars/`. |
| `holdout/` | `MANIFEST.json` (counts per file and sha256 of each held-out scenario) and `README.md` (rules). |
| `sealed/` | AES-256-CBC encrypted held-out scenarios and a full-pool backup. The key is NOT in the project folder. |
| `COVERAGE.md` | counts only (never names a held-out id). |
| `OPEN-QUESTIONS.md` | rules questions the CR and Oracle text do not settle, doc corrections, assumptions. |
| `tools/` | `split_holdout.py`, `unseal_holdout.py` (spec-writer/human only), `coverage.py`. |

## For the implementer

1. Fix failures on their merits; do not special-case scenario ids or setups. Doc 04 section 4.4: holdout failures reveal only the card or rules area.
2. Scenario mode has no auto-resolution: every decision (including pass-only priority) is a script step, except decisions the rules do not give the player (see the addenda: empty attack/block declarations, trivial damage assignment, one-exit venture rooms, identical-outcome replacement ordering).
3. Decision SHAPES (for example `yes_no` then `choose_cards`) are the runner adapter's job; the scenarios fix who decides and the rules outcome. If a shape in SCHEMA conflicts with your API, tell the spec-writer; do not edit the scenarios.
4. A scenario you believe is wrong: report it with the CR text; the spec-writer decides. A wrong spec is worse than a missing one.

## Status

2026-10-01: 1158 scenarios written, each independently reviewed by a separate agent (reviewers edited about 190 scenarios: wrong expectations, missed triggers from permanents in the setup, priority and timing errors, unsupported citations, non-standard spellings): 806 visible, 350 sealed (three scenarios revealed by the first holdout run were promoted to visible and replaced by fresh sealed ones; see holdout-run-2026-10-01.md and holdout/README.md).
- Wave 1: core rules, matchup 1 (UR Cutter vs Alurentell), hidden-information tests.
- Wave 2: per-card scenarios for the other six decks (Boros, BW Death and Taxes, Dimir Tempo, UWx Control, Doomsday, Reanimator) and interaction cases suggested by the Forge interaction tests (casting restrictions, alternative costs, escape, Cavern of Souls, Phelia, seat-swap symmetry).
- Wave 3: matchup 2 (Boros vs BW D&T), matchup 3 (Dimir Tempo vs UWx Control), Magus of the Moon and layers, first strike, stack and Stifle sequences, copies, turn-structure edges.
- Next: Reanimator/Doomsday matchup lines against the non-combo decks, remaining cards with no scenario yet, and replacing weak probes found in review.

## Verification limits

Gatherer, Scryfall and MTGJSON cannot be reached from the sandbox (organization egress policy), so official card rulings could not be retrieved. Oracle text and CR text come from the Savecraft MCP (Scryfall oracle data, CR 2025-11-14). A scenario resting on a recalled ruling is `verified: false`; a handful do (each also has CR support or is flagged). The validator does not check the contents of `expect.obs` or `world_b`; those were checked by hand at review.
