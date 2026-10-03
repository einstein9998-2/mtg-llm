# Sample prompts (hybrid policy, game 0, Death and Taxes = you vs Jund)

Each block is exactly the text a player receives for one decision (headers aside). `->` shows what the policy picked; it is not part of the prompt. Token counts are estimates (chars / 3.6). Priority prompts show the full state at the first prompt of a turn and only changed sections afterwards.

## D1 YesNo (n=2, 84 chars, ~23 tokens) -> Draw first
```
D1 YesNo: you won the die roll: play first or draw first?
0 Play first
1 Draw first
```

## D2 Mulligan (n=2, 194 chars, ~54 tokens) -> Mulligan
```
D2 Mulligan: opening hand (7): Batterskull#7, Plains#4, Plains#5, Stoneforge Mystic#6, Swords to Plowshares#3, Thalia, Guardian of Thraben#1, Wasteland#2 | you are on the draw
0 Keep
1 Mulligan
```

## D3 BottomCards (n=7, 201 chars, ~56 tokens) -> Mother of Runes#9
```
D3 BottomCards: put 1 card(s) on the bottom of your library
0 Mirran Crusader#13
1 Mother of Runes#9
2 Mother of Runes#14
3 Phyrexian Revoker#12
4 Plains#10
5 Plains#11
6 Thalia, Guardian of Thraben#8
```

## D5 Priority (n=3, 864 chars, ~240 tokens) -> Play land Plains#10 (x2)
```
T2 your turn, MAIN1 | life you 20 opp 20
YOUR HAND (7): Mirran Crusader#13, Mother of Runes#14, Phyrexian Revoker#12, Plains#10, Plains#11, Thalia, Guardian of Thraben#8, Wasteland#16
YOUR BF: -
OPP BF: lands: Badlands#15
YOUR GY: -
OPP GY: -
OPP HAND 6, OPP LIB 53 | YOUR LIB 53
STACK (top first): empty
LOG: (20 earlier events omitted) | your Batterskull: hand -> library | you draw Thalia, Guardian of Thraben#8 | you draw Mother of Runes#9 | you draw Plains#10 | you draw Plains#11 | you draw Phyrexian Revoker#12 | you draw Mirran Crusader#13 | you draw Mother of Runes#14 | your Mother of Runes: hand -> library | you mulligan | -- turn 1 (opp) -- | opp's Badlands#15 enters the battlefield from hand | -- turn 2 (you) -- | you draw Wasteland#16
D5 Priority: you have priority in MAIN1, stack empty
0 Pass
1 Play land Plains#10 (x2)
2 Play land Wasteland#16
```

## D6 Priority (n=2, 342 chars, ~95 tokens) -> Pass
```
T2 your turn, MAIN1 | life you 20 opp 20
YOUR HAND (6): Mirran Crusader#13, Mother of Runes#14, Phyrexian Revoker#12, Plains#11, Thalia, Guardian of Thraben#8, Wasteland#16
YOUR BF: lands: Plains#10
LOG: your Plains#10 enters the battlefield from hand
D6 Priority: you have priority in MAIN1, stack empty
0 Pass
1 Cast Mother of Runes#14 {W}
```

## D7 Priority (n=2, 131 chars, ~36 tokens) -> Cast Mother of Runes#14 {W}
```
T2 your turn, MAIN2 | life you 20 opp 20
D7 Priority: you have priority in MAIN2, stack empty
0 Pass
1 Cast Mother of Runes#14 {W}
```

## D8 Priority (n=5, 969 chars, ~269 tokens) -> Play land Wasteland#16
```
T4 your turn, MAIN1 | life you 17 opp 20
YOUR HAND (6): Mirran Crusader#13, Phyrexian Revoker#12, Plains#11, Swords to Plowshares#3, Thalia, Guardian of Thraben#8, Wasteland#16
YOUR BF: Mother of Runes#14 1/1; lands: Plains#10
OPP BF: lands: Badlands#15, Forest#18
YOUR GY: -
OPP GY: Lightning Bolt#17
OPP HAND 5, OPP LIB 52 | YOUR LIB 52
STACK (top first): empty
LOG: cast Mother of Runes#14 (you) | your Mother of Runes#14 enters the battlefield | cast Lightning Bolt#17 (opp) -> you | you life 20 -> 17 | opp's Lightning Bolt#17: stack -> graveyard | -- turn 3 (opp) -- | opp draws a card | opp's Forest#18 enters the battlefield from hand | -- turn 4 (you) -- | you draw Swords to Plowshares#3
D8 Priority: you have priority in MAIN1, stack empty
0 Pass
1 Play land Plains#11
2 Play land Wasteland#16
3 Cast Swords to Plowshares#3 {W}
4 Activate Mother of Runes#14: {T}: Target creature you control gains protection from the color of your choice until end of turn.
```

## D10 Attack (n=2, 143 chars, ~40 tokens) -> Attack opp
```
T4 your turn, COMBAT_DECLARE_ATTACKERS | life you 17 opp 20
D10 Attack: declare attack for Mother of Runes#14 1/1
0 Do not attack
1 Attack opp
```

