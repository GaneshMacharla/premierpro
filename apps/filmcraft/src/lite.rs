//! Lite mode for machines with 8 GB of RAM and no dedicated GPU (`filmcraft_codecs::lite`).
//!
//! On top of the smaller decoder caches and thread counts set in the codecs crate, lite mode
//! here keeps the render and export threads to [`LITE_RENDER_THREADS`], composites the preview
//! on the CPU (no textures in GPU memory, which an integrated GPU takes from system RAM), skips
//! the GPU export renderer, and starts the monitors at 1/4 resolution while playing.

/// Machines with at most this much RAM start in lite mode unless told otherwise. An "8 GB"
/// machine reports a little less than 8 GiB, a 12 GB one well above this.
pub const AUTO_LITE_MAX_RAM: u64 = 9 << 30;

/// Threads of the global rayon pool (rendering, effects, export) in lite mode.
pub const LITE_RENDER_THREADS: usize = 4;

/// Whether to start in lite mode: `--lite` / `--no-lite`, else `FILMCRAFT_LITE` (`1`/`true`/`on`
/// or `0`/`false`/`off`), else whether the machine has at most [`AUTO_LITE_MAX_RAM`].
pub fn decide(flag: Option<bool>, env: Option<&str>, total_ram: Option<u64>) -> bool {
    if let Some(on) = flag {
        return on;
    }
    match env.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        Some("1" | "true" | "on" | "yes") => return true,
        Some("0" | "false" | "off" | "no") => return false,
        _ => {}
    }
    total_ram.is_some_and(|b| b <= AUTO_LITE_MAX_RAM)
}

/// Apply the process-wide part of lite mode; call before any media is opened or rendered.
pub fn apply(on: bool) {
    filmcraft_codecs::lite::set_lite(on);
    if on {
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(LITE_RENDER_THREADS);
        // Fails only when the pool already exists (then it keeps its size).
        if let Err(e) = rayon::ThreadPoolBuilder::new().num_threads(threads).build_global() {
            log::warn!("lite mode: render threads not capped: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_then_environment_then_ram() {
        const GB: u64 = 1 << 30;
        assert!(decide(Some(true), Some("0"), Some(64 * GB)));
        assert!(!decide(Some(false), Some("1"), Some(4 * GB)));
        assert!(decide(None, Some(" On "), Some(64 * GB)));
        assert!(!decide(None, Some("off"), Some(4 * GB)));
        // an unknown value falls through to the RAM
        assert!(decide(None, Some("maybe"), Some(7_800_000_000)));
        assert!(decide(None, None, Some(8 * GB)));
        assert!(!decide(None, None, Some(16 * GB)));
        assert!(!decide(None, None, None));
    }
}
