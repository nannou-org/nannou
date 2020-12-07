//  Created by Nigel Redmon on 12/18/12.
//  EarLevel Engineering: earlevel.com
//  Copyright 2012 Nigel Redmon
//
//  Ported to Rust by Joshua Batty 2020
//
//  For a complete explanation of the ADSR envelope generator and code,
//  read the series of articles by the author, starting here:
//  http://www.earlevel.com/main/2013/06/01/envelope-generators/
//
//  License:
//
//  This source code is provided as is, without warranty.
//  You may copy and distribute verbatim copies of this document.
//  You may modify and use this source code to create binary code for your own purposes, free or commercial.
//
//  1.01  2016-01-02  njr   added calcCoef to SetTargetRatio functions that were in the ADSR widget but missing in this code
//  1.02  2017-01-04  njr   in calcCoef, checked for rate 0, to support non-IEEE compliant compilers
//  1.03  2020-04-08  njr   changed float to double; large target ratio and rate resulted in exp returning 1 in calcCoef

use dasp::signal::Signal;
use ringbuf::Consumer;

#[derive(Debug, PartialEq)]
enum EnvState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Adsr {
    sample_rate: f64,
    state: EnvState,
    output: f64,
    attack_rate: f64,
    decay_rate: f64,
    release_rate: f64,
    attack_coef: f64,
    decay_coef: f64,
    release_coef: f64,
    sustain_level: f64,
    target_ratio_a: f64,
    target_ratio_dr: f64,
    attack_base: f64,
    decay_base: f64,
    release_base: f64,
    on_off_cons: Consumer<bool>,
    attack_cons: Consumer<f64>,
    decay_cons: Consumer<f64>,
    sustain_cons: Consumer<f64>,
    release_cons: Consumer<f64>,
}

impl Adsr {
    pub fn new(
        sample_rate: f64,
        attack_cons: Consumer<f64>,
        decay_cons: Consumer<f64>,
        sustain_cons: Consumer<f64>,
        release_cons: Consumer<f64>,
        on_off_cons: Consumer<bool>,
    ) -> Self {
        let mut adsr = Adsr {
            sample_rate,
            state: EnvState::Idle,
            output: Default::default(),
            attack_rate: Default::default(),
            decay_rate: Default::default(),
            release_rate: Default::default(),
            attack_coef: Default::default(),
            decay_coef: Default::default(),
            release_coef: Default::default(),
            sustain_level: Default::default(),
            target_ratio_a: Default::default(),
            target_ratio_dr: Default::default(),
            attack_base: Default::default(),
            decay_base: Default::default(),
            release_base: Default::default(),
            attack_cons,
            decay_cons,
            sustain_cons,
            release_cons,
            on_off_cons,
        };

        adsr.reset();
        adsr.target_ratio_a(1.0);
        adsr.target_ratio_dr(0.3);

        adsr
    }

    fn calc_coef(&self, rate: f64, target_ratio: f64) -> f64 {
        if rate <= 0.0 {
            0.0
        } else {
            (-((1.0 + target_ratio) / target_ratio).ln() / rate).exp()
        }
    }

    pub fn attack_rate(&mut self, rate: f64) {
        self.attack_rate = rate;
        self.attack_coef = self.calc_coef(rate, self.target_ratio_a);
        self.attack_base = (1.0 + self.target_ratio_a) * (1.0 - self.attack_coef);
    }

    pub fn decay_rate(&mut self, rate: f64) {
        self.decay_rate = rate;
        self.decay_coef = self.calc_coef(rate, self.target_ratio_dr);
        self.decay_base = (self.sustain_level - self.target_ratio_dr) * (1.0 - self.decay_coef);
    }

    pub fn release_rate(&mut self, rate: f64) {
        self.release_rate = rate;
        self.release_coef = self.calc_coef(rate, self.target_ratio_dr);
        self.release_base = -self.target_ratio_dr * (1.0 - self.release_coef);
    }

    pub fn sustain_level(&mut self, level: f64) {
        self.sustain_level = level;
        self.decay_base = (self.sustain_level - self.target_ratio_dr) * (1.0 - self.decay_coef);
    }

    pub fn target_ratio_a(&mut self, target_ratio: f64) {
        self.target_ratio_a = target_ratio.max(0.000000001); // -180.0 dB
        self.attack_coef = self.calc_coef(self.attack_rate, self.target_ratio_a);
        self.attack_base = (1.0 + self.target_ratio_a) * (1.0 - self.attack_coef);
    }

    pub fn target_ratio_dr(&mut self, target_ratio: f64) {
        self.target_ratio_dr = target_ratio.max(0.000000001); // -180.0 dB
        self.decay_coef = self.calc_coef(self.decay_rate, self.target_ratio_dr);
        self.release_coef = self.calc_coef(self.release_rate, self.target_ratio_dr);
        self.decay_base = (self.sustain_level - self.target_ratio_dr) * (1.0 - self.decay_coef);
        self.release_base = -self.target_ratio_dr * (1.0 - self.release_coef);
    }

    #[inline]
    pub fn process(&mut self) -> f64 {
        if let Some(attack) = self.attack_cons.pop() {
            self.attack_rate(attack * self.sample_rate);
        }
        if let Some(decay) = self.decay_cons.pop() {
            self.decay_rate(decay * self.sample_rate);
        }
        if let Some(sustain) = self.sustain_cons.pop() {
            self.sustain_level(sustain);
        }
        if let Some(release) = self.release_cons.pop() {
            self.release_rate(release * self.sample_rate);
        }
        if let Some(gate) = self.on_off_cons.pop() {
            if gate {
                self.state = EnvState::Attack;
            } else if self.state != EnvState::Idle {
                self.state = EnvState::Release;
            }
        }

        match self.state {
            EnvState::Idle => {
                self.output = 0.0;
            }
            EnvState::Attack => {
                self.output = self.attack_base + self.output * self.attack_coef;
                if self.output >= 1.0 {
                    self.output = 1.0;
                    self.state = EnvState::Decay;
                }
            }
            EnvState::Decay => {
                self.output = self.decay_base + self.output * self.decay_coef;
                if self.output <= self.sustain_level {
                    self.state = EnvState::Sustain;
                }
            }
            EnvState::Sustain => {
                self.output = self.sustain_level;
            }
            EnvState::Release => {
                self.output = self.release_base + self.output * self.release_coef;
                if self.output <= 0.0 {
                    self.output = 0.0;
                    self.state = EnvState::Idle;
                }
            }
        }
        self.output
    }

    pub fn _gate(&mut self, on: bool) {
        if on {
            self.state = EnvState::Attack;
        } else if self.state != EnvState::Idle {
            self.state = EnvState::Release;
        }
    }

    pub fn reset(&mut self) {
        self.state = EnvState::Idle;
        self.output = 0.0;
    }
}

impl Signal for Adsr {
    type Frame = f64;
    fn next(&mut self) -> Self::Frame {
        self.process()
    }
}
