# Lockstep differential: Rust engine vs Forge (multi-step lines)

Date: 2026-10-01. Engine for the sections below: `core-frozen-m3h` (`ENGINE_CORE_VERSION = 3`, RFC 0001 applied), identical to `rust-engine/` in the project files. Forge: checkout `153b545f`, headless. Engine source not modified. Single-step results from the first pass are in [REPORT.md](REPORT.md); this report adds the multi-step ("line") method that REPORT.md recommended.

# Rerun on core-frozen-m5 (RFC 0003), 2026-10-02

Engine `core-frozen-m5` (`ENGINE_CORE_VERSION = 5`, hash schema 4: loop counter fix, resolution step cap, Quarry auto-payment). **No new divergences.**

- **Lockstep:** 8,749 lines (random starts regenerated on m5), 50,215 windows, 41,595 actions. The statuses and the six diverged lines are identical to m4 (3 E5 = kd-0012, 1 kd-0007, 2 known harness gaps); 0 unexplained menu differences in scenarios and random, 1 in combos (the kd-0007 case). The E3, E6 and E7 repros still pass; E5 and f1 fail as documented. Interaction scenarios: 210 pass, 22 partial, 1 fail (Flow State, kd-0004), the same as before.
- **Single-step:** normal 5,761 positions (5,758 same, 1 unaligned, 2 Forge could not play; action sets identical, 0 diffs), combat 3,983 runs (3,974 same, 9 unaligned), responses 1,742 (1,739 same, 3 Forge errors; action sets identical at 1,739 response windows). Seeds as in REPORT.md (normal 90000, combat 80000, response 110000) with fewer games (3,500, 3,500, 3,000), so this is a smaller sample than the M3f pass.
- The 10 unaligned single-step cases are the known harness classes, not engine differences: 1 identically named legendary copies (Karakas play in step mode), 8 Bilbo attack-trigger discards the follower cannot mirror, and 1 block Forge made that the engine did not offer (a non-flyer blocking a Guide of Souls carrying flying counters). The last one was noted in REPORT.md and still looks like a Forge-side injection effect (counters injected without the flying keyword), unconfirmed.
- Results: [results/m5-lines/](results/m5-lines/) and [results/m5/](results/m5/).

## Decisions (Brady, 2026-10-01, project chat: "yes to all")

- E4 (Cavern of Souls manual tap, kd-0010) and E5 (Animate Dead extra empty trigger, new entry kd-0012) are **accepted as documented limits**.
- kd-0002, kd-0003, kd-0004 and kd-0007 to kd-0011 are **accepted** (Forge is the one that is wrong; kd-0010 is a design verdict). kd-0005 and kd-0006 stay scope-cut.
- The **Phase 2 gate is passed**.
- The E5 repro is expected to fail from now on (it documents the limit).

# Rerun on core-frozen-m4 (RFC 0002), 2026-10-01 23:30Z

Engine `core-frozen-m4` (`ENGINE_CORE_VERSION = 4`, hash schema 3), same harness and seeds. **E6 and E7 are fixed: both repros pass** ([e6](repros/e6-two-quarries-one-victim-cast-offer.scn), [e7](repros/e7-ajani-pariah-triggers-on-non-cat.scn)), and the random lines that exposed them (seeds 700162, 701366, 700529, 702185) now complete. E3's repro still passes. E5 still fails as expected (documented limit, proposed pending Brady). E4 is unchanged (design difference, kd-0010). The Flow State repro f1 is the known xfail (kd-0004).

Totals on m4: 8,749 lines, 50,215 windows, 41,595 actions (random starts were regenerated on m4, so they are 4,245 lines instead of 4,234; recorded random traces from m3h no longer replay on m4 because the E6/E7 fixes change option lists, which is expected). Results: scenarios 1,840 of 1,864 complete or game over, 10 unaligned, 14 Forge could not play, 0 diverged; combos 2,626 of 2,640, 4 diverged, 8 unaligned, 2 Forge could not play; random 4,187 of 4,245 (including 8 `history_artifact`), 2 diverged, 36 unaligned, 20 Forge could not play. Menu differences: 0 unexplained in scenarios and random; 1 in combos (the kd-0007 Boggart Trawler graveyard play).

The 6 remaining diverged lines are all explained: 3 are E5 (kd-0012: reanimator-02 l3 and l6, reanimator-08 l3), 1 is kd-0007 (aluren-chain-02 l2), and 2 are harness gaps already described under limits (two Force of Will on the stack, and the Kaito enters-tapped replacement order). There is no new finding. No engine bug is open from this method except E5 and the E4 design point.

