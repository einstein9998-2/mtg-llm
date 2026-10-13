#!/bin/bash
# mkv.sh <id> <matchup> <first A|B> <v|c>
cd /home/claude/work/llm2
VN="Your deck has one Emrakul, the Aeons Torn, in the main deck in place of ONE Veil of Summer (so the main has only one Veil this game). This is the thing being tested: read Emrakul's card text with get_card and decide for yourself how and when to use it. Brady's note: it does get cast for free when Omniscience is in play. In your report add one line 'VEIL:' saying whether you drew Veil, what it did or would have done, and whether you would rather have had a second Veil than Emrakul against this opponent."
CN="Your deck is the plain main deck (two Veil of Summer, no Emrakul). In your report write 'EMRAKUL: n/a' and add one line 'VEIL:' saying whether you drew Veil, what it did or would have done, and whether you would change anything in the main against this opponent."
if [ $4 = v ]; then BASE_A=$PWD/decks-base-emrv NOTE="$VN" ./mkgame2.sh $1 $2 base $3 1; else BASE_A=$PWD/decks-base NOTE="$CN" ./mkgame2.sh $1 $2 base $3 1; fi
