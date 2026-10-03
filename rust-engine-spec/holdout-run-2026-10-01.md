# Sealed holdout run against the M3 freeze candidate (2026-10-01)

Run by the spec thread with the key, on a copy of /mnt/project-files/rust-engine/ (release `specrun`, no engine edits).

**Result: 321 of 338 sealed scenarios pass (14 fail, 3 unsupported by the runner).** Visible set still 803 of 803.
By area (held out / not passing): card 193/7, core 90/6, hidden-info 13/1, matchup 42/3.

Per doc 04 section 4.4 only the card or rules area is given for each problem, not scenario ids or contents. The 17 non-passes split as follows after triage (each checked by reading the engine code and, where useful, a small probe).

## Engine problems (3 scenarios): please fix, then I promote the revealed scenarios to visible and add fresh ones
1. **Quantum Riddler (warp):** a card exiled with warp is not offered as castable from exile on a later turn for its normal mana cost (the card definition has the draw, end-step exile and the delayed exile, but no cast-from-exile permission).
2. **Voice of Victory:** the Mobilize 2 attack trigger is missing from the card definition (only the opponent-casting restriction exists).
3. **Payment choice:** a scenario that names a specific land to tap for a spell while mana is floating gets the floating mana spent instead. The player should be able to pick the source (601.2h); if auto-spend of the pool is intended, say so and I change the scenario.

## Runner/adapter gaps in `mtg-spec` (not engine bugs; 9 scenarios, the Daze item below is counted under spec bugs)
- Setup entries for **tokens** ignore `counters`, `damage` and `attached_to` (runner.rs, the token loop after `parse_entry`); the runner reads them but only passes `sick` and `tapped` to `scenario_token`. Affects two combat scenarios.
- **Over-delve** (correction to my first draft, which called it an engine bug): the engine is right, it stops the delve prompt once the generic cost is covered. The runner (runner.rs, delve loop near line 1059) silently drops delve cards that were named but never prompted, so an over-delve cast reports as legal. The runner should fail on unused delve cards. Affects 1 scenario.
- Scripted random event `bottom_order` is not supported (1 scenario).
- Counter kind `flying` is not supported in `counter_kind` (2 scenarios).
- Legal-action pattern `{t: activate, source: <land>}` is not matched by mana-ability options (`Opt::Mana`), only by `Opt::Activate` (1 scenario).
- `legal` patterns with `alt_cost: {return_island: X}` check only that the alternative way is offered, not which Island; I am rewriting those scenarios to use expect_illegal (the engine itself handles them correctly: a probe shows Mountain is rejected and Volcanic Island accepted; counted under spec bugs below).
- One scenario expects three sacrifice choices (creature, artifact, land) as one unordered selection; the engine asks them one at a time. SCHEMA.md leaves the shape to the runner adapter, so the adapter should merge them or accept the sequence; I will document the sequential form as acceptable.
- Doomsday with fewer than five cards available raises a one-candidate pile selection; the script expects none (decisions the rules do not give the player are not raised). Please skip the prompt when the choice is forced, or tell me and I change the script.

## Spec bugs found by this run (5 scenarios; fixed on my side, the sealed set is rebuilt at the next publish)
The two Daze scenarios above, plus three more (a replicate copy on the stack lacked `copy: true`, a noncreature artifact was asserted summoning sick, and the Kaito equipment scenario asserted the on-turn type line, which is an open rules question: CR 205.1a replaces types when an effect sets them and 205.1b keeps them only for "in addition" wording; the engine currently makes Kaito a creature only on his controller's turn; see OPEN-QUESTIONS.md).

## Not covered here
Freeze review of the rules code is reported separately.

## Rerun on the M3d build (spec thread, 2026-10-01 later)
Build: copy of /mnt/project-files/rust-engine/ after the fixes (`engine2`, release `specrun`, no engine edits).
- Sealed holdout (338, rebuilt with my five spec fixes and the sequential Oubliette form, same seed): **338 of 338 pass**.
- Visible set: **806 of 806 pass** (803 plus the three promoted scenarios below).
- The three scenarios behind the engine problems above (Quantum Riddler warp recast, Voice of Victory Mobilize, tap-a-named-land payment) are promoted to visible, and three fresh sealed ones replace them; I checked that each fresh one fails on the pre-fix build and passes on the fixed one.
- Corrections that went in with this: Voice of Victory's token is named `Warrior Token` (oracle.json and scenarios had a longer spelling), and SCHEMA.md has addenda for the one-instruction-several-choices shape, `expect_illegal` for object-specific alternative-cost choices, and token setup keys.
- Published set: 1144 scenarios, 806 visible, 338 sealed (`sealed/*-2026-10-01d.tar.enc`; the older `c` archives were deleted).

## M3f build and four new sealed scenarios (spec thread, 2026-10-01 evening)
Brady chose simultaneous entry for Stronghold Gambit. Four fresh sealed scenarios were added (three Gambit cases: the tie in both reveal orders and a lowest-mana-value case, plus Animate Dead with Grafdigger's Cage); the first three fail on the M3d build and pass on the M3e/M3f build. Rerun on the M3f build: **sealed 342 of 342, visible 806 of 806**. Published set: 1148 scenarios (806 visible, 342 sealed, archives `*-2026-10-01e.tar.enc`).

Update: one more fresh sealed scenario (miracle) added after the M3f review; published set is 1149 (806 visible, 343 sealed, archives `*-2026-10-01f.tar.enc`). The sealed miracle scenario fails on M3f and passes on M3d; rerun the full sealed set on the fixed build when it exists.

M3h (RFC 0001, Lazotep Quarry sacrifice mana): sealed 343 of 343 (includes the miracle scenario, so the M3g miracle fix is confirmed) and visible 806 of 806 on the M3h build. Two fresh sealed Quarry scenarios added (castable with a creature, not castable without); the first fails on M3f and passes on M3h. Published set: 1151 (806 visible, 345 sealed, archives `*-2026-10-01g.tar.enc`).

M4 (RFC 0002, LKI subtype filters and multi-Quarry payment): five fresh sealed scenarios added (Ajani Pariah with a Cat, a non-Cat and an opponent's Cat dying; two Quarries with and without a creature); two of them fail on M3h and pass on M4. Rerun on M4: sealed 350 of 350, visible 806 of 806. Published set: 1156 (archives `*-2026-10-01h.tar.enc`). Ajani's front face and the Cat Warrior token were added to oracle.json from memory (unverified, flagged); the scenarios using them are verified:false.

## 2026-10-02: m5 (RFC 0003) rerun

Visible 806/806 and sealed 352/352 on core-frozen-m5. Sealed archive republished as `sealed/holdout-2026-10-02a.tar.enc` (with `full-pool-2026-10-02a.tar.enc`) after adding two scenarios (1158 in total: Quarry auto-payment keeps the spell's target, and a 300-attacker combat with one block). No visible scenario changed. See freeze-review-rules-2026-10-01.md for the RFC 0003 review.
