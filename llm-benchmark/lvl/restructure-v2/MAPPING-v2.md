# Mapping: version 1 to version 2

Version 1 is /home/claude/work/llm2/restructure/ (playbook.md with P1 to P14, 11 skill files). Version 2 is this folder. MAPPING.md (copied from version 1) still maps the old 150-line playbook to the rule ids; every version 1 id keeps its number and its file in version 2. CHANGES.md lists every change with the Brady line behind it.

## Principle changes (marked)

| Principle | Change | Why |
|---|---|---|
| P1 | **Reworded.** "Always have a route to a payoff" became "Have a route to a payoff"; the Why now says Brady counts all four combo cards as payoffs and you need the right two. | Brady, conflict 2: "Don't think anything is "always"; I'm counting all 4 combo cards as payoffs and you need to assemble the right 2." |
| P13 | Why: one sentence added ("Sideboarding applies the same idea between games"). Principle text unchanged. | Brady: "Start playing matches: best of 3 with sideboarding". |
| P15 | **New.** "Nothing is "always": weigh how often their answer is there and what being wrong costs." | The concept several of Brady's 2026-10-08 rulings share: conflict 2 ("always"), conflict 5 ("probably 95%+"), his P4 comment (Tomb: 3 to 1 hurts more than 3 to 2), conflict 13 (Show and Tell's upsides). Tagged on OH9, ML15, TS9, PR9, ST17. |

No principles were merged. P2 to P12 and P14 are unchanged.

## New file

- skills/sideboarding.md: SB1 to SB4, the base 75, what each sideboard card is for, and the seven Alurentell plans. Indexed in playbook.md.
- PENDING.md (top level): candidate rules Brady has not ruled on.

## New rule ids

| Id | File | From |
|---|---|---|
| OH9 | opening-hand-and-mulligan.md | SYNTHESIS2 rule 1, Brady P1 comment and conflict 2 |
| ML12 | mana-and-life.md | conflict 4 (part of rule 25) |
| ML13 | mana-and-life.md | rule 18, conflict 10 |
| ML14 | mana-and-life.md | rule 19, Brady P6 comment |
| ML15 | mana-and-life.md | conflict 15 |
| TS9 | turn-sequencing.md | rule 12, Brady P4 comment |
| TS10 | turn-sequencing.md | conflict 3 (Wasteland part of rule 15) |
| TS11 | turn-sequencing.md | rule 14, conflict 9 |
| PR15 | protecting-the-combo.md | rule 26, conflict 5 |
| PR16 | protecting-the-combo.md | rule 6, conflict 11 |
| ST16 | show-and-tell.md | rule 28, Brady P9.28 comment |
| ST17 | show-and-tell.md | rule 29, Brady P9.29 comment, conflict 13 |
| AL7 | aluren-and-the-loop.md | conflict 8 |
| AL8 | aluren-and-the-loop.md | conflict 6 |
| AL9 | aluren-and-the-loop.md | rule 30, Brady P10 comment |
| BO4 | vs-boros.md | conflict 1 |
| EF1 to EF11 | process-and-engine.md | rules 20, 34, 35, 36 and SYNTHESIS2 section 4 (not from Brady) |
| SB1 to SB4 | sideboarding.md | Brady's new instructions; engine match rules; sideboard plan file |

Judgment notes (not rules): conflict 12 in mana-and-life.md; conflict 14 in protecting-the-combo.md.

## SYNTHESIS2 section 1, where each candidate rule went

| Rule | Where |
|---|---|
| 1 | OH9 (with OH1's new Breaks-when) |
| 2, 3, 4, 5, 7, 8, 9, 10, 11, 13 | PENDING.md |
| 6 | PR16 |
| 12 | TS9 |
| 14 | TS11 |
| 15 | Wasteland part: TS10; rest: PENDING.md |
| 16 | Engine fact: EF1; strategy part: PENDING.md |
| 17 | PENDING.md |
| 18 | ML13 |
| 19 | ML14 (and BO3) |
| 20 | Engine fact: EF3; Stock Up pick part: PENDING.md |
| 21, 22, 24 | PENDING.md |
| 23 | Judgment note in mana-and-life.md (Brady "Depends", conflict 12) |
| 25 | Duals: ML12; rest: PENDING.md |
| 26 | PR15 |
| 27 | PENDING.md |
| 28 | ST16 |
| 29 | ST17 |
| 30 | Brady's use: AL9; defence part: PENDING.md |
| 31, 32, 33 | PENDING.md |
| 34 | EF4 |
| 35 | EF5 |
| 36 | EF6 |

## SYNTHESIS2 section 2, where each conflict went

| Conflict | Brady | Where |
|---|---|---|
| 1 Force on Voice | agrees mostly with "only with a spare blue" | BO4; PR10 Breaks-when |
| 2 No-enabler sevens | nothing is "always"; four combo cards, right two | P1, P15, OH9; question (a) in PENDING.md |
| 3 Wait or jam under Wasteland | "Agree with jam more often than not" | TS10; Breaks-when of TS6, TS7, TS8, ML7 |
| 4 Boseiju on a dual | "Don't Boseiju a dual" | ML12 |
| 5 Veil and the Daze land | "Yes", probably 95%+ | PR9, PR15; PR7 Breaks-when; P15 |
| 6 Omniscience-only loop | good; never really deck | AL8; AL2, AL6, ST10 notes |
| 7 Aluren or Omniscience put-in | "Usually prefer Omniscience"; Static Prison | ST7, ST8; vs-boros.md note |
| 8 Dig first | "Yes, dig first" | AL7; AL1 Breaks-when |
| 9 Sorcery cantrips | "Okay" | TS11 |
| 10 Petal as spare or combo mana | "Okay" | ML13 |
| 11 One blue pitch per Force | "Okay" | PR16 (UR note: CHANGES.md, not placed) |
| 12 Hedge Maze vs basics under Wasteland | "Depends" | Judgment note, mana-and-life.md; CS13 Breaks-when |
| 13 Aluren vs Show and Tell against Boros | "often better", with Show and Tell's upsides | ST17; ST1 Breaks-when; P15 |
| 14 Force Swords on Atraxa | "Depends but probably doesn't matter that much" | Judgment note, protecting-the-combo.md |
| 15 Tomb vs a fast clock | "the 2 damage is impactful" | ML15 |
