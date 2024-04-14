use std::f32::consts::PI;

use crate::{clock::SimpleClock, DiscretePI};
use num::complex::{Complex32, ComplexFloat};

struct PLL {
    internal_clock: SimpleClock,
    phase_controller: DiscretePI<f32, f32>,
}

impl PLL {
    fn new(rate: f32, relative_noise_bandwidth: f32, dampening_factor: f32) -> Self {
        let kp = (4.0 * dampening_factor) / (dampening_factor + 1.0 / (4.0 * dampening_factor))
            * relative_noise_bandwidth;
        let ki = (4.0) / (dampening_factor + 1.0 / (4.0 * dampening_factor)).powi(2)
            * relative_noise_bandwidth.powi(2);

        dbg!(kp);
        dbg!(ki);

        Self {
            internal_clock: SimpleClock::from_rate(rate),
            phase_controller: DiscretePI::new(kp, ki),
        }
    }

    fn tick(&mut self, val: Complex32) -> Complex32 {
        let input_phase = val.arg();
        let internal_phase = self.internal_clock.phase();

        let mut diff = input_phase - internal_phase;
        if diff < -PI {
            diff += 2.0 * PI;
        }

        dbg!(&diff, input_phase, internal_phase);
        let phase_adjust = self.phase_controller.update(diff);
        let res = self.internal_clock.cis();

        self.internal_clock.advance_by(phase_adjust);
        self.internal_clock.tick();
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{self, create_output_directory};
    use crate::{clock::SimpleClock, pll::PLL};
    use std::f32::consts::PI;

    #[test]
    fn perform_pll_track_1() {
        const N: usize = 300;
        const OMEGA_SENDER: f32 = 2.0 * PI / 5.5;
        const OMEGA_RECEIVER: f32 = 2.0 * PI / 4.1;

        let mut orig_signal_gen = SimpleClock::new(PI, OMEGA_SENDER);
        let mut orig_signal_shift = SimpleClock::new(PI, OMEGA_SENDER * 7.0);

        let mut pll = PLL::new(OMEGA_RECEIVER, 0.05, 3.0);
        let mut orig_signal = Vec::with_capacity(N);
        let mut pll_signal = Vec::with_capacity(N);
        let t: Vec<_> = (0..N).collect();

        for _ in 0..N {
            let signal_in = orig_signal_gen.cis();
            let signal_shift = orig_signal_shift.sin();
            let signal_out = pll.tick(signal_in);

            orig_signal.push(signal_in.re);
            pll_signal.push(signal_out.re);

            orig_signal_gen.tick();
            orig_signal_shift.tick();
            orig_signal_gen.advance_by(signal_shift * 0.3);
        }

        let output_dir = create_output_directory("PLL");

        //println!("{:?}", orig_signal);
        use plotly::{ImageFormat, Plot, Scatter};
        let mut plot = Plot::new();
        let trace_in = Scatter::new(t.clone(), orig_signal);
        let trace_out = Scatter::new(t.clone(), pll_signal);
        plot.add_trace(trace_in);
        plot.add_trace(trace_out);
        plot.write_image(
            output_dir.join("Track1.png"),
            ImageFormat::PNG,
            8000,
            600,
            1.0,
        );
    }
}
