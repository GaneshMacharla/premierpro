//! Lite mode: smaller caches and fewer decoder threads for machines with little RAM (8 GB and
//! no dedicated GPU).
//!
//! The process-wide switch is read when the decoder pool and each decoder are made, so the app
//! sets it once at start-up, before any media is opened ([`set_lite`]).
//!
//! | | normal | lite |
//! |---|---|---|
//! | decoded frames all caches keep ([`frame_budget`]) | 1 GiB | 256 MiB |
//! | decoders kept once idle / while recently used | 4 / 8 | 2 / 3 |
//! | worker threads per decoder ([`decoder_threads`]) | cores (≤ 16) | ≤ 4 |
//!
//! A frame-threaded decoder holds `threads + 2` pictures in flight, so the thread cap is what
//! keeps a 4K H.264 or HEVC clip from taking half a gigabyte on its own.

use std::sync::atomic::{AtomicBool, Ordering};

static LITE: AtomicBool = AtomicBool::new(false);

/// Most worker threads a decoder gets in lite mode.
pub const LITE_DECODER_THREADS: usize = 4;
/// [`frame_budget`] in lite mode.
pub const LITE_FRAME_BUDGET: usize = 256 << 20;
/// The monitors' frame cache (Settings ▸ Memory) is capped at this in lite mode (MB).
pub const LITE_MONITOR_CACHE_MB: u32 = 256;

/// Turn lite mode on or off for the process; call before opening media.
pub fn set_lite(on: bool) {
    LITE.store(on, Ordering::Relaxed);
}

/// Whether lite mode is on.
pub fn lite() -> bool {
    LITE.load(Ordering::Relaxed)
}

/// Decoded frames all caches may hold together (see [`crate::gop`]).
pub fn frame_budget() -> usize {
    frame_budget_for(lite())
}

/// The worker threads for a new decoder that would otherwise use `threads`.
pub fn decoder_threads(threads: usize) -> usize {
    decoder_threads_for(lite(), threads)
}

/// [`frame_budget`] with lite mode `on` or off; tests use it without touching the process switch.
fn frame_budget_for(on: bool) -> usize {
    if on { LITE_FRAME_BUDGET } else { crate::gop::FRAME_BUDGET }
}

/// [`decoder_threads`] with lite mode `on` or off.
fn decoder_threads_for(on: bool, threads: usize) -> usize {
    if on { threads.clamp(1, LITE_DECODER_THREADS) } else { threads.max(1) }
}

/// Available cores (at most 16), capped by [`decoder_threads`].
pub fn default_decoder_threads() -> usize {
    decoder_threads(std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).min(16))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests run in parallel and the decoder pool reads the switch once, so this test never sets it.
    #[test]
    fn lite_caps_threads_and_budget() {
        assert_eq!(decoder_threads_for(false, 16), 16);
        assert_eq!(decoder_threads_for(false, 0), 1);
        assert_eq!(frame_budget_for(false), crate::gop::FRAME_BUDGET);
        assert_eq!(decoder_threads_for(true, 16), LITE_DECODER_THREADS);
        assert_eq!(decoder_threads_for(true, 2), 2);
        assert_eq!(decoder_threads_for(true, 0), 1);
        assert_eq!(frame_budget_for(true), LITE_FRAME_BUDGET);
    }
}
