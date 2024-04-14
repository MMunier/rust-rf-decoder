use std::f32::consts::PI;

use num::complex::Complex32;

#[derive(Debug)]
pub struct SimpleClock {
    phase: f32,
    rate: f32,
}

impl Default for SimpleClock {
    fn default() -> Self {
        Self {
            phase: 0.0,
            rate: 2.0 * PI,
        }
    }
}

impl SimpleClock {
    pub fn new(phase: f32, rate: f32) -> Self {
        Self { phase, rate }
    }

    pub fn from_rate(rate: f32) -> Self {
        Self {
            rate,
            ..Self::default()
        }
    }

    /// Returns the timer phase overrun,
    /// if a full period has occured
    pub fn tick(&mut self) -> Option<f32> {
        self.advance_by(self.rate)
    }

    /// Manually reverting a clock should NOT yield another event,
    /// therefor the phase is allowed to go into the negatives
    ///
    pub fn advance_by(&mut self, phase_diff: f32) -> Option<f32> {
        self.phase += phase_diff;
        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
            return Some(self.phase);
        }

        None
    }

    pub fn phase(&self) -> f32 {
        self.phase
    }

    #[inline]
    pub fn sin(&self) -> f32 {
        self.phase.sin()
    }

    #[inline]
    pub fn cos(&self) -> f32 {
        self.phase.cos()
    }

    #[inline]
    pub fn cis(&self) -> Complex32 {
        Complex32::cis(self.phase)
    }
}
