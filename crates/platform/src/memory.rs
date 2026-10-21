//! The machine's physical memory, for lite mode (`filmcraft_codecs::lite`): the desktop app turns
//! it on by itself on machines with little RAM.

/// Total physical memory in bytes; `None` where it cannot be read.
pub fn total_bytes() -> Option<u64> {
    imp::total_bytes().filter(|&b| b > 0)
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
mod imp {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    pub fn total_bytes() -> Option<u64> {
        let mut status = MEMORYSTATUSEX { dwLength: u32::try_from(std::mem::size_of::<MEMORYSTATUSEX>()).ok()?, ..Default::default() };
        // SAFETY: `status` is a valid, writable MEMORYSTATUSEX with `dwLength` set as the API requires.
        unsafe { GlobalMemoryStatusEx(&mut status) }.ok()?;
        Some(status.ullTotalPhys)
    }
}

#[cfg(target_os = "linux")]
mod imp {
    pub fn total_bytes() -> Option<u64> {
        parse_meminfo(&std::fs::read_to_string("/proc/meminfo").ok()?)
    }

    /// `MemTotal:  8049116 kB` → bytes.
    pub fn parse_meminfo(text: &str) -> Option<u64> {
        let line = text.lines().find_map(|l| l.strip_prefix("MemTotal:"))?;
        let kb: u64 = line.trim().strip_suffix("kB")?.trim().parse().ok()?;
        kb.checked_mul(1024)
    }

    #[cfg(test)]
    #[test]
    fn meminfo_parses() {
        assert_eq!(parse_meminfo("MemTotal:        8049116 kB\nMemFree: 1 kB\n"), Some(8_049_116 * 1024));
        assert_eq!(parse_meminfo("MemFree: 1 kB\n"), None);
        assert_eq!(parse_meminfo("MemTotal: lots kB\n"), None);
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod imp {
    pub fn total_bytes() -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn total_memory_is_plausible_where_known() {
        if let Some(b) = super::total_bytes() {
            assert!(b >= 256 << 20, "{b} bytes");
        }
    }
}
