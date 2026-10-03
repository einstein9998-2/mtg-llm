# MTG LLM

An AI that plays Legacy Magic: The Gathering (1v1) at a high level and improves through self-play, at low cost per game.

- **Closed pool.** Eight fixed 75-card Legacy decks (UR Cutter, Alurentell, Boros Aggro, BW Death and Taxes, Dimir Tempo, UWx Control, Doomsday, Reanimator). Generalizing to unseen cards is not a goal.
- **Cost first.** No LLM in high-volume inner loops by default. The project compares an LLM player with tools against a neural net plus search on win rate per dollar.
- **The engine owns all facts.** Players only choose among engine-enumerated legal actions. Hidden information (the opponent's hand, library order) is never exposed to a player, including through search or simulate tools.

## Phases

1. **Forge headless** (`serializer/`, `forge-runner/`, `forge-cloning/`, `llm-tools/`, `llm-player/`): drive Forge from outside, serialize game state, enumerate legal actions, and let an LLM play through a prompt loop.
2. **Custom Rust engine** (`engine-design/`, `rust-engine/`, `rust-engine-spec/`, `differential-tests/`, `interaction-tests/`, `rust-engine-review/`): design docs first, cards as a DSL, a rules spec written by a separate agent from the implementer, and differential testing against Forge. The core is **frozen** (tag `core-frozen-m5` in the original working history); changes go through an RFC in `rust-engine/rfcs/`.
3. **Policy/value net plus determinized search** (`rust-engine/crates/mtg-agent`): expert iteration with determinized MCTS, trained in PyTorch, run on the CPU inside the search. Results are in `self-play-results/`.

## Layout

| Path | What is in it |
|---|---|
| `decks/` | The eight decklists, the deck-set write-up and the Forge card coverage table. `superseded-*` hold earlier deck sets. |
| `engine-design/` | Design docs: core, action API and hidden info, card DSL, differential testing spec, pool mechanics inventory. Start with its `README.md`. |
| `rust-engine/` | The Rust workspace (`mtg-core`, `mtg-dsl`, `mtg-cards`, `mtg-view`, `mtg-agent`, `mtg-fuzz`, `mtg-spec`, `mtg-debug`), card definitions in RON, goldens, RFCs, `MILESTONES.md`, `CORE-FREEZE-REVIEW.md` and `PHASE3-HANDOFF.md` (the API for learners). |
| `rust-engine-spec/` | Rules spec: visible YAML scenarios, schema, coverage and open questions, and the **sealed** holdout (`sealed/*.tar.enc`, AES-256 encrypted; the key is not in this repo and must never be added). |
| `rust-engine-review/` | Independent hidden-information and determinism reviews, probes and the M3g-to-M5 diff. |
| `differential-tests/` | Rust and Forge-probe harness comparing the engine with Forge, reports (`REPORT.md`, `LOCKSTEP-REPORT.md`), `known-divergences/` and `repros/`. |
| `interaction-tests/` | Forge rules-check harness, findings and scenarios. |
| `serializer/`, `forge-runner/`, `forge-cloning/` | Phase 1 Forge tooling: state serializer and legal-action enumerator, batch runner, and the report on why Forge forking is too slow and wrong for search. |
| `llm-tools/` | Command-line tools an LLM player calls (card lookup, library odds, opponent-hand sampling, matchup notes). |
| `llm-player/` | The LLM player loop, playbooks (for example `alurentell-playbook.md`), batch reports and engine oddities. |
| `self-play-results/` | Phase 3 training and arena results. |
| `docs/` | Loose notes, such as the Ajani runaway engine bug. |

## Quick start

```sh
cd rust-engine
cargo test            # toolchain pinned in rust-toolchain.toml
```

Spec scenarios run through the `mtg-spec` crate; see `rust-engine-spec/README.md`. Phase 1 tools need a local Forge checkout and Java; see `serializer/README.md`.

## Not in this repo

Result dumps (`*.jsonl.gz`, `*.tsv`, per-game logs under `llm-player/runs/`), trained nets and checkpoints, build output, and the training scripts and run data that live only on the author's PC. Game records and self-play dumps made on older engine versions must be regenerated, since state hashes changed with each core version.
