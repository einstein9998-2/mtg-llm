//! Deliberate leaks for mutation-testing the non-interference test. Compiled to constant `false`
//! unless the `canary` feature is on.

pub const VIEWID_FROM_CARDID: u32 = 1;
pub const OPTION_ORDER_BY_CARDID: u32 = 2;
pub const HASH_OVER_STATE: u32 = 4;
pub const ERROR_NAMES_CARD: u32 = 8;

#[cfg(feature = "canary")]
mod imp {
    use std::sync::atomic::{AtomicU32, Ordering};
    static MASK: AtomicU32 = AtomicU32::new(0);
    pub fn set(m: u32) {
        MASK.store(m, Ordering::SeqCst);
    }
    #[inline]
    pub fn on(bit: u32) -> bool {
        MASK.load(Ordering::Relaxed) & bit != 0
    }
}

#[cfg(not(feature = "canary"))]
mod imp {
    #[inline(always)]
    pub fn on(_bit: u32) -> bool {
        false
    }
}

pub use imp::on;
#[cfg(feature = "canary")]
pub use imp::set;
