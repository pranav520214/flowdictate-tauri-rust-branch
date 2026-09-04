//! # Audio Resampling
//!
//! Resamples microphone audio from native rates (e.g., 48kHz, 44.1kHz) down
//! to 16kHz, which is the required format for whisper.cpp.

use rubato::{
    SincInterpolationParameters, SincInterpolationType, Resampler, SincFixedIn, WindowFunction,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResampleError {
    #[error("Resampler initialization failed: {0}")]
    Initialization(String),
    #[error("Resampling process failed: {0}")]
    Process(String),
}

/// A wrapper around `rubato`'s Sinc resampler.
pub struct AudioResampler {
    inner: SincFixedIn<f32>,
    input_buffer: Vec<Vec<f32>>,
    output_buffer: Vec<Vec<f32>>,
    chunk_size: usize,
}

impl AudioResampler {
    /// Creates a new resampler from the specified input rate to 16,000 Hz.
    /// `chunk_size` is the number of frames (samples per channel) expected per call.
    pub fn new(input_rate: u32, chunk_size: usize) -> Result<Self, ResampleError> {
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        // 16kHz target
        let ratio = 16000.0 / input_rate as f64;

        let inner = SincFixedIn::<f32>::new(ratio, 2.0, params, chunk_size, 1)
            .map_err(|e| ResampleError::Initialization(e.to_string()))?;

        // Allocate input and output buffers (mono, so 1 channel)
        let input_buffer = inner.input_buffer_allocate(false);
        let output_buffer = inner.output_buffer_allocate(true);

        Ok(Self {
            inner,
            input_buffer,
            output_buffer,
            chunk_size,
        })
    }

    /// Resamples a chunk of mono f32 data.
    /// The input slice length should match `chunk_size`.
    /// Returns a slice of the resampled data.
    pub fn process(&mut self, input: &[f32]) -> Result<&[f32], ResampleError> {
        if input.len() != self.chunk_size {
            return Err(ResampleError::Process(format!(
                "Expected chunk size {}, got {}",
                self.chunk_size,
                input.len()
            )));
        }

        // Copy input to the input buffer
        self.input_buffer[0].clear();
        self.input_buffer[0].extend_from_slice(input);

        // Process
        let (_, out_len) = self
            .inner
            .process_into_buffer(&self.input_buffer, &mut self.output_buffer, None)
            .map_err(|e| ResampleError::Process(e.to_string()))?;

        Ok(&self.output_buffer[0][..out_len])
    }

    /// Returns the expected input chunk size in frames.
    pub fn input_frames_next(&self) -> usize {
        self.inner.input_frames_next()
    }
}
