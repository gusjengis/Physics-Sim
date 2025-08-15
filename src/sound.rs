// use async_std::{channel, path::PathBuf};
// use cgmath::num_traits::Float;
// use chrono::{format, DurationRound, Local};
// use claxon::{metadata::StreamInfo, FlacReader};
// use egui::{TextBuffer, Ui};
// use std::{f32::consts::PI, ops::RangeInclusive};

// pub struct Sound {
//     pub name: String,
//     pub duration: u64,
//     pub auto_duration: bool,
//     pub sources: Vec<Source>,
//     pub effects: Vec<Effect>,
// }

// impl Sound {
//     pub fn new() -> Self {
//         Self {
//             name: String::from("New Sound"),
//             duration: 1,
//             auto_duration: true,
//             sources: vec![],
//             effects: vec![],
//         }
//     }

//     pub fn push_source(&mut self, source: Source) {
//         self.sources.push(source);
//     }

//     pub fn push_effect(&mut self, effect: Effect) {
//         self.effects.push(effect);
//     }

//     pub fn new_source(&mut self) {
//         self.sources.push(Source::None);
//     }

//     pub fn new_effect(&mut self) {
//         self.effects.push(Effect::None);
//     }

//     pub fn compute_sample(&mut self, sample_rate: u64, sample_clock: u64) -> (f32, f32) {
//         let mut sample = (0.0, 0.0);
//         for source in &mut self.sources {
//             let source_sample = match source {
//                 Source::None => (0.0, 0.0),
//                 Source::Wave(wave, freq, amp) => {
//                     let wave_sample = wave.compute_sample(sample_rate, sample_clock, *freq, *amp);
//                     (wave_sample, wave_sample)
//                 }
//                 Source::File(..) => source.file_sample(sample_rate, sample_clock),
//             };
//             sample.0 += source_sample.0;
//             sample.1 += source_sample.1;
//         }
//         sample
//     }

//     pub fn name_mut(&mut self) -> &mut String {
//         &mut self.name
//     }

//     pub fn set_name(&mut self, name: &str) {
//         self.name = String::from(name);
//     }
//     pub fn ui(&mut self, ui: &mut Ui, curr_source: i32, curr_effect: i32) {
//         self.sources[curr_source as usize].ui(ui, format!("{}:{}", self.name, curr_source).as_str());
//     }

//     pub fn duration(&self) -> u64 {
//         match self.auto_duration {
//             true => {
//                 let mut max = 0;
//                 for source in self.sources.iter() {
//                     max = match source {
//                         Source::None => 0,
//                         Source::Wave(wave, _, _) => 0,
//                         Source::File(None, ..) => 0,
//                         Source::File(_, buffer, ..) => (buffer.len() as f64 / (44.100)).ceil() as u64,
//                     }
//                     .max(max);
//                 }
//                 max
//             }
//             false => self.duration,
//         }
//     }
// }

// pub struct SoundInstance {
//     pub sid: usize,
//     pub pan: f32,
//     pub volume: f32,
//     pub duration: u64,
// }

// impl SoundInstance {
//     pub fn new(sid: usize, pan: f32, volume: f32, duration: u64) -> Self {
//         Self { sid, pan, volume, duration }
//     }
// }

// pub enum Source {
//     None,
//     Wave(Wave, f32, f32),                                                   // wave type, freqency, amplitude
//     File(Option<PathBuf>, Vec<f32>, usize, usize, f32, Option<StreamInfo>), // path, sample buffer, sample index, start time/sample, amplitude
// }

// impl PartialEq for Source {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Self::Wave(l0, l1, l2), Self::Wave(r0, r1, r2)) => l0 == r0 && l1 == r1 && l2 == r2,
//             (Self::File(l0, l1, l2, l3, l4, l5), Self::File(r0, r1, r2, r3, r4, r5)) => l0 == r0,
//             _ => core::mem::discriminant(self) == core::mem::discriminant(other),
//         }
//     }
// }

// impl Source {
//     pub fn sine(freq: f32, amp: f32) -> Self {
//         Source::Wave(Wave::Sine, freq, amp)
//     }

//     pub fn square(freq: f32, amp: f32) -> Self {
//         Source::Wave(Wave::Square, freq, amp)
//     }

//     pub fn triangle(freq: f32, amp: f32) -> Self {
//         Source::Wave(Wave::Triangle, freq, amp)
//     }

//     pub fn sawtooth(freq: f32, amp: f32) -> Self {
//         Source::Wave(Wave::Sawtooth, freq, amp)
//     }

//     pub fn as_type_string(&self) -> &str {
//         match self {
//             Source::None => "Undefined",
//             Source::Wave(wave, ..) => "Wave",
//             Source::File(..) => "File",
//         }
//     }

//     pub fn ui(&mut self, ui: &mut Ui, id: &str) {
//         egui::ComboBox::new(format!("Source {}", id), "Source").selected_text(self.as_type_string()).show_ui(ui, |ui| {
//             if ui.selectable_label(matches!(self, Source::Wave(..)), "Wave").clicked() {
//                 *self = Source::sine(0.0, 0.0);
//             }
//             if ui.selectable_label(matches!(self, Source::File(..)), "File").clicked() {
//                 *self = Source::File(None, vec![], 0, 0, 1.0, None);
//             }
//         });

