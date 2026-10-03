"""Known-divergence registry matching shared by compare.py and resp_actions.py (doc 04 section 9)."""
import glob, os, collections
try:
    import tomllib
except ImportError:  # python < 3.11
    tomllib = None


def load(kd_dir):
    kds = []
    if kd_dir and tomllib:
        for fn in sorted(glob.glob(os.path.join(kd_dir, 'kd-*.toml'))):
            kds.append(tomllib.load(open(fn, 'rb')))
    return kds


def feature_tags(state_lines):
    bf = ';'.join(l.split('=', 1)[1] for l in state_lines if l.split('=')[0] in ('humanbattlefield', 'aibattlefield'))
    tags = set()
    if 'Aluren' in bf: tags.add('aluren-on-bf')
    if 'Bilbo, Thief in the Night' in bf: tags.add('bilbo-on-bf')
    if 'Lazotep Quarry' in bf: tags.add('quarry-on-bf')
    if 'Deafening Silence' in bf: tags.add('deafening-silence-on-bf')
    if "Grafdigger's Cage" in bf: tags.add('grafdigger-on-bf')
    if 'Cavern of Souls' in bf: tags.add('cavern-on-bf')
    if 'Voice of Victory' in bf: tags.add('voice-of-victory-on-bf')
    lifes = [int(l.split('=')[1]) for l in state_lines if l.split('=')[0] in ('humanlife', 'ailife')]
    if 'Ancient Tomb' in bf and lifes and min(lifes) <= 5: tags.add('tomb-low-life')
    return tags


def match(kds, side, key, tags, actor):
    """The id of the first known divergence that covers an action-set difference (side RUST_ONLY/FORGE_ONLY, canonical key), or None."""
    parts = key.split('|')
    kind, zone, card = (parts + ['', '', ''])[:3]
    for kd in kds:
        m = kd.get('match', {})
        if m.get('sides') and side not in m['sides']: continue
        if m.get('kinds') and kind not in m['kinds']: continue
        if m.get('zones') and zone not in m['zones']: continue
        if m.get('cards') and card not in m['cards']: continue
        if m.get('feature_tags') and not set(m['feature_tags']) <= tags: continue
        if m.get('actor_seat') is not None and m['actor_seat'] != actor: continue
        return kd['id']
    return None
