# RFC 0004: Lower object cap for token explosions, plus the review follow-ups of RFC 0001 to 0003

Status: in review (gates green: suite, spec 806, own 32, 1000-game fuzz identical to m5; rules review requested from the spec thread, rerun from the differential thread)
Author: Rust engine core thread  Reviewers (two independent, one a rules reviewer): spec thread (rules), differential thread (rerun), hidden-information/determinism reviewer (re-read)

## Problem
Report: /mnt/project-files/ENGINE-NOTE-token-explosion.md (self-play on m5, Boros Aggro vs BW Death and Taxes, game 1405, seed 102): a game reached 2,063 Cat tokens from Ocelot Pride and then never finished, because every derived-state refresh and every search fork walks all permanents (cost at least quadratic in the permanent count). The 60,000-object cap of M3g is far too high to help and the step cap of RFC 0003 does not apply (no single resolution loops).

Is it a rules bug? No. Checked with probe scenarios on m5: four Ocelot Prides with the city's blessing and a life gain this turn create exactly 30 Cat tokens at the end step (each trigger creates one token, then copies every token that entered this turn: 2, 6, 14, 30), which is what the card says, and tokens from earlier turns are not copied again (a board of 30 older tokens ends at 60, not more). The explosion happens when many tokens entered the same turn (other token makers, or several turns of play): then each Pride multiplies that number again (80 tokens that entered this turn give well over a thousand). That is legal, degenerate play, not an engine error, and it is reachable by random or search-guided play in Boros Aggro. The scenarios `own-ocelot-four-prides-make-thirty-tokens` (new, passes on m5 too) and `own-ocelot-token-explosion-ends-as-a-draw` record this.

Also from the m5 hidden-information/determinism review (four small items, none reachable in the pool today).

## Proposed change
`mtg-core` and one test.
- `RUNAWAY_OBJECTS` 60,000 to 1,000: a game with 1,000 live objects is declared a draw (same mechanism as M3g). No real game or pool deck comes near a few hundred objects. Documented as a deviation (the rules have no such limit).
- `sac_candidates` calls `refresh()` first (with two Quarries it could read a stale derived cache).
- `pay_plan`'s avoid list for the automatic Quarry sacrifice also contains the cost's chosen picks and the source permanent.
- Documentation comment on `obj_key`: battlefield view ids are assigned in entry order for both seats, so the canonical order is the same for either seat; `sac_pick`, `attack_candidates`, `target_key` rely on it.
- The Show and Tell fork test compares fork rates within opponent-hand-size strata instead of unstratified (the unstratified rates differ because hand size drives both; no leak).

## Rules basis
None (robustness). The draw on a runaway is a deviation from CR (no such limit), recorded in CORE-FREEZE-REVIEW.md next to the 4,000,000-step and arena caps.

## Impact
- State layout and hashes: none. `HASH_SCHEMA` stays 4; `ENGINE_CORE_VERSION` 5 to 6 (behavior change for games above 1,000 objects).
- Determinism and golden replays: goldens re-recorded for the header only (check: action sequences and checkpoint hashes unchanged).
- Hidden information: none.
- Effect VM / IR: none.

## Test plan
Own scenarios (two new Ocelot ones above, 32 total), spec visible 806, workspace suite (includes the arena-exhaustion draw test, now at the new cap, and the stratified Show and Tell test), 1000-game legacyfuzz, differential and sealed reruns requested.

## Not done
Making the derived-state refresh incremental (the quadratic cost) is a performance project, not a correctness fix; with the cap at 1,000 the worst refresh is cheap. Tell the engine owner if search needs more speed on large boards.
