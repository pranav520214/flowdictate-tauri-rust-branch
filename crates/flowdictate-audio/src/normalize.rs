//! # PCM Normalization
//!
//! Converts various native PCM sample formats into the canonical internal
//! representation (f32, mono, [-1.0, 1.0]).

/// Trait for sample types that can be normalized to f32.
pub trait Normalize {
    /// Converts a sample to f32 and normalizes it to the [-1.0, 1.0] range.
    fn normalize(self) -> f32;
}

impl Normalize for i16 {
    #[inline]
    fn normalize(self) -> f32 {
        self as f32 / 32768.0
    }
}

impl Normalize for i32 {
    #[inline]
    fn normalize(self) -> f32 {
        self as f32 / 2147483648.0
    }
}

impl Normalize for f32 {
    #[inline]
    fn normalize(self) -> f32 {
        // Clamp to [-1.0, 1.0] to prevent downstream instability
        self.clamp(-1.0, 1.0)
    }
}

/// Averages multiple channels into a single mono channel.
#[inline]
pub fn downmix_to_mono(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    let sum: f32 = frame.iter().sum();
    sum / frame.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_i16() {
        assert_eq!(0i16.normalize(), 0.0);
        assert_eq!(32767i16.normalize(), 32767.0 / 32768.0);
        assert_eq!((-32768i16).normalize(), -1.0);
    }

    #[test]
    fn test_normalize_i32() {
        assert_eq!(0i32.normalize(), 0.0);
        assert_eq!(2147483647i32.normalize(), 2147483647.0 / 2147483648.0);
        assert_eq!((-2147483648i32).normalize(), -1.0);
    }

    #[test]
    fn test_normalize_f32() {
        assert_eq!(0.5f32.normalize(), 0.5);
        assert_eq!(1.5f32.normalize(), 1.0);
        assert_eq!((-2.0f32).normalize(), -1.0);
    }

    #[test]
    fn test_downmix() {
        assert_eq!(downmix_to_mono(&[0.5, -0.5]), 0.0);
        assert_eq!(downmix_to_mono(&[1.0, 1.0]), 1.0);
        assert_eq!(downmix_to_mono(&[0.5]), 0.5);
    }
}
