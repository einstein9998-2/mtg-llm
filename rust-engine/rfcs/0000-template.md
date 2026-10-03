# RFC NNNN: <title>

Status: draft | in review | accepted | rejected
Author: <who>  Reviewers (two independent, one a rules reviewer): <who>, <who>

## Problem
What fails or is missing, with the scenario or card that shows it.

## Proposed change
The change in `mtg-core` (or `mtg-view` API types), concretely.

## Rules basis
CR citations (CR effective 2025-11-14) and rulings, each marked verified or not.

## Impact
- State layout and `hash_rules` / `hash_full`:
- Determinism and golden replays (list every changed golden and why it is expected):
- Hidden information (who may see any new state; non-interference and fork tests):
- Effect VM / IR (new variants are core-adjacent, doc 01 section 14.1):
- `ENGINE_CORE_VERSION` bump: yes / no

## Test plan
Rulings scenarios (authored by the spec thread), differential runs, fuzz, non-interference, benchmark.