## D24 Block (n=2, 164 chars, ~46 tokens) -> Do not block
```
T7 opp turn, COMBAT_DECLARE_BLOCKERS | life you 17 opp 15
D24 Block: declare block for Mother of Runes#14 1/1
0 Do not block
1 Block Bloodbraid Elf#23 3/2 (T, att)
```

## D29 Target (n=2, 152 chars, ~42 tokens) -> Thalia, Guardian of Thraben#8 (your bf)
```
D29 Target: targets for Mother of Runes#14: Select target creature you control
0 Mother of Runes#14 (your bf)
1 Thalia, Guardian of Thraben#8 (your bf)
```

## D30 ChooseOption (n=5, 458 chars, ~127 tokens) -> blue
```
T8 your turn, MAIN1 | life you 14 opp 15
YOUR BF: Aether Vial#21 (1xCHARGE), Mother of Runes#14 1/1 (T), Thalia, Guardian of Thraben#8 2/1; lands: Plains#10, Plains#11, Wasteland#16
STACK (top first): ability of Mother of Runes#14 (you) -> Thalia, Guardian of Thraben#8
LOG: put on stack: ability of Mother of Runes#14 (you) -> Thalia, Guardian of Thraben#8
D30 ChooseOption: Mother of Runes#14: choose a protection type
0 black
1 blue
2 green
3 red
4 white
```

## D46 ChooseCards (n=2, 202 chars, ~56 tokens) -> Phyrexian Revoker#12 (your hand)
```
T10 your turn, MAIN1 | life you 9 opp 16
D46 ChooseCards: Aether Vial#21: Select a card from your hand (may choose none) (pick 0-1)
0 Phyrexian Revoker#12 (your hand)
1 Stoneforge Mystic#29 (your hand)
```

## D47 ChooseName (n=48, 1047 chars, ~291 tokens) -> Wear // Tear
```
T10 your turn, MAIN1 | life you 9 opp 16
D47 ChooseName: Phyrexian Revoker#12: name a card (from the card pool)
0 Abrupt Decay
1 Aether Vial
2 Ancient Grudge
3 Armageddon
4 Batterskull
5 Bloodbraid Elf
6 Brimaz, King of Oreskos
7 Collective Brutality
8 Containment Priest
9 Council's Judgment
10 Dark Confidant
11 Disenchant
12 Elspeth, Sun's Champion
13 Ethersworn Canonist
14 Fatal Push
15 Flickerwisp
16 Grim Lavamancer
17 Hymn to Tourach
18 Inquisition of Kozilek
19 Kataki, War's Wage
20 Kolaghan's Command
21 Lightning Bolt
22 Liliana of the Veil
23 Liliana, the Last Hope
24 Maelstrom Pulse
25 Mana Tithe
26 Mirran Crusader
27 Mother of Runes
28 Oblivion Ring
29 Obstinate Baloth
30 Orcish Bowmasters
31 Pernicious Deed
32 Phyrexian Revoker
33 Pithing Needle
34 Pyrokinesis
35 Recruiter of the Guard
36 Rest in Peace
37 Scavenging Ooze
38 Stoneforge Mystic
39 Surgical Extraction
40 Sword of Fire and Ice
41 Swords to Plowshares
42 Tarmogoyf
43 Thalia, Guardian of Thraben
44 Thoughtseize
45 Toxic Deluge
46 Umezawa's Jitte
47 Wear // Tear
```

## D64 OrderTriggers (n=2, 911 chars, ~253 tokens) -> trigger of Aether Vial#21: At the beginning of your upkeep, you may put a charge counter on ~.
```
T12 your turn, UPKEEP | life you 7 opp 16
YOUR HAND (0): 
YOUR BF: Aether Vial#21 (2xCHARGE), Aether Vial#25, Mother of Runes#14 1/1, Stoneforge Mystic#29 1/2; lands: Plains#10, Plains#11
OPP BF: Scavenging Ooze#24 2/2 (T); lands: Badlands#15, Badlands#30, Forest#18, Forest#22, Mountain#26
YOUR GY: Mirran Crusader#13, Swords to Plowshares#3, Thalia, Guardian of Thraben#8, Wasteland#16, Phyrexian Revoker#12
OPP GY: Lightning Bolt#17, Thoughtseize#20, Fatal Push#28, Wasteland#19, Bloodbraid Elf#27
EXILE: yours - | opp Bloodbraid Elf#23
OPP HAND 2, OPP LIB 46 | YOUR LIB 49
STACK (top first): empty
LOG: -- turn 12 (you) --
D64 OrderTriggers: pick the trigger to put on the stack next (first picked resolves last)
0 trigger of Aether Vial#21: At the beginning of your upkeep, you may put a charge counter on ~.
1 trigger of Aether Vial#25: At the beginning of your upkeep, you may put a charge counter on ~.
```
