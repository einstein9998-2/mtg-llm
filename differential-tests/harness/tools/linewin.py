#!/usr/bin/env python3
"""linewin.py <forge.jsonl> <id substring> [w0 [w1]]: the Forge windows (seat, stack, chosen, summary) and log segments of one line."""
import json, sys
for l in open(sys.argv[1]):
    r = json.loads(l)
    if sys.argv[2] in r['id']:
        w = r['windows']; log = r['log']
        a = int(sys.argv[3]) if len(sys.argv) > 3 else 0
        b = int(sys.argv[4]) if len(sys.argv) > 4 else len(w) - 1
        print('==', r['id'], 'over', r.get('over'), r.get('winner'))
        for i in range(a, min(b, len(w) - 1) + 1):
            x = w[i]
            end = w[i + 1]['log_at'] if i + 1 < len(w) else len(log)
            print(f"-- w{i} {x['seat']} stack={x['stack']} chosen={x['chosen']} {x.get('played','')}")
            print('   keys:', x['keys'])
            for s in x['summary']: print('   ', s[:260])
            for e in log[x['log_at']:end]: print('     log:', e[:230])
