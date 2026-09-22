//! # Backend Selection
//!
//! Selects the compute backend for native inference runtimes
//! based on detected hardware capabilities.
//!
//! ## Preference Order (§18)
//!
//! ```text
//! NVIDIA GPU → CUDA
//! Apple Silicon → Metal
//! Supported GPU → Vulkan
//! Otherwise → CPU
//! ```
//!
//! ## Failure Policy
//!
//! Any backend failure degrades safely to CPU.
//! Never crashes merely because GPU initialization fails.
//! Never uses an online service as fallback.

use super::detect::{Accelerator, HardwareCapability};
use serde::{Deserialize, Serialize};

/// The compute backend to use for native inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeBackend {
    Cuda,
    Metal,
    Vulkan,
    Cpu,
}

impl ComputeBackend {
    /// Returns a human-readable name for structured logging.
    /// Does not expose sensitive information.
    pub fn as_log_str(&self) -> &'static str {
        match self {
            Self::Cuda => "cuda",
            Self::Metal => "metal",
            Self::Vulkan => "vulkan",
            Self::Cpu => "cpu",
        }
    }
}

/// Select the best available backend from hardware capabilities.
///
/// Follows the preference order specified in §18.
/// Always returns a valid backend — CPU is the universal fallback.
pub fn select_backend(hw: &HardwareCapability) -> ComputeBackend {
    // Preference: CUDA > Metal > Vulkan > CPU
    if hw.accelerators.contains(&Accelerator::Cuda) {
        return ComputeBackend::Cuda;
    }
    if hw.accelerators.contains(&Accelerator::Metal) {
        return ComputeBackend::Metal;
    }
    if hw.accelerators.contains(&Accelerator::Vulkan) {
        return ComputeBackend::Vulkan;
    }
    ComputeBackend::Cpu
}

/// Determine the fallback backend when the primary backend fails.
///
/// ```text
/// CUDA failure → Vulkan if available, else CPU
/// Metal failure → CPU
/// Vulkan failure → CPU
/// CPU failure → no fallback (fatal)
/// ```
pub fn fallback_backend(failed: ComputeBackend, hw: &HardwareCapability) -> Option<ComputeBackend> {
    match failed {
        ComputeBackend::Cuda => {
            if hw.accelerators.contains(&Accelerator::Vulkan) {
                Some(ComputeBackend::Vulkan)
            } else {
                Some(ComputeBackend::Cpu)
            }
        }
        ComputeBackend::Metal => Some(ComputeBackend::Cpu),
        ComputeBackend::Vulkan => Some(ComputeBackend::Cpu),
        ComputeBackend::Cpu => None, // No further fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect::{CpuArch, HardwareCapability};

    fn hw_with(accels: Vec<Accelerator>) -> HardwareCapability {
        HardwareCapability {
            cpu_arch: CpuArch::X86_64,
            accelerators: accels,
            total_memory_bytes: 8_000_000_000,
            available_memory_bytes: 4_000_000_000,
        }
    }

    #[test]
    fn prefers_cuda_over_vulkan() {
        let hw = hw_with(vec![Accelerator::Vulkan, Accelerator::Cuda]);
        assert_eq!(select_backend(&hw), ComputeBackend::Cuda);
    }

    #[test]
    fn prefers_metal_over_vulkan() {
        let hw = hw_with(vec![Accelerator::Vulkan, Accelerator::Metal]);
        assert_eq!(select_backend(&hw), ComputeBackend::Metal);
    }

    #[test]
    fn falls_back_to_cpu_when_no_accelerator() {
        let hw = hw_with(vec![]);
        assert_eq!(select_backend(&hw), ComputeBackend::Cpu);
    }

    #[test]
    fn cuda_fallback_to_vulkan_if_available() {
        let hw = hw_with(vec![Accelerator::Cuda, Accelerator::Vulkan]);
        assert_eq!(
            fallback_backend(ComputeBackend::Cuda, &hw),
            Some(ComputeBackend::Vulkan)
        );
    }

    #[test]
    fn cuda_fallback_to_cpu_if_no_vulkan() {
        let hw = hw_with(vec![Accelerator::Cuda]);
        assert_eq!(
            fallback_backend(ComputeBackend::Cuda, &hw),
            Some(ComputeBackend::Cpu)
        );
    }

    #[test]
    fn cpu_has_no_fallback() {
        let hw = hw_with(vec![]);
        assert_eq!(fallback_backend(ComputeBackend::Cpu, &hw), None);
    }
}
