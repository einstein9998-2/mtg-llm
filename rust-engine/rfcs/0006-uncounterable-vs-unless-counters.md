# RFC 0006: "Counter unless its controller pays" must not counter a spell that can't be countered

Status: in review (patch NOT applied to the live engine; awaiting Brady's sign-off)
Author: "Daze vs Veil of Summer" thread  Reviewers (two independent, one a rules reviewer): pending

## Problem
Two LLM players in the LLM-vs-net benchmark (games n211, n219 in `llm-benchmark/game-logs/`) saw Daze still asking for payment while Veil of Summer ("Spells you control can't be countered this turn") was active. Asking is legal. The real defect is that the engine then **counters the spell anyway** when the payment is declined or impossible.

Cause: `counter_unless` (`mtg-core/src/resolve.rs`) ends in `counter_stack_obj`, which counters any spell without calling `Cx::can_be_countered` (`legal.rs`). Every other counter effect (`CounterSpell`, `CounterSpellExile`, `CounterAny`, `CounterSpellOf`) does call it. Affected paths:
- `Effect::CounterUnless`: Daze, Flusterstorm.
- `Effect::CounterEventUnless`: Koma's ward {4}.
All three sources of "can't be countered" are ignored on those paths: Veil's `PlayerFx::SpellsUncounterable`, Cavern of Souls mana (`CastInfo::uncounterable`) and the `UNCOUNTERABLE` keyword (Koma). It is wrong on an explicit "no" and on the implicit decline when the controller has no mana to pay.

Not affected (checked in code and by scenario): Force of Will, Hydroblast, Pyroblast, Force of Negation (exile branch too), Consign to Memory, Lavinia's trigger, Stifle (abilities only, so "can't be countered" on spells does not apply). Paying the cost already worked.

## Proposed change
One edit in `counter_stack_obj` (`resolve.rs`): the spell branch becomes `else if cx.can_be_countered(victim)`. The ability branch is untouched (abilities are not spells; uncounterable effects only cover spells). Patch: `0006-uncounterable-vs-unless-counters.patch` next to this file. The payment prompt is deliberately kept (see Rules basis), so the decision sequence for the players is unchanged.

## Rules basis (verified via Savecraft, CR effective 2025-11-14; card text from Scryfall data)
- Daze: "Counter target spell unless its controller pays {1}." Veil of Summer: "Spells you control can't be countered this turn."
- CR 118.12a: "[Do something] unless [a player does something else]" means "[A player] may [do something else]. If [that player] doesn't, [do something]." So the payment is still offered; only "do something" (counter) is impossible.
- CR 101.2: "can't" beats "can". CR 701.6a defines countering as cancelling a spell. Veil's effect is a rules modification, not a replacement effect (spec scenario `veil-of-summer-in-response-makes-force-of-will-do-nothing`).
- Open design choice (not a rules question): the engine could skip the useless prompt for an uncounterable spell, since paying has no effect. Rules-faithful is to keep it (a player may pay). Skipping would remove a pointless decision for LLM and net players; it would change the decision stream and the new scenarios that answer the prompt. Default here: keep it. Say the word and I'll skip it in a follow-up.

## Impact
- State layout and `hash_rules` / `hash_full`: none.
- Determinism and golden replays: none changed (goldens and `cargo test --workspace --release` pass unchanged on the patched copy). Only games where an unless-counter targets an uncounterable spell differ; older records of such games are wrong and should be regenerated if they matter. LLM benchmark games n211 and n219 are such cases (the benchmark used a pinned engine copy, so its published numbers stand, but note the two games).
- Hidden information: none.
- Effect VM / IR: no new variants.
- `ENGINE_CORE_VERSION` bump: no (goldens bit-identical), same reasoning as RFC 0005. Brady may want a bump for bookkeeping.

## Test plan (done on a scratch copy of the live engine with the patch)
- 14 scenarios written by a separate agent (spec writer) from card text and CR only, before the fix: `rust-engine-spec/scenarios/cards/uncounterable-*.yaml`, report `rust-engine-spec/uncounterable-report-2026-10-06.md`. Before the patch: 9 failed (Daze hard cast, Daze alt cost, Veil in response, Flusterstorm, Daze vs Cavern creature, Daze vs Koma, tapped-out Daze, two Koma ward cases), 5 passed (pay case, FoN, Pyroblast, Lavinia, Consign).
- After the patch: visible spec 838/838 (824 existing + 14 new), `cargo test --workspace --release` green.
- Not run: legacy fuzz, differential tests against Forge (no Forge jar here), sealed holdout.
