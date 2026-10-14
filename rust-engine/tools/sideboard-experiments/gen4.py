#!/usr/bin/env python3
"""Round 3 (2026-10-13): Emrakul for one land, or as a 61st card, in game 1 (unboarded). Writes decks-<m>-emrg1-<cfg> and q14.txt."""
from gen3 import write_main_deck, EMR
CFG = {'nlisl': {'Island': -1}, 'nlhm': {'Hedge Maze': -1}, 'nlstr': {'Flooded Strand': -1},
       'nlcot': {'City of Traitors': -1}, 'nlbos': {'Boseiju, Who Endures': -1}, 'nlsav': {'Savannah': -1}, 'e61': {}}
MS = ('ur', 'uw', 'ub', 'tron', 'storm')
if __name__ == '__main__':
    with open('q14.txt', 'w') as f:
        for m in MS:
            for nm, ed in CFG.items():
                write_main_deck(f'{m}-emrg1-{nm}', {**ed, EMR: +1})
                for s in ('play', 'draw'): f.write(f'nn {m} emrg1-{nm} {s} 400 1\n')
    print(sum(1 for _ in open('q14.txt')), 'cells')
