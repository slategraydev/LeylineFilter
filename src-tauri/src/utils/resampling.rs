use rubato::{Resampler, FastFixedIn, PolynomialDegree};

/// A utility that handles resampling and buffering for modules that require
/// a fixed sample rate and block size (like AI/RNN models).
pub struct AudioBlockProcessor {
    target_rate: f32,
    source_rate: f32,
    block_size: usize,

    // Accumulators to handle mismatch between system chunks and module blocks
    input_accumulator: Vec<f32>,
    process_accumulator: Vec<f32>,
    resample_out_accumulator: Vec<f32>,
    output_accumulator: Vec<f32>,

    resampler_in: Option<FastFixedIn<f32>>,
    resampler_out: Option<FastFixedIn<f32>>,

    // Pre-allocated buffers for resampling to avoid frequent allocations
    resample_in_buf: Vec<Vec<f32>>,

    // Reusable block buffer
    block_buffer: Vec<f32>,
}

impl AudioBlockProcessor {
    pub fn new(source_rate: f32, target_rate: f32, block_size: usize) -> Self {
        let mut s = Self {
            target_rate,
            source_rate,
            block_size,
            input_accumulator: Vec::with_capacity(block_size * 4),
            process_accumulator: Vec::with_capacity(block_size * 4),
            resample_out_accumulator: Vec::with_capacity(block_size * 4),
            output_accumulator: Vec::with_capacity(block_size * 4),
            resampler_in: None,
            resampler_out: None,
            resample_in_buf: vec![vec![0.0; 1024]], // Pre-allocate with reasonable max
            block_buffer: vec![0.0; block_size],
        };
        s.setup_resamplers();
        // Initial silence to prime the pipeline latency (1 block)
        s.output_accumulator.extend(std::iter::repeat(0.0).take(block_size));
        s
    }

            fn setup_resamplers(&mut self) {
                if (self.source_rate - self.target_rate).abs() > 1.0 {
                    log::info!("AudioBlockProcessor: Setting up resamplers ({}Hz -> {}Hz)", self.source_rate, self.target_rate);
                    // Match RNNoise block size for stability
                    let chunk_size = 480;
                        self.resampler_in = FastFixedIn::<f32>::new(
                    self.target_rate as f64 / self.source_rate as f64,
                    2.0,
                    PolynomialDegree::Cubic,
                    chunk_size,
                    1,
                ).ok();

                self.resampler_out = FastFixedIn::<f32>::new(
                    self.source_rate as f64 / self.target_rate as f64,
                    2.0,
                    PolynomialDegree::Cubic,
                    chunk_size,
                    1,
                ).ok();
            } else {
                log::info!("AudioBlockProcessor: No resampling needed ({}Hz)", self.source_rate);
                self.resampler_in = None;
                self.resampler_out = None;
            }
        }
        pub fn prepare(&mut self, source_rate: f32) {
        if (self.source_rate - source_rate).abs() > 1.0 {
            self.source_rate = source_rate;
            self.setup_resamplers();
            self.input_accumulator.clear();
            self.process_accumulator.clear();
            self.resample_out_accumulator.clear();
            self.output_accumulator.clear();
            self.output_accumulator.extend(std::iter::repeat(0.0).take(self.block_size));
        }
    }

    pub fn process<F>(&mut self, samples: &mut [f32], mut f: F)
    where
        F: FnMut(&[f32], &mut [f32]),
    {
        // 1. Add new samples to input accumulator
        self.input_accumulator.extend_from_slice(samples);

        // 2. Resample from input accumulator to process accumulator if needed
        if let Some(ref mut rs) = self.resampler_in {
            while self.input_accumulator.len() >= rs.input_frames_next() {
                let needed = rs.input_frames_next();

                // Copy to pre-allocated resample buffer
                if self.resample_in_buf[0].len() < needed {
                    self.resample_in_buf[0].resize(needed, 0.0);
                }
                self.resample_in_buf[0][..needed].copy_from_slice(&self.input_accumulator[..needed]);
                self.input_accumulator.drain(..needed);

                // We have to use the slice because rubato expects the inner vec length to match needed
                let view = [&self.resample_in_buf[0][..needed]];
                if let Ok(resampled) = rs.process(&view, None) {
                    self.process_accumulator.extend_from_slice(&resampled[0]);
                }
            }
        } else {
            self.process_accumulator.extend_from_slice(&self.input_accumulator);
            self.input_accumulator.clear();
        }

        // 3. Process fixed-size blocks from process accumulator
        while self.process_accumulator.len() >= self.block_size {
            let input_block = &self.process_accumulator[..self.block_size];
            f(input_block, &mut self.block_buffer);
            self.process_accumulator.drain(..self.block_size);

            if self.resampler_out.is_some() {
                self.resample_out_accumulator.extend_from_slice(&self.block_buffer);
            } else {
                self.output_accumulator.extend_from_slice(&self.block_buffer);
            }
        }

        // 4. Resample back to source rate if needed
        if let Some(ref mut rs) = self.resampler_out {
            while self.resample_out_accumulator.len() >= rs.input_frames_next() {
                let needed = rs.input_frames_next();

                if self.resample_in_buf[0].len() < needed {
                    self.resample_in_buf[0].resize(needed, 0.0);
                }
                self.resample_in_buf[0][..needed].copy_from_slice(&self.resample_out_accumulator[..needed]);
                self.resample_out_accumulator.drain(..needed);

                let view = [&self.resample_in_buf[0][..needed]];
                if let Ok(resampled) = rs.process(&view, None) {
                    self.output_accumulator.extend_from_slice(&resampled[0]);
                }
            }
        }

        // 5. Fill the output samples from output accumulator
        let out_len = samples.len();
        if self.output_accumulator.len() >= out_len {
            samples.copy_from_slice(&self.output_accumulator[..out_len]);
            self.output_accumulator.drain(..out_len);
        } else {
            // Safety fallback: if we don't have enough, fill available and then silence
            let available = self.output_accumulator.len();
            samples[..available].copy_from_slice(&self.output_accumulator[..available]);
            samples[available..].fill(0.0);
            self.output_accumulator.clear();
        }
    }
}