The m3h results are kept in [results/lines-m3h/](results/lines-m3h/); the sections below describe the m3h run.

## Verdict

The engine and Forge stay aligned through multi-step lines almost everywhere. Out of 8,738 lines (50,161 priority windows, 41,553 actions played) there are **no unexplained state divergences and no unexplained action-set differences**. Every difference that remains is one of:

- **3 new engine findings** (E5 to E7) plus 1 design difference (E4), each with a minimal repro in [repros/](repros/).
- **Forge-side differences**, registered as known divergences kd-0007 to kd-0011 (all `proposed`; none is `accepted`, since doc 04 section 6.4 needs Brady's approval for Forge-bug and design verdicts).
- **Harness limits** listed below (counted, not compared).

Findings E4 to E7 are in code frozen since core-frozen-m3h, so fixing them needs an RFC from the "Rust engine core" thread. None changes who wins a line in the pool decks except E7 (a wrong-flip option for Ajani) and E6 (a missing cast offer).

## Coverage

Each line is played by Forge's engine under a shared deterministic hash policy: both seats pick from the same engine-enumerated menu (pass 50% with a non-empty stack, otherwise about 5 to 18%, 75% bias toward the cards in hand), up to 12 priority windows. The engine follows Forge's decisions window by window (same card, same targets, same choices) and the harness compares at every window: priority seat and stack size, full canonical state (life, hand, battlefield with tapped/counters/damage, graveyard, exile), library contents and top-3 order, and the action menu. Games that end inside a line also compare the final life and winner.

| Set | Start states | Lines | Windows | Actions played | Complete or game over | Diverged | Unaligned (harness) | Forge could not play |
|---|---|---|---|---|---|---|---|---|
| Scenario starts | 233 interaction scenarios, 8 seeds each | 1,864 | 7,712 | 5,916 | 1,840 | 0 | 10 | 14 |
| Combo starts | 110 hand-built multi-step states, 24 seeds each | 2,640 | 21,923 | 19,316 | 2,626 | 4 | 8 | 2 |
| Random-game starts | main-phase windows of 2,500 random games between the 8 decks | 4,234 | 20,526 | 16,321 | 4,167 (plus 7 `history_artifact`) | 6 | 36 | 18 |
| **Total** | | **8,738** | **50,161** | **41,553** | | | | |

Combo families (24 seeds each, deterministic states in [harness/lines/combos.scn](harness/lines/combos.scn)): aluren-chain 26 states, aluren-acererak 8, aluren-cast 6, aluren-seat2 4, show-and-tell 18, doomsday 16, reanimator 18, counter-war 14. Across all sets the lines cast Doomsday 234 times, Show and Tell 403, Acererak 400, Reanimate 224, Animate Dead 104, Surgical Extraction 93, Force of Will 437 and Aluren 71; 2,211 lines have three or more non-pass actions. Per-family results: aluren-chain 622 of 624 complete (1 diverged, 1 Forge could not play), aluren-acererak 192 of 192, aluren-cast 143 of 144, aluren-seat2 96 of 96, counter-war 336 of 336, doomsday 383 of 384, show-and-tell 425 of 432, reanimator 429 of 432 (3 diverged, all E5).

Seeds: random starts use game seeds 700000 to 702499 (`linegen random /mnt/project-files/decks 2500 700000 5 --max 12`), scenario and combo lines use line seeds 1000 + s. Libraries are compared at each window; where a shuffle or a random bottom order happens the harness scripts the engine's random choice to Forge's order (engine `scenario_script_random` hook) so the line continues.

## Findings

| # | Verdict | What | Repro | State |
|---|---|---|---|---|
| E3 | engine bug | Lazotep Quarry sacrifice mana in cast offers | [e3](repros/e3-quarry-sacrifice-mana-cast.scn) | **fixed on m3h**, repro passes |
| E4 | design difference, not a rules error | Cavern of Souls' restricted mana is not used by the engine's automatic payability check: with Island + Cavern (Merfolk named) the cast of Thassa's Oracle is offered only after the Cavern is tapped by hand. Forge offers it at once. Nothing is unreachable. | [e4](repros/e4-cavern-restricted-mana-cast-offer.scn) | kd-0010 `proposed`: engine thread to confirm it is intended; if so Brady accepts |
| E5 | engine deviation, low impact | Animate Dead's "when it leaves the battlefield" is a separate trigger in the engine, a delayed trigger created by the enters ability in Forge (`animate_dead.txt`). If the Aura leaves before its enters trigger resolves (target exiled in response, Surgical Extraction on Archon of Cruelty), the engine puts a second, empty trigger on the stack; Forge does not. One extra stack object and priority window, no change to game state. | [e5](repros/e5-animate-dead-leaves-trigger-without-etb.scn) | open; 3 lines (reanimator-02 l3, l6, reanimator-08 l3) |
| E6 | engine bug (same family as E3) | Two Lazotep Quarries and one creature: the cast of a two-mana spell is not offered (Quarry A sacrificing the creature for {R}, Quarry B for {C}). With sacrifice sources allowed, `mana_sources_ex` turns every Quarry into a sacrifice source ("one tap per permanent") and keeps only as many as there are victims, so the second Quarry is dropped. Forge offers and plays it. One Quarry plus a Mountain works. | [e6](repros/e6-two-quarries-one-victim-cast-offer.scn) | open; found in 2 random lines (seeds 700529, 702185) |
| E7 | engine bug | Ajani, Nacatl Pariah's trigger ("other Cats you control die") goes on the stack when a permanent that is not a Cat is sacrificed as a cost: Wasteland, a fetchland. Answering yes flips Ajani into Ajani, Nacatl Avenger with no Cat dead. Forge puts only the ability on the stack. Not checked: whether destroyed or damaged non-Cats also trigger it. | [e7](repros/e7-ajani-pariah-triggers-on-non-cat.scn) | open; found in 2 random lines (seeds 700162, 701366); repro fails on the engine alone |

E7 is the one most likely to matter: Boros Aggro runs Ajani and fetchlands, so a policy would see a flip option that the rules do not give it. Single-step runs missed E5 to E7 because they never reach a second stack object or an Ajani on the board together with a cost sacrifice, and E6 needs a particular land pair.

### Forge-side differences (all `proposed`, entries in [known-divergences/](known-divergences/))

| Entry | What | Seen in |
|---|---|---|
| kd-0002 | Aluren with the second seat as actor (known Forge seat-order bug) | 2,209 menu windows across all sets |
| kd-0003 | Bilbo discount applied in Forge's hand-spell pre-check | 3 |
| kd-0007 (new) | With Aluren on the battlefield and a spell on the stack, Forge lists a modal double-faced card's land face as playable and plays it, including from the graveyard (Boggart Trawler to Boggart Bog, line aluren-chain-02 l2) | 100 menu windows plus 1 diverged line |
| kd-0008 (new) | Deafening Silence: Forge's menu lists a second noncreature spell (Brainstorm, Ponder), the play then fails with `checkRestrictions refused` | 7 menu windows, 7 failed plays |
| kd-0009 (new) | Grafdigger's Cage: Forge lists Faithless Looting's flashback, the play fails | 8 menu windows, 7 failed plays |
| kd-0010 (new, E4) | Cavern of Souls restricted mana, see E4 | 8 |
| kd-0011 (new) | Voice of Victory: Forge lists Force of Will (alt cost) and Stifle for the opponent during the Voice player's turn, the play fails | 2 |

kd-0008, kd-0009 and kd-0011 belong with kd-0003: Forge's affordability pre-check ignores a restriction that the play itself enforces. In each case the engine is right. kd-0004 to kd-0006 are unchanged from REPORT.md.

## What is still not compared (limits)

- **Unaligned lines (54 of 8,738).** The follower could not mirror a Forge decision: 21 payments from different lands (a tapped-land name or life paid differs), cast-from-exile target choices Forge's AI makes without logging (Amped Raptor's free spell, about 10), replicate counts for Consign to Memory (4), Archon of Cruelty trigger targets, and a few single choices. These lines are counted and not compared past that window.
- **Same-name stack objects.** Targets on the stack are named by card name, so two Force of Will on the stack are ambiguous. One line (doomsday-alurentell seed 700812) diverges for that reason (rust stack 1, forge 0) and is classed as a harness gap.
- **Trigger and replacement order.** Forge's AI does not log the order it puts simultaneous triggers in. The harness retries the line with the other orders (up to 24) and accepts the first that matches, so an order-dependent bug could hide behind this. One line (Kaito cast under an opposing Spider-Woman, seed 700995) differs because Forge applied "enters tapped" before Kaito's loyalty counters and the engine did not ask for the order; this is a replacement-order choice (CR 616.1), not mirrored by the follower.
- **Identically named legendary permanents:** the line is retried over the pick combinations, as for triggers.
- **History Forge cannot be given** in random starts: loyalty abilities and once-per-turn abilities used earlier in the turn (the harness suppresses the resulting Forge-only menu entries, 4 windows), transformed permanents (now skipped at export), and spells cast earlier in the turn (skipped). Differences that vanish when the engine is rebuilt from the injected text are `history_artifact` (7).
- **Policy.** Lines never take shuffling actions (fetchlands, Prismatic Vista, Personal Tutor, Edge of Autumn, Boseiju), land plays with a spell on the stack (Forge bug kd-0007), playback of MDFC backs as separate keys, casts from exile, Nethergoyf escape, or multi-ability activations. Choices inside effects use the shared canonical rules (smallest name, first legal mode, opponent-first targets), so this does not test AI quality. Lines are at most 12 windows; no mulligans, no combat blocks beyond what a window sees.
- **Forge's own AI** makes the choices Forge makes (for example searches and scry); the engine follows them. A Forge-side bug in those choices would not show.
- **No XMage third opinion** (doc 04 section 7).

