# Prismatic Ending spots

Branch-and-roll-out tool for "cast Prismatic Ending now or hold it". Report: [REPORT.md](REPORT.md).

- `rust-engine/crates/mtg-agent/src/bin/endingspot.rs`: plays post-board games (MCTS by default, or `--gen net` to sample the net priors, about 100 times cheaper), and at each sorcery-speed turn where Ending can exile an opposing creature, copies the game and plays it out under six arms (cast, late1, late2, wait, never, free). One JSON line per spot. `--baseline N` plays N net-only games from a plan directory (used for the deck-count tests), `--max-life N` keeps only spots at life N or less.
- `analyze.py spots.jsonl [...]`: tables by creature count, life, turn, combo in hand, Show and Tell conflict.
- `parse_llm.py logs...`: Ending spots in the LLM-played full game logs.

```sh
cargo build --release -p mtg-agent --bin endingspot
endingspot <decks> <plans> alurentell ur-cutter --net alurentell-vs-ur-m5chant-net3.bin --games 6000 --gen net --reps 32 --threads 4 --out spots-ur.jsonl
python3 analyze.py spots-ur.jsonl
```

Needs the main-engine (176 definition) m5chant nets, not the later `-live-` nets. Decks and plans are in the shared folder (`decks/`, `sideboard-experiments/plans/`).
