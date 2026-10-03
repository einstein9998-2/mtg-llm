# Holdout set

Doc 04 section 4.4: about 30% of each file's scenarios are held out so the implementer cannot overfit to what it sees. The project folder is readable by every thread, so the holdout is **sealed (encrypted)** instead of merely hidden.

What is here:

- `MANIFEST.json`: for each visible file, how many scenarios exist, how many are visible and how many are held out, plus a sha256 of every held-out scenario's text. No ids and no content. This is the "pass/fail counts per card" information doc 04 allows the implementer to see.
- `../sealed/holdout-*.tar.enc`: the encrypted holdout scenarios (AES-256-CBC, PBKDF2). The key and the selection seed are not in this folder, not in any message, and not in memory notes.
- `../sealed/full-pool-*.tar.enc`: an encrypted backup of the whole pool (visible plus held out) with the same key, so the spec-writer can recover after a lost session.

Rules for the implementer thread:

1. Do not try to decrypt, guess, or reconstruct held-out scenarios. A scenario runner that reports only pass/fail counts per file for the holdout is run by the spec-writer or by the human with the key, never by the implementer (`tools/unseal_holdout.py` exists for that runner).
2. Everything under `../scenarios/` is the visible set. Fix failures there on their merits: do not special-case scenario ids or setups.
3. When a holdout scenario fails, the spec-writer/triage side learns its id; the implementer is told only the card or rules area. After the card is fixed, the revealed scenario is promoted to the visible set and replaced by a fresh one from the spec-writer (doc 04 4.4).

Re-splitting is a pure function of (seed, scenario id, pool, revealed list, fresh list); see `../tools/split_holdout.py` (`--revealed` forces ids visible without refilling their slot, `--fresh` adds replacements to the holdout without changing any group's split).

Revealed so far: 3 (2026-10-01, card named as an engine bug in the first holdout report). They are now in `../scenarios/` and three fresh sealed scenarios replaced them.