//         match self {
//             Source::None => {}
//             Source::Wave(ref mut wave, ref mut freq, ref mut amp) => {
//                 egui::ComboBox::new(format!("Wave Type {}", id), "Wave").selected_text(wave.to_str()).show_ui(ui, |ui| {
//                     if ui.selectable_label(*wave == Wave::Sine, "Sine").clicked() {
//                         *wave = Wave::Sine;
//                     }
//                     if ui.selectable_label(*wave == Wave::Square, "Square").clicked() {
//                         *wave = Wave::Square;
//                     }
//                     if ui.selectable_label(*wave == Wave::Triangle, "Triangle").clicked() {
//                         *wave = Wave::Triangle;
//                     }
//                     if ui.selectable_label(*wave == Wave::Sawtooth, "Sawtooth").clicked() {
//                         *wave = Wave::Sawtooth;
//                     }
//                 });
//                 ui.horizontal(|ui| {
//                     ui.add(egui::DragValue::new(freq).clamp_range(0.0..=22050.0));
//                     ui.add(egui::DragValue::new(amp).clamp_range(0.0..=1.0).speed(0.01));
//                 });
//             }
//             Source::File(path, samples, ref mut s_index, _, ref mut amp, ..) => {
//                 let mut new_file = false;
//                 if ui.button("Select File").clicked() {
//                     if let Some(file_path) = rfd::FileDialog::new().pick_file() {
//                         *path = Some(file_path.into());
//                         new_file = true;
//                     }
//                 }

//                 if let Some(file_path) = path {
//                     ui.label(format!("Selected file: {}", file_path.display()));
//                     ui.add(egui::Slider::new(s_index, 0..=(samples.len() - 1)).text("Playback"));
//                     ui.add(egui::DragValue::new(amp).clamp_range(0.0..=1.0).speed(0.01).prefix("Volume: "));
//                     if new_file {
//                         self.init_source();
//                     }
//                     // Placeholder variables for demonstration
//                 } else {
//                     ui.label("No file selected");
//                 }
//             }
//         }
//     }

//     pub fn init_source(&mut self) -> Result<(), Box<dyn std::error::Error>> {
//         match self {
//             Source::None => Ok(()),
//             Source::Wave(_, _, _) => Ok(()),
//             Source::File(None, ..) => Ok(()),
//             Source::File(Some(path), samples, s_index, start, amp, stream_info) => {
//                 let start_time = Local::now();
//                 let mut reader = FlacReader::open(path)?;
//                 samples.clear();
//                 *stream_info = Some(reader.streaminfo());
//                 let max_loudness = 2.0.powi(reader.streaminfo().bits_per_sample as i32 - 1) - 1.0;
//                 for sample in reader.samples() {
//                     samples.push(sample? as f32 / max_loudness);
//                 }
//                 println!("File Loaded: {}ms", Local::now().timestamp_millis() - start_time.timestamp_millis());
//                 *s_index = 0;
//                 *start = 0;
//                 *amp = 1.0;
//                 Ok(())
//             }
//         }
//     }

//     fn file_sample(&mut self, sample_rate: u64, sample_clock: u64) -> (f32, f32) {
//         match self {
//             Source::None => (0.0, 0.0),
//             Source::Wave(..) => (0.0, 0.0),
//             Source::File(None, ..) => (0.0, 0.0),
//             Source::File(.., None) => (0.0, 0.0),
//             Source::File(_, vec, s_index, _, amp, Some(stream_info)) => {
//                 let channels = stream_info.channels as usize;
//                 let file_sr = stream_info.sample_rate as u64;
//                 let playback_speed = file_sr as f64 / sample_rate as f64;
//                 let target_sample = (*s_index as f64 * playback_speed) as usize;
//                 let mut res = (0.0, 0.0);
//                 if target_sample + channels < vec.len() {
//                     let mut sum = 0.0;
//                     if channels == 2 {
//                         res.0 = *amp * vec[target_sample * channels + 0];
//                         res.1 = *amp * vec[target_sample * channels + 1];
//                     } else {
//                         for i in 0..channels as usize {
//                             sum += vec[target_sample * channels + i];
//                         }

//                         let sample = *amp * sum / (channels as f32);
//                         res = (sample, sample);
//                     }

//                     *s_index += 1;
//                 }
//                 res
//             }
//         }
//     }
// }

// #[derive(PartialEq)]
// pub enum Wave {
//     Sine,
//     Square,
//     Triangle,
//     Sawtooth,
// }

// impl Wave {
//     pub fn compute_sample(&self, sample_rate: u64, sample_clock: u64, freq: f32, amp: f32) -> f32 {
//         let period = sample_rate as f64 / freq as f64;
//         let phase = (sample_clock as f64 % period) / period;
//         let sample = sample_clock as f64 % period;

//         match self {
//             Wave::Sine => (sample as f32 * freq * 2.0 * PI / sample_rate as f32).sin() * amp,

//             Wave::Square => {
//                 if phase < 0.5 {
//                     return amp;
//                 }
//                 return -amp;
//             }

//             Wave::Triangle => {
//                 let triangle_wave = if phase < 0.5 { (4.0 * phase - 1.0) as f32 } else { (3.0 - 4.0 * phase) as f32 };
//                 return triangle_wave * amp;
//             }

//             Wave::Sawtooth => (2.0 * phase - 1.0) as f32 * amp,
//         }
//     }

//     fn to_str(&self) -> &str {
//         match self {
//             Wave::Sine => "Sine",
//             Wave::Square => "Square",
//             Wave::Triangle => "Triangle",
//             Wave::Sawtooth => "Sawtooth",
//         }
//     }
// }

// pub enum Effect {
//     None,
// }

// impl Effect {
//     pub fn ui(&self, ui: &mut Ui) {}
// }
