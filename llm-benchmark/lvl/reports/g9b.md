RESULT: LOST
Turns: game ended on engine turn 7 (opponent's 4th turn; I took only 3 turns of my own)
Prompts: 634 (engine count; about 300 were mine, nearly all "pass" during the opponent's loop)
Odds/opp/sim calls: 1 opp, 0 odds, 0 sim (engine summary says 2 odds-type calls)
Flags: 1

- Plan: Boros aggro on the play-second, curve Parlor, Voice of Victory (stops their spells on my turn), then Ajani. Attacked once (opp 20 -> 14 after Tomb damage). Kept the 7: 3 lands, Voice x2, Ajani, Erode.
- Key play and mistake (turn 6, my 3rd turn): I tapped out for Ajani with the only W source (Elegant Parlor) while holding 2 Swords + Erode. Opponent then cast Aluren (turn 7) and looped Acererak (free via Aluren, Lost Mine of Phandelver ventures; ETB bounces it each time). With no untapped W I could never Swords/Erode Acererak in response to the trigger and was locked out of my own turn. Holding W up would probably have stopped the loop (Swords in response to ETB exiles it). Voice only stops their casts on MY turn, which never came. I also only had 3 lands and 1 W source, so I could not hold up removal and develop.
- Opponent surprise: infinite-style Aluren + Acererak loop; chose Dark Pool every cycle (drain me 1, gain 1), made Goblins/Treasures, drew a card per cycle. About 22 full cycles in one turn took me from 19 to 0; on the last cycle it ventured into Tomb of Annihilation (Trapped Entry: each player loses 1 life) for the kill at 1 life. Because the loop is free and unbounded, the engine allowed ~600 prompts in one turn.
- Engine notes: Acererak keeps returning to hand even after "completed dungeons: Lost Mine" (its check is for Tomb of Annihilation, which only began being entered after many completions, so that looks consistent). Not a bug, but the unbounded loop with no loop-detection or turn cap is notable. Aluren also let me cast my own creatures at instant speed for free, but nothing in my hand helped.
- Tools: 1 opp call (could they hold Acererak/Atraxa/Force: 38% each) did not change a decision; the real info was the opponent's play pattern (passed 2 turns with open mana, then Aluren), which I read too late.
