# RFC 0010: A search into exile starts with no stale "moved" objects

Status: applied to the live engine 2026-10-08 (Brady said yes to fixing the Storm `Hand -> Hand` fuzz finding); independent scenarios by a separate writer
Author: "Alternate UR Cutter list" thread

## Problem
`legacyfuzz` reported `I3 illegal zone transition Hand -> Hand` in Storm games (storm v alurentell seed 103; alurentell v storm-tes-alt seed 6392), on the unmodified engine too. Cause: Beseech the Mirror is modelled as `Search(dest: Exile)`, then `CastMovedFree`, then `MoveTo(Moved(0), Hand)` ("if you don't cast it, put it into your hand"). The search only fills the "moved" list when it finds a card. When the library holds no match (or the player stops), the list still holds whatever an earlier effect left there (here the bargain sacrifice, a bounced Acererak, a token), and `MoveTo(Moved(0), Hand)` moved that object: a hand card to the hand (the I3 hit), a permanent or token bounced to hand.

## Change
`fx_search` takes the destination and clears the moved list when it is `Exile`, before it asks for the card. `Effect::Search` and `SearchNoShuffle` pass `dest`. Two files: `libfx.rs`, `resolve.rs` (`rfcs/0010-search-clears-moved.patch`).

Searches into other zones are untouched on purpose: `moved` is part of the state hash, so clearing it for every search changes the checkpoints of real-pool game records that contain a Personal Tutor (r12 does), and no card reads `Moved` after those searches.

## Tests
- Three scenarios by an independent writer (`rust-engine-ur/scenarios/ur-C-*`): a failed bargained search after a bounce leaves the earlier card where it was; a failed search with an earlier Eldrazi Scion token does not bounce the token (fails on the old engine); a successful search still puts the found card into hand.
- Spec 1232/1232 (all earlier scenarios unchanged), workspace tests 83/0 with all test-pool and real-pool goldens replaying bit-identically, fuzz of 3000 games over the Legacy pool with both Storm lists.

## Impact
No new state field; the hash changes only inside games where a search into exile ran with a non-empty moved list (Beseech games). Card database hash unchanged.
