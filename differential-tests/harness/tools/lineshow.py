#!/usr/bin/env python3
"""lineshow.py <out.jsonl> <id substring> [--pos pos.jsonl]: print one line's result."""
import json, sys
rows = [json.loads(l) for l in open(sys.argv[1])]
for r in rows:
    if sys.argv[2] in r['id']:
        print('==', r['id'], r['status'], 'windows', r.get('windows'))
        print('played:', r.get('played'))
        for d in r.get('diffs', []): print('DIFF', d['window'], d['kind'], d['detail'][:6])
        print('forge log at divergence:'); [print('   ', x[:200]) for x in r.get('forge_log_at_divergence', [])]
        print('forge summary at divergence:'); [print('   ', x[:300]) for x in (r.get('forge_summary_at_divergence') or [])]
        print('rust trace:'); [print('   ', x[:200]) for x in r.get('rust_trace', [])[-14:]]
        print('forge notes:', [n[:150] for n in r.get('forge_notes', [])][:8])
