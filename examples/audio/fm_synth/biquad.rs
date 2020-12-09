//  Biquad code from EarLevel
//
//  Created by Nigel Redmon on 11/24/12
//  EarLevel Engineering: earlevel.com
//  Copyright 2012 Nigel Redmon
//
//  Ported to Rust by Joshua Batty 2020
//
//  For a complete explanation of the Biquad code:
//  http://www.earlevel.com/main/2012/11/26/biquad-c-source-code/
//
//  License:
//
//  This source code is provided as is, without warranty.
//  You may copy and distribute verbatim copies of this document.
//  You may modify and use this source code to create binary code
//  for your own purposes, free or commercial.

use dasp::signal::Signal;
use ringbuf::Consumer;

#[derive(Debug)]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
    Peak,
    Lowshelf,
    Highshelf,
}

pub struct Biquad<S>
where
    S: Signal<Frame = f64>,
{
    signal: S,
    sample_rate: f64,
    filter_type: FilterType,
    a0: f64,
    a1: f64,
    a2: f64,
    b1: f64,
    b2: f64,
    fc: f64,
    q: f64,
    peak_gain: f64,
    z1: f64,
    z2: f64,
    filter_type_cons: Consumer<FilterType>,
    cutoff_cons: Consumer<f64>,
    resonance_cons: Consumer<f64>,
    peak_gain_cons: Consumer<f64>,
}

impl<S> Biquad<S>
where
    S: Signal<Frame = f64>,
{
    pub fn new(
        signal: S,
        sample_rate: f64,
        filter_type_cons: Consumer<FilterType>,
        cutoff_cons: Consumer<f64>,
        resonance_cons: Consumer<f64>,
        peak_gain_cons: Consumer<f64>,
    ) -> Self {
        let mut biquad = Biquad {
            signal,
            sample_rate,
            filter_type: FilterType::Lowpass,
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            b1: 0.0,
            b2: 0.0,
            fc: 0.0,
            q: 0.707,
            peak_gain: 0.0,
            z1: 0.0,
            z2: 0.0,
            filter_type_cons,
            cutoff_cons,
            resonance_cons,
            peak_gain_cons,
        };
        biquad.calc_biquad();
        biquad
    }

    pub fn filter_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
        self.calc_biquad();
    }

    pub fn resonance(&mut self, q: f64) {
        self.q = q;
        self.calc_biquad();
    }

    pub fn cutoff(&mut self, fc: f64) {
        self.fc = fc / self.sample_rate;
        self.calc_biquad();
    }

    pub fn peak_gain(&mut self, peak_gain_db: f64) {
        self.peak_gain = peak_gain_db;
        self.calc_biquad();
    }

    pub fn _set_biquad(&mut self, filter_type: FilterType, fc: f64, q: f64, peak_gain_db: f64) {
        self.filter_type = filter_type;
        self.q = q;
        self.fc = fc;
        self.peak_gain(peak_gain_db);
    }

    fn calc_biquad(&mut self) {
        let v = 10.0_f64.powf(self.peak_gain.abs() / 20.0);
        let k = std::f64::consts::PI * self.fc;
        match self.filter_type {
            FilterType::Lowpass => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = k * k * norm;
                self.a1 = 2.0 * self.a0;
                self.a2 = self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            FilterType::Highpass => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = 1.0 * norm;
                self.a1 = -2.0 * self.a0;
                self.a2 = self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            FilterType::Bandpass => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = k / self.q * norm;
                self.a1 = 0.0;
                self.a2 = -self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            FilterType::Notch => {
                let norm = 1.0 / (1.0 + k / self.q + k * k);
                self.a0 = (1.0 + k * k) * norm;
                self.a1 = 2.0 * (k * k - 1.0) * norm;
                self.a2 = self.a0;
                self.b1 = self.a1;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
            FilterType::Peak => {
                if self.peak_gain >= 0.0 {
                    // boost
                    let norm = 1.0 / (1.0 + 1.0 / self.q * k + k * k);
                    self.a0 = (1.0 + v / self.q * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - v / self.q * k + k * k) * norm;
                    self.b1 = self.a1;
                    self.b2 = (1.0 - 1.0 / self.q * k + k * k) * norm;
                } else {
                    // cut
                    let norm = 1.0 / (1.0 + v / self.q * k + k * k);
                    self.a0 = (1.0 + 1.0 / self.q * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - 1.0 / self.q * k + k * k) * norm;
                    self.b1 = self.a1;
                    self.b2 = (1.0 - v / self.q * k + k * k) * norm;
                }
            }
            FilterType::Lowshelf => {
                if self.peak_gain >= 0.0 {
                    // boost
                    let norm = 1.0 / (1.0 + 2.0_f64.sqrt() * k + k * k);
                    self.a0 = (1.0 + (2.0_f64 * v).sqrt() * k + v * k * k) * norm;
                    self.a1 = 2.0 * (v * k * k - 1.0) * norm;
                    self.a2 = (1.0 - (2.0_f64 * v).sqrt() * k + v * k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - 2.0_f64.sqrt() * k + k * k) * norm;
                } else {
                    // cut
                    let norm = 1.0 / (1.0 + (2.0_f64 * v).sqrt() * k + v * k * k);
                    self.a0 = (1.0 + 2.0_f64.sqrt() * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - 2.0_f64.sqrt() * k + k * k) * norm;
                    self.b1 = 2.0 * (v * k * k - 1.0) * norm;
                    self.b2 = (1.0 - (2.0_f64 * v).sqrt() * k + v * k * k) * norm;
                }
            }
            FilterType::Highshelf => {
                if self.peak_gain >= 0.0 {
                    // boost
                    let norm = 1.0 / (1.0 + 2.0_f64.sqrt() * k + k * k);
                    self.a0 = (v + (2.0_f64 * v).sqrt() * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - v) * norm;
                    self.a2 = (v - (2.0_f64 * v).sqrt() * k + k * k) * norm;
                    self.b1 = 2.0 * (k * k - 1.0) * norm;
                    self.b2 = (1.0 - 2.0_f64.sqrt() * k + k * k) * norm;
                } else {
                    // cut
                    let norm = 1.0 / (v + (2.0_f64 * v).sqrt() * k + k * k);
                    self.a0 = (1.0 + 2.0_f64.sqrt() * k + k * k) * norm;
                    self.a1 = 2.0 * (k * k - 1.0) * norm;
                    self.a2 = (1.0 - 2.0_f64.sqrt() * k + k * k) * norm;
                    self.b1 = 2.0 * (k * k - v) * norm;
                    self.b2 = (v - (2.0_f64 * v).sqrt() * k + k * k) * norm;
                }
            }
        }
    }

    pub fn process(&mut self) -> f64 {
        if let Some(filter_type) = self.filter_type_cons.pop() {
            self.filter_type(filter_type);
        }
        if let Some(fc) = self.cutoff_cons.pop() {
            self.cutoff(fc);
        }
        if let Some(q) = self.resonance_cons.pop() {
            self.resonance(q);
        }
        if let Some(gain) = self.peak_gain_cons.pop() {
            self.peak_gain(gain);
        }

        let input: f64 = self.signal.next();

        let out = input * self.a0 + self.z1;
        self.z1 = input * self.a1 + self.z2 - self.b1 * out;
        self.z2 = input * self.a2 - self.b2 * out;
        out
    }
}

impl<S> Signal for Biquad<S>
where
    S: Signal<Frame = f64>,
{
    type Frame = S::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.process()
    }
}
