#!/usr/bin/env python3
"""Pass priority until my turn or until the opponent puts something dangerous on the stack.
Dangerous = anything but cantrips/fetches/triggers (Charm, Heat, Bolt, creatures, Wasteland, Cutter...). Usage: pass_safe.py [max]"""
import json, subprocess, sys, re
STATE="/tmp/claude-0/-home-claude/c7ffc76b-a18f-57e7-ae47-4ad97926fc8f/scratchpad/broker/cur.json"
SAFE=("Dragon's Rage Channeler","Cori-Steel Cutter","Ponder","Brainstorm","Preordain","Mishra's Bauble","trigger of","ability of Misty","ability of Polluted","ability of Flooded","ability of Scalding","ability of Wooded","ability of Prismatic","ability of Windswept","ability of Arid","ability of Bloodstained","Thundering Falls")
def cur():
    return json.load(open(STATE))
n=int(sys.argv[1]) if len(sys.argv)>1 else 30
for i in range(n):
    m=cur(); p=m["prompt"]
    first=p.split("\n")[0]
    if m["kind"]!="Priority": print(p); break
    if "your turn" in first and "MAIN1" in first and "STACK (top first): empty" not in p and False: pass
    mine = re.search(r"T\d+ your turn", first) is not None
    if mine and "MAIN1" in first: print(p); break
    st=re.search(r"STACK \(top first\): (.*)", p)
    if st and st.group(1).strip()!="empty":
        items=[x.strip() for x in st.group(1).split(";")]
        if not all(any(s in it for s in SAFE) for it in items):
            print("STOP: ",st.group(1)); print(p); break
    out=subprocess.run(["python3","play.py","pick","0","pass"],capture_output=True,text=True).stdout
    if "GAME OVER" in out: print(out[-300:]); break
else:
    print("max reached")
