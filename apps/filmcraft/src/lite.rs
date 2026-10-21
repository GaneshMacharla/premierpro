//! Lite mode for machines with 8 GB of RAM and no dedicated GPU (`filmcraft_codecs::lite`).
//!
//! On top of the smaller decoder caches and thread counts set in the codecs crate, lite mode
//! here keeps the render and export threads to [`LITE_RENDER_THREADS`], composites the preview
//! on the CPU (no textures in GPU memory, which an integrated GPU takes from system RAM), skips
//! the GPU export renderer, and starts the monitors at 1/4 resolution while playing.
//!
//! Chosen in two steps: [`decide`] at start-up from the flags and the RAM, then [`Choice::with_adapter`]
//! once the window has its graphics adapter, before any media is opened. An automatic choice
//! is undone on a dedicated GPU, which brings its own video memory.

/// Machines with at most this much RAM start in lite mode unless told otherwise. An "8 GB"
/// machine reports a little less than 8 GiB, a 12 GB one well above this.
pub const AUTO_LITE_MAX_RAM: u64 = 9 << 30;

/// Threads of the global rayon pool (rendering, effects, export) in lite mode.
pub const LITE_RENDER_THREADS: usize = 4;

/// Whether to use lite mode, and whether the user said so or the RAM did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// `--lite` / `--no-lite` or `FILMCRAFT_LITE`: kept whatever the GPU.
    Forced(bool),
    /// From the RAM: undone on a dedicated GPU ([`Choice::with_adapter`]).
    Auto(bool),
}

impl Choice {
    /// The final choice once the window's adapter is known (`None`: no wgpu adapter).
    pub fn with_adapter(self, device: Option<eframe::wgpu::DeviceType>) -> bool {
        match self {
            Choice::Forced(on) => on,
            Choice::Auto(on) => on && device != Some(eframe::wgpu::DeviceType::DiscreteGpu),
        }
    }
}

/// `--lite` / `--no-lite`, else `FILMCRAFT_LITE` (`1`/`true`/`on` or `0`/`false`/`off`), else
/// whether the machine has at most [`AUTO_LITE_MAX_RAM`].
pub fn decide(flag: Option<bool>, env: Option<&str>, total_ram: Option<u64>) -> Choice {
    if let Some(on) = flag {
        return Choice::Forced(on);
    }
    match env.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        Some("1" | "true" | "on" | "yes") => return Choice::Forced(true),
        Some("0" | "false" | "off" | "no") => return Choice::Forced(false),
        _ => {}
    }
    Choice::Auto(total_ram.is_some_and(|b| b <= AUTO_LITE_MAX_RAM))
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

    use Choice::{Auto, Forced};
    use eframe::wgpu::DeviceType;

    #[test]
    fn flag_then_environment_then_ram() {
        const GB: u64 = 1 << 30;
        assert_eq!(decide(Some(true), Some("0"), Some(64 * GB)), Forced(true));
        assert_eq!(decide(Some(false), Some("1"), Some(4 * GB)), Forced(false));
        assert_eq!(decide(None, Some(" On "), Some(64 * GB)), Forced(true));
        assert_eq!(decide(None, Some("off"), Some(4 * GB)), Forced(false));
        // an unknown value falls through to the RAM
        assert_eq!(decide(None, Some("maybe"), Some(7_800_000_000)), Auto(true));
        assert_eq!(decide(None, None, Some(8 * GB)), Auto(true));
        assert_eq!(decide(None, None, Some(16 * GB)), Auto(false));
        assert_eq!(decide(None, None, None), Auto(false));
    }

    #[test]
    fn dedicated_gpu_undoes_only_the_automatic_choice() {
        assert!(!Auto(true).with_adapter(Some(DeviceType::DiscreteGpu)));
        assert!(Auto(true).with_adapter(Some(DeviceType::IntegratedGpu)));
        assert!(Auto(true).with_adapter(Some(DeviceType::Cpu)));
        assert!(Auto(true).with_adapter(None));
        assert!(!Auto(false).with_adapter(Some(DeviceType::IntegratedGpu)));
        assert!(Forced(true).with_adapter(Some(DeviceType::DiscreteGpu)));
        assert!(!Forced(false).with_adapter(Some(DeviceType::Cpu)));
    }
}
