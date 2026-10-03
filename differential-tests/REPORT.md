# Differential testing: Rust engine vs Forge (Phase 2 gate, M4 first pass)

Date: 2026-10-01. Engine: mirror of `rust-engine/` at M3f (`ENGINE_CORE_VERSION = 3`, snapshot taken 19:37Z), plus the same runs on the earlier M3e snapshot (`ENGINE_CORE_VERSION = 2`). Forge: checkout `153b545f`, headless. The engine source was not modified. Method and layout: [README.md](README.md).

## Verdict

No unexplained step, combat or action-set divergence between the engine and Forge in about 25,000 sampled positions on M3f. One real engine bug is still open (E3, a repro fails). Two engine bugs found earlier (E1, E2) were already fixed upstream. Forge-side differences and harness limits are registered as known-divergence entries, all `proposed` or `scope-cut`; none is `accepted`, because doc 04 section 6.4 requires a human to approve Forge-bug, version-divergence and spec-ambiguity verdicts.

This is a first pass with a simplified method (single-step comparison from injected positions, see Limits), not the full lockstep of doc 04 section 5. It does not by itself clear the Phase 2 gate.

## Coverage (M3f engine, seeds in the last section)

| Kind | Positions | Same | Real mismatch | Unaligned (harness) | Forge could not play |
|---|---|---|---|---|---|
| Priority action (normal) | 13,276 | 13,272 (incl. 3 history artifacts, 14 payment-noise suppressed) | 0 | 1 | 3 |
| Combat run (attack, Forge AI blocks, to main 2) | 8,220 | 8,186 (77 history artifacts) | 0 | 34 | 0 |
| Stack response (opponent answers a spell) | 3,577 | 3,568 | 0 | 0 | 9 |
| Action sets at those positions (normal) | 13,276 | 13,276 identical | 0 diffs | | |
| Action sets at response windows | 3,568 | 3,568 identical | 0 diffs | | |

Also: the 233 interaction scenarios on the engine give 210 pass, 22 partial, 1 fail on both M3e and M3f ([m3f/scnrun.log](results/m3f/scnrun.log)). The one failure is Flow State, where the scenario's expectation follows Forge (kd-0004).

M3e results (same seeds) were equivalent: 0 real mismatches, action sets identical. The M3e logs are in `results/m3e/`.

All 12 positions Forge could not play are explained: 10 involve Aluren (kd-0002, Forge's known seat-order bug), Ancient Tomb with a player at 5 life or less (kd-0006), or both, and 2 are Lazotep Quarry's X ability, whose X-dependent targets the probe cannot supply.

## Findings

| # | Verdict | What | State |
|---|---|---|---|
| E1 | engine bug | Tamiyo, Inquisitive Student flipped on the wrong draw count (third draw of the turn) | fixed upstream in M3d, repro passes ([e1](repros/e1-tamiyo-third-draw.scn)) |
| E2 | engine bug | Voice of Victory Mobilize tokens | fixed upstream in M3d/M3e, repro passes ([e2](repros/e2-voice-of-victory-mobilize.scn)) |
| E3 | engine bug, open | Lazotep Quarry: tapping it while sacrificing a creature adds one mana of any color. Forge counts this when deciding which spells are castable (Plains + Quarry pays for Goblin Bombardment or Amped Raptor); the engine offers no cast, though activating the Quarry manually works. Doc 01 line 272 puts costed mana abilities in the payability solver | repro [e3](repros/e3-quarry-sacrifice-mana-cast.scn) still fails on M3f. Found in an earlier exploratory batch (12,498 positions: 13 raw action-set diffs, 9 covered by known harness cases, 4 all Quarry). The final seeds did not hit it. **Needs a fix by the engine thread.** |
| kd-0002 | Forge bug (known) | Aluren with the second seat as actor (known Forge seat-order bug, see interaction-tests/FINDINGS.md). Rust-only casts with Aluren on the battlefield are suppressed | proposed |
| kd-0003 | Forge bug | Bilbo's discount is applied in Forge's hand-spell affordability pre-check, so Forge offers a hand cast it cannot pay. Confirmed by playing it for Kaito, inferred for Force of Will | proposed |
| kd-0004 | Forge bug | Flow State counts itself as a sorcery in the graveyard while it resolves (CR 608.2n says it is still on the stack) | proposed, carried as an xfail scenario |
| kd-0005 | harness scope cut | Casts from exile (warp, adventure) depend on permissions that cannot be injected | scope-cut |
| kd-0006 | harness artifact | Forge's AI payer will not tap Ancient Tomb at 5 life or less, which hides casts that need it | scope-cut |

Entries are in [known-divergences/](known-divergences/); each lists its CR citation and the revisit condition. Suppression counts are printed per entry in the compare logs.

## What the harness still cannot see (limits)

- Single step from an injected position: no multi-step sequences, no live lockstep, no randomness leader/follower.
- Positions with a non-empty stack at the start (except the response kind), mid-combat state, energy, mana in pool, tokens, attachments, chosen-state permanents (Doomsday-style piles, chosen names), dungeon/emblems, impending permanents, stolen permanents and spells-cast-this-turn counters are skipped when exporting. Skip counts per reason are in each `posgen.err`.
- Durable effects and delayed triggers cannot be given to Forge. A difference that disappears when Rust is rebuilt from the injected text is classed `history_artifact` (3 + 77 + 1 here); these are not checked.
- Combat `unaligned` cases (34, 0.4%) are choices the follower could not mirror: attack-trigger targets not logged, discard-chosen, a forced-attack token, a block Forge made that Rust did not offer. They are counted, not compared. The block-offer one ("Forge blocked ...") is worth a look by the engine thread; I did not isolate it.
- Cost and target choices use canonical rules on both sides (smallest name, first legal mode), so they are not checking AI quality; Nethergoyf escape and X-dependent targets are not modeled.
- No mulligan, sideboarding or game-start checks; no XMage third opinion (doc 04 section 7).
- The Forge probe leaks memory, so runs are chunked (800 positions, 3 JVMs).

## Reproduce

Seeds (posgen args `<decks> <games> <seed0> <cap> [mode]`): response 6000 games seed 110000; combat 7000 games seed 80000; normal 8000 games seed 90000 sampled to 15000 (the sample is all positions that have an action). Position ids (`<deckA>-<deckB>-s<seed>-d<decision>[-combat|-resp]`) in each `status.tsv` regenerate any case. Commands are in the README. Per-position results: `results/<engine>/<kind>/status.tsv`; summaries: `stepdiff.txt`, `compare.txt`, `resp-actions.txt`.

## Needs a human

1. Approve or reject the `proposed` entries kd-0002 to kd-0004 (Forge-side verdicts).
2. Decide whether the Phase 2 gate should demand the full lockstep of doc 04 section 5 before sealing. My recommendation is yes for multi-step interactions (Doomsday, Show and Tell, Aluren chains), which single-step positions do not exercise.
3. E3 goes to the "Rust engine core" thread.
