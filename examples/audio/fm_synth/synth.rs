use crate::adsr::Adsr;
use crate::biquad::{Biquad, FilterType};
use dasp::signal::{self as signal, Signal};
use ringbuf::{Consumer, Producer, RingBuffer};

pub struct Filter {
    pub cutoff: f32,
    pub resonance: f32,
    pub peak_gain: f32,
    pub filter_type: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pitch {
    pub ratio: f32,
    pub ratio_offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Operator {
    pub pitch: Pitch,
    pub env: Envelope,
    pub amp: f32,
}

pub struct Producers {
    pub mod_amp: Producer<f64>,
    pub carrier_amp: Producer<f64>,

    pub cutoff: Producer<f64>,
    pub resonance: Producer<f64>,
    pub peak_gain: Producer<f64>,
    pub filter_type: Producer<FilterType>,

    pub mod_hz: Producer<f64>,
    pub carrier_hz: Producer<f64>,

    pub mod_env_on_off: Producer<bool>,
    pub mod_attack: Producer<f64>,
    pub mod_decay: Producer<f64>,
    pub mod_sustain: Producer<f64>,
    pub mod_release: Producer<f64>,

    pub carrier_env_on_off: Producer<bool>,
    pub carrier_attack: Producer<f64>,
    pub carrier_decay: Producer<f64>,
    pub carrier_sustain: Producer<f64>,
    pub carrier_release: Producer<f64>,
}

pub struct Synth {
    pub producers: Producers,
}

impl Synth {
    pub fn new(
        sample_rate: f64,
        master_frequency: f32,
        op1: &Operator,
        op2: &Operator,
        filter: &Filter,
    ) -> (Self, Box<dyn Signal<Frame = f64> + Send>) {
        let mod_env_on_off = RingBuffer::<bool>::new(1);
        let (mod_env_on_off_producer, mod_env_on_off_cons) = mod_env_on_off.split();

        let mod_attack = RingBuffer::<f64>::new(1);
        let (mut mod_attack_producer, mod_attack_cons) = mod_attack.split();
        mod_attack_producer.push(op1.env.attack as f64).unwrap();

        let mod_decay = RingBuffer::<f64>::new(1);
        let (mut mod_decay_producer, mod_decay_cons) = mod_decay.split();
        mod_decay_producer.push(op1.env.decay as f64).unwrap();

        let mod_sustain = RingBuffer::<f64>::new(1);
        let (mut mod_sustain_producer, mod_sustain_cons) = mod_sustain.split();
        mod_sustain_producer.push(op1.env.sustain as f64).unwrap();

        let mod_release = RingBuffer::<f64>::new(1);
        let (mut mod_release_producer, mod_release_cons) = mod_release.split();
        mod_release_producer.push(op1.env.release as f64).unwrap();

        let op1_frequency = crate::calculate_operator_frequency(master_frequency, &op1);
        let mod_hz = RingBuffer::<f64>::new(1);
        let (mut mod_hz_producer, mod_hz_cons) = mod_hz.split();
        mod_hz_producer.push(op1_frequency as f64).unwrap();

        let mod_amp = RingBuffer::<f64>::new(1);
        let (mut mod_amp_producer, mod_amp_cons) = mod_amp.split();
        mod_amp_producer.push(op1.amp as f64).unwrap();

        let modulator_hz_signal = Param::new(op1_frequency as f64, mod_hz_cons);
        let modulator_amp_signal = Param::new(op1.amp as f64, mod_amp_cons);

        let modulator_envelope = Adsr::new(
            sample_rate,
            mod_attack_cons,
            mod_decay_cons,
            mod_sustain_cons,
            mod_release_cons,
            mod_env_on_off_cons,
        );

        let modulator = signal::rate(sample_rate)
            .hz(modulator_hz_signal)
            .sine()
            .mul_amp(modulator_envelope)
            .mul_amp(modulator_amp_signal);

        let carrier_env_on_off = RingBuffer::<bool>::new(1);
        let (carrier_env_on_off_producer, carrier_env_on_off_cons) = carrier_env_on_off.split();

        let carrier_attack = RingBuffer::<f64>::new(1);
        let (mut carrier_attack_producer, carrier_attack_cons) = carrier_attack.split();
        carrier_attack_producer.push(op2.env.attack as f64).unwrap();

        let carrier_decay = RingBuffer::<f64>::new(1);
        let (mut carrier_decay_producer, carrier_decay_cons) = carrier_decay.split();
        carrier_decay_producer.push(op2.env.decay as f64).unwrap();

        let carrier_sustain = RingBuffer::<f64>::new(1);
        let (mut carrier_sustain_producer, carrier_sustain_cons) = carrier_sustain.split();
        carrier_sustain_producer
            .push(op2.env.sustain as f64)
            .unwrap();

        let carrier_release = RingBuffer::<f64>::new(1);
        let (mut carrier_release_producer, carrier_release_cons) = carrier_release.split();
        carrier_release_producer
            .push(op2.env.release as f64)
            .unwrap();

        let carrier_envelope = Adsr::new(
            sample_rate,
            carrier_attack_cons,
            carrier_decay_cons,
            carrier_sustain_cons,
            carrier_release_cons,
            carrier_env_on_off_cons,
        );

        let op2_frequency = crate::calculate_operator_frequency(master_frequency, &op2);
        let carrier_hz = RingBuffer::<f64>::new(1);
        let (mut carrier_hz_producer, carrier_hz_cons) = carrier_hz.split();
        carrier_hz_producer.push(op2_frequency as f64).unwrap();

        let carrier_amp = RingBuffer::<f64>::new(1);
        let (mut carrier_amp_producer, carrier_amp_cons) = carrier_amp.split();
        carrier_amp_producer.push(op2.amp as f64).unwrap();

        let carrier_hz_signal = Param::new(op2_frequency as f64, carrier_hz_cons);
        let carrier_amp_signal = Param::new(op2.amp as f64, carrier_amp_cons);

        let final_hz_signal = carrier_hz_signal.add_amp(modulator);
        let carrier = signal::rate(sample_rate)
            .hz(final_hz_signal)
            .sine()
            .mul_amp(carrier_amp_signal)
            .mul_amp(carrier_envelope);

        let filter_type = RingBuffer::<FilterType>::new(1);
        let (mut filter_type_producer, filter_type_cons) = filter_type.split();
        filter_type_producer.push(FilterType::Lowpass).unwrap();

        let cutoff = RingBuffer::<f64>::new(1);
        let (mut cutoff_producer, cutoff_cons) = cutoff.split();
        cutoff_producer.push(filter.cutoff as f64).unwrap();

        let resonance = RingBuffer::<f64>::new(1);
        let (mut resonance_producer, resonance_cons) = resonance.split();
        resonance_producer.push(filter.resonance as f64).unwrap();

        let peak_gain = RingBuffer::<f64>::new(1);
        let (mut peak_gain_producer, peak_gain_cons) = peak_gain.split();
        peak_gain_producer.push(filter.peak_gain as f64).unwrap();

        let filter_dsp = Biquad::new(
            carrier,
            sample_rate,
            filter_type_cons,
            cutoff_cons,
            resonance_cons,
            peak_gain_cons,
        );

        let fm_synth_signal = Box::new(filter_dsp) as Box<dyn Signal<Frame = f64> + Send>;

        let producers = Producers {
            mod_hz: mod_hz_producer,
            mod_amp: mod_amp_producer,
            carrier_hz: carrier_hz_producer,
            carrier_amp: carrier_amp_producer,
            cutoff: cutoff_producer,
            resonance: resonance_producer,
            peak_gain: peak_gain_producer,
            filter_type: filter_type_producer,
            mod_env_on_off: mod_env_on_off_producer,
            mod_attack: mod_attack_producer,
            mod_decay: mod_decay_producer,
            mod_sustain: mod_sustain_producer,
            mod_release: mod_release_producer,
            carrier_env_on_off: carrier_env_on_off_producer,
            carrier_attack: carrier_attack_producer,
            carrier_decay: carrier_decay_producer,
            carrier_sustain: carrier_sustain_producer,
            carrier_release: carrier_release_producer,
        };

        (Synth { producers }, fm_synth_signal)
    }
}

struct Param<T> {
    last: T,
    consumer: Consumer<T>,
}

impl<T> Param<T> {
    pub fn new(init_value: T, consumer: Consumer<T>) -> Self {
        let last = init_value;
        Self { consumer, last }
    }
}

impl<T> Signal for Param<T>
where
    T: dasp::Frame,
{
    type Frame = T;
    fn next(&mut self) -> Self::Frame {
        match self.consumer.pop() {
            None => self.last,
            Some(v) => {
                self.last = v;
                v
            }
        }
    }
}

pub const CARRIER_RATIOS: &[f32] = &[
    0.25, 0.5, 0.75, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
    15.0, 16.0,
];
pub const MODULATOR_RATIOS: &[f32] = &[
    0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5, 2.75, 3.0, 3.25, 3.5, 3.75, 4.0, 4.25,
    4.5, 4.75, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0, 8.5, 9.0, 9.5, 10.0, 11.0, 12.0, 13.0, 14.0,
    15.0, 16.0,
];
