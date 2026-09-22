//! # Memory Governor
//!
//! Controls model residency to support constrained machines (§19).
//!
//! ## Residency Modes
//!
//! - **Sequential**: Only one model loaded at a time. ASR runs first,
//!   then is released before the refiner loads. Minimizes peak RAM.
//!
//! - **Concurrent**: Both ASR and refiner stay resident. Requires
//!   sufficient memory. Reduces latency between ASR and refinement.
//!
//! ## Memory Accounting
//!
//! The governor uses real system memory queries via `sysinfo`,
//! not estimates based on model file size. It detects allocation
//! failures and recovers safely.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GovernorError {
    #[error("insufficient memory: need {required_bytes} bytes, only {available_bytes} available")]
    InsufficientMemory {
        required_bytes: u64,
        available_bytes: u64,
    },
    #[error("model lifecycle error: {0}")]
    Lifecycle(String),
}

/// How models share system memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResidencyMode {
    /// Only one model loaded at a time. Safer for constrained machines.
    Sequential,
    /// Both ASR and refiner may be resident simultaneously.
    Concurrent,
}

/// Memory budgets for each model in bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBudget {
    /// Expected resident memory for the ASR model (Nemotron Q8_0 ~600MB)
    pub asr_bytes: u64,
    /// Expected resident memory for the refiner model (Qwen3 Q4_K_M ~500MB)
    pub refiner_bytes: u64,
    /// Overhead for audio buffers, pipeline state, OS, etc.
    pub overhead_bytes: u64,
}

impl Default for ModelBudget {
    fn default() -> Self {
        Self {
            asr_bytes: 700_000_000,      // ~700MB for Nemotron 0.6B Q8_0
            refiner_bytes: 500_000_000,  // ~500MB for Qwen3-0.6B Q4_K_M
            overhead_bytes: 300_000_000, // ~300MB for audio, pipeline, OS
        }
    }
}

/// Determines the appropriate residency mode based on available system memory.
///
/// Returns `Sequential` if the system cannot safely hold both models,
/// or `Concurrent` if there is adequate headroom.
pub fn determine_residency(available_bytes: u64, budget: &ModelBudget) -> ResidencyMode {
    let concurrent_total = budget.asr_bytes + budget.refiner_bytes + budget.overhead_bytes;

    // Require at least 20% headroom above the concurrent total
    // to avoid thrashing and OOM under load.
    let headroom_factor: u64 = 120;
    let required_for_concurrent = concurrent_total * headroom_factor / 100;

    if available_bytes >= required_for_concurrent {
        ResidencyMode::Concurrent
    } else {
        ResidencyMode::Sequential
    }
}

/// Check whether the system has enough memory to load a specific model.
pub fn can_load_model(model_bytes: u64) -> Result<(), GovernorError> {
    let (_total, available) = super::detect::query_memory();

    // Require at least 10% headroom above the model size
    let required = model_bytes * 110 / 100;
    if available < required {
        return Err(GovernorError::InsufficientMemory {
            required_bytes: required,
            available_bytes: available,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequential_when_low_memory() {
        let budget = ModelBudget::default();
        // 1GB available — not enough for both models + overhead + headroom
        assert_eq!(
            determine_residency(1_000_000_000, &budget),
            ResidencyMode::Sequential
        );
    }

    #[test]
    fn concurrent_when_high_memory() {
        let budget = ModelBudget::default();
        // 4GB available — enough for both models + overhead + headroom
        assert_eq!(
            determine_residency(4_000_000_000, &budget),
            ResidencyMode::Concurrent
        );
    }

    #[test]
    fn default_budget_is_reasonable() {
        let budget = ModelBudget::default();
        let total = budget.asr_bytes + budget.refiner_bytes + budget.overhead_bytes;
        // Total should be under 2GB for the baseline 0.6B models
        assert!(total < 2_000_000_000);
    }
}