## Method notes

- Forge probe: `forge-probe/src/Probe.java` mode `line`; per window it records the seat, stack, phase, canonical menu, state summary, library order (top first, both seats), the chosen action and a log segment.
- Engine follower: [harness/crates/mtg-diff/src/line.rs](harness/crates/mtg-diff/src/line.rs) (`run_line`, library reconciliation) and `step.rs` (`run_step_ex`, which follows the Forge log). Binaries: `linegen`, `linediff`, `scnmenu` in [harness/crates/mtg-diff/src/bin/](harness/crates/mtg-diff/src/bin/).
- Ignorable Forge log entries in line mode: forced or single-candidate choices, a choice that takes every candidate, identical-name candidates, duplicates of used calls, summary calls that repeat single picks (Stock Up), `confirmAction` with yes, `payCostToPreventEffect` with no, `orderMoveToZoneList` to graveyard or exile.
- Follower fixes added during this run: restricted-mana tap before a cast (so Cavern lines continue), surveil/scry top and bottom order, Ponder-style shuffle then draw (library order scripted from Forge's remaining library plus the cards drawn), a shuffle that leaves identical library order no longer ends the line, duplicate zone keys in scenario states merged so both engines see the same state (the Skyclave Apparition scenario lists `humanbattlefield` twice and Forge kept only the last).

## Reproduce

```
cd harness && CARGO_TARGET_DIR=/tmp/target-diff cargo build --release --bins   # Cargo.toml points at ../../../../rust-engine
sh forge-probe/build.sh                                                        # needs the Forge jar and interaction-test classes
/tmp/target-diff/release/linegen scn interaction-tests/scenarios --seeds 8 > scn.jsonl
/tmp/target-diff/release/linegen scn harness/lines/combos.scn --seeds 24 > combos.jsonl
/tmp/target-diff/release/linegen random /mnt/project-files/decks 2500 700000 5 --max 12 > random.jsonl
TARGET=/tmp/target-diff/release OUT=<probe classes> bash harness/tools/run-lines.sh <out dir> <positions.jsonl>
python3 harness/tools/lineshow.py <out dir>/out.jsonl <id fragment>     # one line: diffs, Forge log, engine trace
python3 harness/tools/linewin.py <out dir>/forge.jsonl <id>              # Forge windows and log segments
```
Results: [results/lines/](results/lines/) (per set: `summary.txt`, `menu.txt` with known-divergence suppression counts, `status.tsv`, and `out.jsonl.gz`, `forge.jsonl.gz`, `pos.jsonl.gz`). Line ids (`scn:<file>:<scenario>:l<seed>`, `scn:combos.scn:<family>-<n>:l<seed>`, `rnd:<deckA>-<deckB>-s<seed>-d<decision>`) regenerate any case. The probe leaks memory, so Forge runs are chunked (300 lines, 3 JVMs); a full pass over all three sets takes about 10 minutes on 4 cores.

## Needs a human

1. Approve or reject the `proposed` Forge-side entries: kd-0002, kd-0003, kd-0004 (earlier), kd-0007 to kd-0011 (new), plus the design verdict kd-0010 (E4, together with the engine thread).
2. Send E5 to E7 to the "Rust engine core" thread. Update: E6 and E7 were fixed in RFC 0002 (m4); E5 and E4 remain, proposed as documented limits.
3. Decide whether this evidence meets the Phase 2 gate. My view: yes for the pool's multi-step interactions, with E6 and E7 fixed first and the lockstep rerun on the fixed core (about 10 minutes).
