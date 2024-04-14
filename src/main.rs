use bitstream_decoder::BitStreamDecoder;
use clock::SimpleClock;
use fir_interpolator_taps::{FIRInterpolator, Interpolateable};
use num::complex::ComplexFloat;
use num::Num;
use num::{complex::Complex32, Complex};
use std::env::args;
use std::f32::consts::PI;
use std::fmt::Debug;
use std::io::{BufReader, Read, SeekFrom, Write};
use std::ops::{Index, Mul};
use std::{fs, io::Seek};
use syncword::SyncwordPacketizer;

use crate::bytes::Bytes;
use crate::signals::lsfr::LSFR;
use crate::syncword::SyncwordScanXCorr;

// mod block_macro;
mod bitstream_decoder;
mod bytes;
mod clock;
mod fir_interpolator_taps;
mod pll;
mod ringbuffer;
mod signals;
mod syncword;

#[cfg(test)]
mod test_utils;

fn read_complex_value<T: Read>(source_stream: &mut T) -> Option<Complex<f32>> {
    let mut buf_re = [0; 4];
    let mut buf_im = [0; 4];
    source_stream.read_exact(&mut buf_re).ok()?;
    let re = f32::from_ne_bytes(buf_re);

    source_stream.read_exact(&mut buf_im).ok()?;
    let im = f32::from_ne_bytes(buf_im);
    return Some(Complex::new(re, im));
}

fn open_source_file() -> Option<fs::File> {
    let filename = args().nth(1)?;
    let mut file = fs::File::open(filename).ok()?;

    let file_len = file.seek(SeekFrom::End(0)).ok()?;
    if file_len & 3 != 0 {
        return None;
    }

    file.seek(SeekFrom::Start(0)).ok()?;
    Some(file)
}

#[derive(Debug, Default)]
struct PT1<T>
where
    T: Default + Debug + Num + Mul<f32, Output = T>,
{
    current: T,
    alpha: f32,
}

impl<T> PT1<T>
where
    T: Default + Debug + Num + Mul<f32, Output = T> + Copy,
{
    fn from_alpha(alpha: f32) -> Self {
        Self {
            alpha,
            ..Self::default()
        }
    }

    fn tick(&mut self, value: T) -> T {
        self.current = self.current * (1.0 - self.alpha) + value * self.alpha;
        self.current
    }
}

#[derive(Debug)]
struct DiscretePI<F, T> {
    kp: F,
    ki: F,
    integrator: T,
}

impl<F, T> DiscretePI<F, T>
where
    F: num::Num + Copy,
    T: num::Num + Copy,
    F: std::ops::Mul<T, Output = T>,
{
    fn new(kp: F, ki: F) -> Self {
        Self {
            kp,
            ki,
            integrator: T::zero(),
        }
    }

    // fn from_damp_cutoff(d: T, omega: T) -> Self {
    // }

    fn update(&mut self, val: T) -> T {
        self.integrator = self.integrator + self.ki * val;
        self.integrator + self.kp * val
    }
}

struct AGC {
    pt: PT1<f32>,
}

impl AGC {
    fn from_alpha(alpha: f32) -> Self {
        Self {
            pt: PT1 {
                alpha,
                current: 1.0,
            },
        }
    }

    fn tick(&mut self, value: Complex32) -> Complex32 {
        let value_norm_sqr = value.norm_sqr();
        let square_filtered = self.pt.tick(value_norm_sqr);
        value / square_filtered.sqrt()
    }
}

#[derive(Debug)]
struct FIRFilter<T, const N: usize>
where
    T: Num + Debug + Default + Copy,
{
    current_idx: usize,
    fir_consts: [T; N],
    value_hist: [T; N],
}

impl<T, const N: usize> Default for FIRFilter<T, N>
where
    T: Num + Debug + Default + Copy,
{
    fn default() -> Self {
        Self {
            current_idx: 0,
            fir_consts: [T::one(); N],
            value_hist: [T::one(); N],
        }
    }
}

impl<T, const N: usize> FIRFilter<T, N>
where
    T: Num + Debug + Default + Copy,
{
    fn new(factors: [T; N]) -> Self {
        Self {
            fir_consts: factors,
            ..Default::default()
        }
    }

    fn tick(&mut self, value: T) -> T {
        self.value_hist[self.current_idx] = value;
        let mut acc = T::zero();
        for i in 0..N {
            acc = acc + self.fir_consts[i] * self.value_hist[(self.current_idx + N - i) % N];
        }
        self.current_idx += 1;
        self.current_idx %= N;
        acc
    }
}

trait TimingErrorEstimator {
    fn estimate<T, F>(&mut self, buf: &T) -> F
    where
        T: Index<usize, Output = Complex<F>>,
        F: num::Float;
}

#[derive(Default)]
struct GardnerErrorEstimator {}
impl TimingErrorEstimator for GardnerErrorEstimator {
    fn estimate<T, F>(&mut self, buf: &T) -> F
    where
        T: Index<usize, Output = Complex<F>>,
        F: num::Float,
    {
        let a = buf[0];
        let b = buf[1];
        let c = buf[2];

        let real_part: F = b.re * (a.re - c.re);
        let imag_part: F = b.im * (a.im - c.im);
        real_part + imag_part
    }
}

