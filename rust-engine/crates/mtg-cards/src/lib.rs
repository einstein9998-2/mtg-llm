//! Card database construction. M1 ships a small hand-built test pool (vanilla creatures, burn,
//! counterspell, removal, a pinger, an anthem, token makers) used by the M0 spike, the fuzzer and
//! the engine's own unit tests. The RON/DSL pipeline of doc 03 arrives in M2.

pub mod legacy;
pub mod testpool;
