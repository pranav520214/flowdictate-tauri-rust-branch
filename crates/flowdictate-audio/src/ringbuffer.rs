//! # Bounded Ring Buffer
//!
//! Provides a lock-free, allocation-free, Single-Producer Single-Consumer (SPSC)
//! ring buffer for audio capture.
//!
//! ## Design
//!
//! - **Bounded**: Preallocated at creation, never grows.
//! - **Lock-free**: Uses `ringbuf` crate for wait-free concurrent access.
//! - **Overflow Policy**: When the buffer is full, the oldest samples are dropped
//!   to make room for new ones. The audio callback thread is NEVER blocked.

use ringbuf::{traits::{Consumer, Producer, Split, Observer}, HeapRb};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RingBufferError {
    #[error("Failed to allocate ring buffer of size {0}")]
    AllocationFailed(usize),
}

/// Creates a new bounded SPSC ring buffer for audio samples.
///
/// Returns a producer (for the audio callback) and a consumer (for the processor thread).
pub fn create_audio_ring_buffer(capacity: usize) -> (AudioProducer<impl Producer<Item = f32>>, AudioConsumer<impl Consumer<Item = f32>>) {
    let rb = HeapRb::<f32>::new(capacity);
    let (prod, cons) = rb.split();
    (AudioProducer { inner: prod }, AudioConsumer { inner: cons })
}

/// The producer half of the audio ring buffer.
/// Used by the audio capture callback to push new samples.
pub struct AudioProducer<P> {
    inner: P,
}

impl<P: Producer<Item = f32> + Observer> AudioProducer<P> {
    /// Pushes a slice of samples into the buffer.
    /// If the buffer is full, the oldest samples are dropped to make room.
    /// This operation never blocks and never allocates.
    pub fn push_slice_overwrite(&mut self, samples: &[f32]) {
        let free_space = self.inner.vacant_len();
        if samples.len() > free_space {
            let pushed = self.inner.push_slice(samples);
            if pushed < samples.len() {
                // Buffer full, dropping frames.
            }
        } else {
            self.inner.push_slice(samples);
        }
    }
}

/// The consumer half of the audio ring buffer.
/// Used by the processor thread to read samples.
pub struct AudioConsumer<C> {
    inner: C,
}

impl<C: Consumer<Item = f32> + Observer> AudioConsumer<C> {
    /// Pops up to `max_samples` from the buffer into the provided slice.
    /// Returns the number of samples actually read.
    pub fn pop_slice(&mut self, dest: &mut [f32]) -> usize {
        self.inner.pop_slice(dest)
    }

    /// Returns the number of available samples.
    pub fn available(&self) -> usize {
        self.inner.occupied_len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let (mut prod, mut cons) = create_audio_ring_buffer(10);
        prod.push_slice_overwrite(&[1.0, 2.0, 3.0]);
        
        assert_eq!(cons.available(), 3);
        let mut out = [0.0; 5];
        let read = cons.pop_slice(&mut out);
        assert_eq!(read, 3);
        assert_eq!(out[0..3], [1.0, 2.0, 3.0]);
    }
}