#[derive(Debug)]
struct SymbolSync<ES>
where
    ES: TimingErrorEstimator,
{
    error_est: ES,
    interp_clock: SimpleClock,
    interp_sample_buffer: crate::ringbuffer::RingBuffer<Complex32, 3>,
    input_sample_buffer: crate::ringbuffer::RingBuffer<Complex32, 8>,
    timing_controller: DiscretePI<f32, f32>,
    output_sample: bool,
}

#[allow(dead_code)]
impl<ES> SymbolSync<ES>
where
    ES: TimingErrorEstimator,
{
    fn new(sps: f32, estimator: ES, relative_noise_bandwidth: f32, dampening_factor: f32) -> Self {
        let interp_clock_rate = 4.0 * core::f32::consts::PI / sps;
        let interp_clock = SimpleClock::from_rate(interp_clock_rate);

        let kp = (4.0 * dampening_factor) / (dampening_factor + 1.0 / (4.0 * dampening_factor))
            * relative_noise_bandwidth
            / sps;
        let ki = (4.0) / (dampening_factor + 1.0 / (4.0 * dampening_factor)).powi(2)
            * relative_noise_bandwidth.powi(2)
            / sps.powi(2);
        let timing_controller = DiscretePI::new(kp, ki);

        Self {
            error_est: estimator,
            timing_controller,
            input_sample_buffer: Default::default(),
            interp_sample_buffer: Default::default(),
            interp_clock,
            output_sample: true,
        }
    }

    fn tick(&mut self, sample: Complex32) -> Option<Complex32> {
        self.input_sample_buffer.push(sample);
        let interp_phase_overrun = self.interp_clock.tick()?;

        let interp_value = FIRInterpolator::interpolate(
            &self.input_sample_buffer,
            interp_phase_overrun / (2.0 * PI),
        );
        self.interp_sample_buffer.push(interp_value);

        let timing_error = self.error_est.estimate(&self.interp_sample_buffer);
        let timing_adjust = self.timing_controller.update(timing_error);

        // println!("Overrun: {} + timing_adjust: {}", interp_phase_overrun, timing_adjust);
        if let Some(tick) = self.interp_clock.advance_by(timing_adjust) {
            panic!("Adjust advance increased clock by full cycle!? {}", tick);
        }

        // Return every 2nd interpolated sample
        self.output_sample ^= true;
        if self.output_sample {
            return Some(interp_value);
        }
        None
    }
}

const SYNCWORD: [bool; 32] = [
    false, false, false, true, true, false, true, false, true, true, false, false, true, true,
    true, true, true, true, true, true, true, true, false, false, false, false, false, true, true,
    true, false, true,
];

fn main() -> Result<(), ()> {
    let mut source = BufReader::new(open_source_file().expect("Failed to open source-file!"));
    let mut acg_filter = AGC::from_alpha(0.01);
    let mut roll_avg_filter = FIRFilter::new([Complex32::from(0.2); 5]);
    let mut symbol_sync = SymbolSync::new(5.0, GardnerErrorEstimator {}, 0.0, 0.0);
    let mut syncword_packetizer: SyncwordPacketizer<bool, 32, 10200> =
        SyncwordPacketizer::new(SYNCWORD, 1);
    let bitstream_decoder = BitStreamDecoder::BE;
    let mut bitstream: Vec<u8> = Vec::new();
    let mut sample_idx = -1isize;
    loop {
        sample_idx += 1;
        let sample = match read_complex_value(&mut source) {
            Some(val) => val,
            None => break,
        };

        let acg_sample = acg_filter.tick(sample);
        let roll_avg_out = roll_avg_filter.tick(acg_sample);
        // println!(
        //     "{:?}, norm: {:?} (before: {:?})",
        //     roll_avg_out,
        //     roll_avg_out.norm(),
        //     sample.norm()
        // );

        let symbol_out = match symbol_sync.tick(roll_avg_out) {
            None => continue,
            Some(symbol_out) => symbol_out,
        };
        // println!("{}", symbol_out);
        let val = symbol_out.re >= 0.0;
        let packet = match syncword_packetizer.tick(val) {
            None => continue,
            Some(pkt) => pkt,
        };

        let mut prng_lsfr = LSFR::<u8>::new(0b10101001, 0xFF);
        for bit_idx in 0..packet.len() {
            packet[bit_idx] ^= prng_lsfr.next().unwrap();
        }

        println!("packet @ {:#6}:", sample_idx);
        // println!("    {:?}", &packet);

        let packet_bytes = bitstream_decoder.decode(&packet);
        println!("    {}", Bytes(&packet_bytes));
    }

    std::fs::File::create("bitstream.out")
        .expect("Failed to create bitstream")
        .write_all(&mut bitstream)
        .expect("Failed to write bitstream");

    Ok(())
}
