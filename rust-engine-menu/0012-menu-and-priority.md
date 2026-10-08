# RFC 0012: menu text and mana-source choice hook

Status: applied to the live engine 2026-10-08 (Brady asked to fix the engine and menu issues from the overnight postboard summary)
Author: match thread

## Problem
The 54 overnight games (see `match-logs/overnight-waves/SUMMARY.md`) showed menu text that left players guessing:
- Activated abilities were listed as "Activate Stormchaser's Talent ability 2" with no rules text, so a level-up looked like it was never offered.
- Delve prompts did not say that cards are exiled one at a time or when Done appears.
- Autopay gave no choice of mana source (Karakas, Lazotep Quarry, the wrong dual).

## Change
`mtg-view` only:
- `project.rs`: `Activate` labels now end with the ability's Oracle text, cut at 80 characters. A Delve `ChooseCards` prompt gets a context line ("exile graveyard cards one at a time, each one pays for one generic mana; Done is offered once the rest of the cost can be paid").
- `game.rs`: `Game::set_pay_hint(seat, view_ids)` maps seat-view ids to objects and calls the existing `State::scenario_pay_hint`. The harness uses it so the player can name the lands that pay (`pick TAG SEQ IDX pay=Tundra,Island`). The engine already prefers hinted sources and falls back to the others when they cannot pay, so a payable cast stays payable.

## Not changed
- The harness (`llmgame.rs`, `lg.sh`) carries the rest: combat priority windows on the opponent's turn, shortened Lazotep Quarry lists with `lg.sh show TAG full`, `lg.sh wait`, the `pay=` parser and the `.pay` sidecar for replays. Those live in `match-logs/tooling/` in the shared folder, not in this package.
- Daze after Veil still asks the payment question; RFC 0006 scripts that prompt (8 spec scenarios), so it stays. An early return in `counter_unless` was tried and dropped for that reason.
- Force of Will auto-picking its pitch card when only one blue card is in hand, FoW being offered at 1 life or under Veil, duplicate card names without ids in some prompts, and a spare-mana line in the Daze prompt were left alone.
- Reported Acererak bounce bug: not reproduced by reading the code. Object references carry a generation that changes on every zone move (`ops.rs`), and `eval_o` rejects a stale reference, so an old trigger cannot bounce a later Acererak.
- Stormchaser's Talent level-up is offered (ability 2, sorcery timing, needs level 1); only its label was opaque.

## Tests
`cargo test --release --workspace` passes and `specrun` passes 838 of 838 scenarios. Two full two-player games driven at random exercised the new labels and windows; one of them with a deck of Lazotep Quarries showed the shortened list. A replay of an old game (w12 g2) through the new binary matches the old binary except for the longer Activate labels.
