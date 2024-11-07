use std::f32::consts::PI;

pub struct Sound {
    sources: Vec<Source>,
    effects: Vec<Effect>,
}

impl Sound {
    pub fn new() -> Self {
        Self { sources: vec![], effects: vec![] }
    }

    pub fn push_source(&mut self, source: Source) {
        self.sources.push(source);
    }

    pub fn push_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    pub fn compute_sample(&self, sample_rate: f32, sample_clock: f32) -> f32 {
        let mut sample = 0.0;
        for source in &self.sources {
            sample += match source {
                Source::Wave(wave, freq, amp) => wave.compute_sample(sample_rate, sample_clock, *freq, *amp),
                Source::Recording => 0.0,
            }
        }
        return sample;
    }
}

pub struct SoundInstance {
    pub sid: usize,
    pub pan: f32,
    pub volume: f32,
}

impl SoundInstance {
    pub fn new(sid: usize, pan: f32, volume: f32) -> Self {
        Self { sid, pan, volume }
    }
}

pub enum Source {
    Wave(Wave, f32, f32),
    Recording,
}

impl Source {
    pub fn Sine(freq: f32, amp: f32) -> Self {
        Source::Wave(Wave::Sine, freq, amp)
    }
}

pub enum Wave {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}
impl Wave {
    pub fn compute_sample(&self, sample_rate: f32, sample_clock: f32, freq: f32, amp: f32) -> f32 {
        match self {
            Wave::Sine => (sample_clock * freq * 2.0 * PI / sample_rate).sin() * amp,
            Wave::Square => todo!(),
            Wave::Triangle => todo!(),
            Wave::Sawtooth => todo!(),
        }
    }
}

pub enum Effect {
    None,
}
