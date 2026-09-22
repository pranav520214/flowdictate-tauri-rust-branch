//! # Hardware Detection
//!
//! Detects CPU architecture and available accelerators without
//! requiring network access or privileged operations.

use serde::{Deserialize, Serialize};

/// CPU architecture of the host system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuArch {
    X86_64,
    Aarch64,
    Unknown,
}

/// A specific compute accelerator available on the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accelerator {
    /// NVIDIA CUDA (requires nvcuda.dll / libcuda.so)
    Cuda,
    /// Apple Metal (macOS / iOS only)
    Metal,
    /// Vulkan (cross-platform GPU compute)
    Vulkan,
}

/// Complete hardware capability report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapability {
    pub cpu_arch: CpuArch,
    pub accelerators: Vec<Accelerator>,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
}

/// Detect the CPU architecture at compile time.
pub fn detect_cpu_arch() -> CpuArch {
    #[cfg(target_arch = "x86_64")]
    {
        CpuArch::X86_64
    }
    #[cfg(target_arch = "aarch64")]
    {
        CpuArch::Aarch64
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        CpuArch::Unknown
    }
}

/// Detect available accelerators by probing for driver libraries.
///
/// This function checks for the existence of GPU driver shared libraries
/// on the filesystem. It does not load or execute them — it only confirms
/// their presence as a prerequisite for backend selection.
pub fn detect_accelerators() -> Vec<Accelerator> {
    let mut found = Vec::new();

    if detect_cuda() {
        found.push(Accelerator::Cuda);
    }
    if detect_metal() {
        found.push(Accelerator::Metal);
    }
    if detect_vulkan() {
        found.push(Accelerator::Vulkan);
    }

    found
}

/// Query system memory using sysinfo.
pub fn query_memory() -> (u64, u64) {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_memory();
    (sys.total_memory(), sys.available_memory())
}

/// Build a complete hardware capability report.
pub fn detect_hardware() -> HardwareCapability {
    let (total, available) = query_memory();
    HardwareCapability {
        cpu_arch: detect_cpu_arch(),
        accelerators: detect_accelerators(),
        total_memory_bytes: total,
        available_memory_bytes: available,
    }
}

// ── Private detection helpers ──

fn detect_cuda() -> bool {
    #[cfg(target_os = "windows")]
    {
        std::path::Path::new("C:\\Windows\\System32\\nvcuda.dll").exists()
    }
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new("/usr/lib/x86_64-linux-gnu/libcuda.so").exists()
            || std::path::Path::new("/usr/lib/x86_64-linux-gnu/libcuda.so.1").exists()
            || std::path::Path::new("/usr/lib/libcuda.so").exists()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

fn detect_metal() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Metal is available on all macOS 10.14+ systems
        true
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

fn detect_vulkan() -> bool {
    #[cfg(target_os = "windows")]
    {
        std::path::Path::new("C:\\Windows\\System32\\vulkan-1.dll").exists()
    }
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new("/usr/lib/x86_64-linux-gnu/libvulkan.so.1").exists()
            || std::path::Path::new("/usr/lib/libvulkan.so.1").exists()
    }
    #[cfg(target_os = "macos")]
    {
        // MoltenVK may be available but Metal is preferred
        false
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_cpu_arch_returns_known_value() {
        let arch = detect_cpu_arch();
        // On test machines we expect x86_64 or aarch64
        assert_ne!(arch, CpuArch::Unknown);
    }

    #[test]
    fn detect_hardware_returns_positive_memory() {
        let hw = detect_hardware();
        assert!(hw.total_memory_bytes > 0);
        // Available may be 0 on extremely constrained systems,
        // but total should always be positive.
    }

    #[test]
    fn accelerator_list_does_not_panic() {
        // This test verifies that accelerator detection doesn't
        // panic on any platform, even if no accelerators are found.
        let _accel = detect_accelerators();
    }
}
