use std::{f32::consts::PI, ops::RangeInclusive};

use egui::Ui;

pub struct Sound {
    pub sources: Vec<Source>,
    pub effects: Vec<Effect>,
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

    pub fn compute_sample(&self, sample_rate: u64, sample_clock: u64) -> f32 {
        let mut sample = 0.0;
        for source in &self.sources {
            sample += match source {
                Source::Wave(wave, freq, amp) => wave.compute_sample(sample_rate, sample_clock, *freq, *amp),
                Source::File => 0.0,
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
    File,
}

impl Source {
    pub fn Sine(freq: f32, amp: f32) -> Self {
        Source::Wave(Wave::Sine, freq, amp)
    }

    pub fn Square(freq: f32, amp: f32) -> Self {
        Source::Wave(Wave::Square, freq, amp)
    }

    pub fn Triangle(freq: f32, amp: f32) -> Self {
        Source::Wave(Wave::Triangle, freq, amp)
    }

    pub fn Sawtooth(freq: f32, amp: f32) -> Self {
        Source::Wave(Wave::Sawtooth, freq, amp)
    }

    pub fn as_type_string(&self) -> &str {
        match self {
            Source::Wave(wave, _, _) => "Wave",
            Source::File => "File",
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, id: &str) {
        match self {
            Source::Wave(ref mut wave, ref mut freq, ref mut amp) => {
                egui::ComboBox::new(format!("Wave Type {}", id), "Wave").selected_text(wave.to_str()).show_ui(ui, |ui| {
                    if ui.selectable_label(*wave == Wave::Sine, "Sine").clicked() {
                        *wave = Wave::Sine;
                    }
                    if ui.selectable_label(*wave == Wave::Square, "Square").clicked() {
                        *wave = Wave::Square;
                    }
                    if ui.selectable_label(*wave == Wave::Triangle, "Triangle").clicked() {
                        *wave = Wave::Triangle;
                    }
                    if ui.selectable_label(*wave == Wave::Sawtooth, "Sawtooth").clicked() {
                        *wave = Wave::Sawtooth;
                    }
                });
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(freq).clamp_range(0.0..=22050.0));
                    ui.add(egui::DragValue::new(amp).clamp_range(0.0..=1.0).speed(0.01));
                });
            }
            Source::File => {}
        }
    }
}

#[derive(PartialEq)]
pub enum Wave {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}
impl Wave {
    pub fn compute_sample(&self, sample_rate: u64, sample_clock: u64, freq: f32, amp: f32) -> f32 {
        let period = sample_rate as f64 / freq as f64;
        let phase = (sample_clock as f64 % period) / period;
        let sample = sample_clock as f64 % period;

        match self {
            Wave::Sine => (sample as f32 * freq * 2.0 * PI / sample_rate as f32).sin() * amp,

            Wave::Square => {
                if phase < 0.5 {
                    return amp;
                }
                return -amp;
            }

            Wave::Triangle => {
                let triangle_wave = if phase < 0.5 { (4.0 * phase - 1.0) as f32 } else { (3.0 - 4.0 * phase) as f32 };
                return triangle_wave * amp;
            }

            Wave::Sawtooth => (2.0 * phase - 1.0) as f32 * amp,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Wave::Sine => "Sine",
            Wave::Square => "Square",
            Wave::Triangle => "Triangle",
            Wave::Sawtooth => "Sawtooth",
        }
    }
}

pub enum Effect {
    None,
}

impl Effect {
    pub fn ui(&self, ui: &mut Ui) {}
}
