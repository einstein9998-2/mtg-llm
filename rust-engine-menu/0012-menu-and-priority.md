# RFC 0012: menu text, mana-source choice hook, no pointless Daze prompt

Status: applied to the live engine 2026-10-08 (Brady asked to fix the engine and menu issues from the overnight postboard summary)
Author: match thread

## Problem
The 54 overnight games (see `match-logs/overnight-waves/SUMMARY.md`) showed menu text that left players guessing:
- Activated abilities were listed as "Activate Stormchaser's Talent ability 2" with no rules text, so a level-up looked like it was never offered.
- Delve prompts did not say that cards are exiled one at a time or when Done appears.
- Autopay gave no choice of mana source (Karakas, Lazotep Quarry, the wrong dual).
- Daze still asked the opponent to pay when the spell could not be countered (Veil of Summer).

## Change
`mtg-view` and one line of `mtg-core` only:
- `project.rs`: `Activate` labels now end with the ability's Oracle text, cut at 80 characters. A Delve `ChooseCards` prompt gets a context line ("exile graveyard cards one at a time, each one pays for one generic mana; Done is offered once the rest of the cost can be paid").
- `game.rs`: `Game::set_pay_hint(seat, view_ids)` maps seat-view ids to objects and calls the existing `State::scenario_pay_hint`. The harness uses it so the player can name the lands that pay (`pick TAG SEQ IDX pay=Tundra,Island`). The engine already prefers hinted sources and falls back to the others when they cannot pay, so a payable cast stays payable.
- `resolve.rs` (`counter_unless`): when the victim cannot be countered, return before asking the payment question. Paying could not change anything.

## Not changed
- The harness (`llmgame.rs`, `lg.sh`) carries the rest: combat priority windows on the opponent's turn, shortened Lazotep Quarry lists with `lg.sh show TAG full`, `lg.sh wait`, the `pay=` parser and the `.pay` sidecar for replays. Those live in `match-logs/tooling/` in the shared folder, not in this package.
- Force of Will auto-picking its pitch card when only one blue card is in hand, FoW being offered at 1 life or under Veil, duplicate card names without ids in some prompts, and a spare-mana line in the Daze prompt were left alone.
- Reported Acererak bounce bug: not reproduced by reading the code. Object references carry a generation that changes on every zone move (`ops.rs`), and `eval_o` rejects a stale reference, so an old trigger cannot bounce a later Acererak.
- Stormchaser's Talent level-up is offered (ability 2, sorcery timing, needs level 1); only its label was opaque.

## Tests
`cargo test --release --workspace` passes. Two full two-player games driven at random exercised the new labels and windows; one of them with a deck of Lazotep Quarries showed the shortened list. A replay of an old game (w12 g2) through the new binary matches the old binary except for the longer Activate labels.
