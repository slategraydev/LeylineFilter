// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// PARAMETER SMOOTHING
// One-pole low-pass filter for artifact-free DSP parameter transitions.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// A utility for smoothing parameter changes in the audio thread to prevent clicks and pops.
///
/// It uses a simple one-pole low-pass filter (exponential smoothing) to ramp
/// values towards a target.
pub struct ParameterSmoother {
    current_value: f32,
    target_value: f32,
    coeff: f32,
}

impl ParameterSmoother {
    /// Creates a new smoother with an initial value and a smoothing time.
    ///
    /// # Arguments
    /// * `initial_value` - The starting value.
    /// * `smoothing_ms` - The time in milliseconds to reach approx 63% of the target.
    /// * `sample_rate` - The current processing sample rate.
    pub fn new(initial_value: f32, smoothing_ms: f32, sample_rate: f32) -> Self {
        let mut s = Self {
            current_value: initial_value,
            target_value: initial_value,
            coeff: 0.0,
        };
        s.set_smoothing_ms(smoothing_ms, sample_rate);
        s
    }

    /// Updates the smoothing time constant.
    pub fn set_smoothing_ms(&mut self, smoothing_ms: f32, sample_rate: f32) {
        if smoothing_ms <= 0.0 {
            self.coeff = 0.0;
        } else {
            // Standard formula for one-pole filter coefficient
            self.coeff = (-1.0 / (smoothing_ms * sample_rate / 1000.0)).exp();
        }
    }

    /// Sets a new target value.
    pub fn set_target(&mut self, target: f32) {
        self.target_value = target;
    }

    /// Returns the target value.
    pub fn get_target(&self) -> f32 {
        self.target_value
    }

    /// Forces the smoother to a specific value, bypassing any smoothing.
    #[allow(dead_code)]
    pub fn reset(&mut self, value: f32) {
        self.current_value = value;
        self.target_value = value;
    }

    /// Returns the current smoothed value and advances the filter by one step.
    #[inline(always)]
    pub fn next(&mut self) -> f32 {
        if (self.current_value - self.target_value).abs() < 1e-6 {
            self.current_value = self.target_value;
        } else {
            self.current_value =
                self.coeff * (self.current_value - self.target_value) + self.target_value;
        }
        self.current_value
    }

    /// Returns the current value without advancing.
    pub fn current(&self) -> f32 {
        self.current_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoother_settling() {
        let mut s = ParameterSmoother::new(0.0, 10.0, 1000.0); // 10ms smoothing @ 1kHz
        s.set_target(1.0);

        // After 10 samples (10ms), it should be approx 63.2% of the way (1 - 1/e)
        let mut val = 0.0;
        for _ in 0..10 {
            val = s.next();
        }
        assert!(val > 0.6 && val < 0.7);

        // After many more samples, it should reach the target
        for _ in 0..1000 {
            val = s.next();
        }
        assert!((val - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_smoother_reset() {
        let mut s = ParameterSmoother::new(0.0, 10.0, 1000.0);
        s.set_target(1.0);
        s.next();
        assert!(s.current() > 0.0);

        s.reset(0.5);
        assert_eq!(s.current(), 0.5);
        assert_eq!(s.next(), 0.5);
    }
}
