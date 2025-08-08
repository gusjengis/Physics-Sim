// use crate::sound::*;
// use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
// use cpal::SupportedBufferSize;
// use std::f32::consts::PI;
// use std::sync::{Arc, Mutex};
// use std::thread;

// pub struct AudioController {
//     host: cpal::Host,
//     device: cpal::Device,
//     stream: cpal::Stream, // Audio stream
//     pub ar: Arc<Mutex<AudioRecord>>,
//     pub sounds: Arc<Mutex<Vec<Sound>>>,
//     pub active_sounds: Arc<Mutex<Vec<SoundInstance>>>,
// }

// impl AudioController {
//     pub fn new() -> Self {
//         let host = cpal::default_host();
//         let device = host.default_output_device().expect("No output device available");
//         let config = device.default_output_config().unwrap();

//         let sample_rate = config.sample_rate().0 as u64;
//         println!("SR: {}", sample_rate);
//         let mut sample_clock: u64 = 0;

//         let ar = Arc::new(Mutex::new(AudioRecord::new()));
//         let sounds: Arc<Mutex<Vec<Sound>>> = Arc::new(Mutex::new(vec![]));
//         let active_sounds: Arc<Mutex<Vec<SoundInstance>>> = Arc::new(Mutex::new(vec![]));

//         let ar_clone = Arc::clone(&ar);
//         let sounds_clone = Arc::clone(&sounds);
//         let active_sounds_clone = Arc::clone(&active_sounds);

//         let stream = device
//             .build_output_stream(
//                 &config.into(),
//                 move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
//                     let active_sounds = active_sounds_clone.lock().unwrap();
//                     let mut sounds = sounds_clone.lock().unwrap();
//                     let mut ar = ar_clone.lock().unwrap();
//                     let mut left_record = vec![];
//                     let mut right_record = vec![];
//                     for frame in data.chunks_mut(2) {
//                         let mut left_sample = 0.0;
//                         let mut right_sample = 0.0;

//                         for sound in active_sounds.iter() {
//                             let sample = sounds[sound.sid].compute_sample(sample_rate, sample_clock);

//                             let left_gain = (1.0 - sound.pan) * sound.volume;
//                             let right_gain = (1.0 + sound.pan) * sound.volume;

//                             left_sample += sample.0 * left_gain;
//                             right_sample += sample.1 * right_gain;
//                         }

//                         frame[0] = left_sample; // Left channel
//                         frame[1] = right_sample; // Right channel
//                                                  //
//                         left_record.push(left_sample);
//                         right_record.push(right_sample);

//                         sample_clock = (sample_clock + 1);
//                     }
//                     ar.push_buffer(left_record, right_record);
//                 },
//                 |err| eprintln!("Error in audio stream: {}", err),
//                 None,
//             )
//             .expect("Failed to create audio stream");

//         stream.play().expect("Failed to play stream");

//         Self {
//             host,
//             device,
//             stream,
//             sounds,
//             active_sounds,
//             ar,
//         }
//     }

//     fn push_sound(&mut self, sound: Sound) {
//         self.sounds.lock().unwrap().push(sound);
//     }

//     fn play_sound(&mut self, sid: usize, pan: f32, volume: f32, duration: u64) {
//         self.active_sounds.lock().unwrap().push(SoundInstance::new(sid, pan, volume, duration));
//     }

//     fn pause(&mut self) {
//         self.stream.pause().expect("Failed to pause stream");
//     }

//     pub fn ringtone(&mut self) {
//         let mut ringtone_a = Sound::new();
//         ringtone_a.set_name("Left");
//         let mut ringtone_b = Sound::new();
//         ringtone_b.set_name("Right");
//         ringtone_a.push_source(Source::sine(100.0, 0.0));
//         ringtone_a.push_source(Source::square(100.0, 0.0));
//         ringtone_b.push_source(Source::triangle(100.0, 0.0));
//         ringtone_b.push_source(Source::sawtooth(100.0, 0.0));

//         self.push_sound(ringtone_a);
//         self.push_sound(ringtone_b);
//         self.play_sound(0, -1.0, 1.0, 0);
//         self.play_sound(1, 1.0, 1.0, 0);
//     }

//     pub fn new_sound(&self) {
//         self.sounds.lock().unwrap().push(Sound::new());
//     }

//     pub fn play(&mut self, sid: usize) {
//         let mut sounds = self.sounds.lock().unwrap();
//         let mut duration = self.active_sounds.lock().unwrap().push(SoundInstance::new(sid, 0.0, 1.0, sounds[sid].duration()));
//     }
// }

// pub struct AudioRecord {
//     pub samples: usize,
//     pub left: Vec<f32>,
//     pub right: Vec<f32>,
//     start_index: usize,
// }

// impl AudioRecord {
//     fn new() -> Self {
//         let samples = 44100;
//         Self {
//             samples,
//             left: vec![0.0; samples],
//             right: vec![0.0; samples],
//             start_index: 0,
//         }
//     }

//     pub fn push_buffer(&mut self, left: Vec<f32>, right: Vec<f32>) {
//         let len = left.len();

//         if len > self.samples {
//             panic!("Input buffer size exceeds circular buffer capacity");
//         }

//         let end_index = (self.start_index + len) % self.samples;

//         if self.start_index < end_index {
//             self.left[self.start_index..end_index].copy_from_slice(&left);
//             self.right[self.start_index..end_index].copy_from_slice(&right);
//         } else {
//             let split = self.samples - self.start_index;
//             self.left[self.start_index..].copy_from_slice(&left[..split]);
//             self.left[..end_index].copy_from_slice(&left[split..]);
//             self.right[self.start_index..].copy_from_slice(&right[..split]);
//             self.right[..end_index].copy_from_slice(&right[split..]);
//         }

//         self.start_index = end_index;
//     }

//     pub fn get_record(&self) -> (Vec<f32>, Vec<f32>) {
//         let mut left_buffer = Vec::with_capacity(self.samples);
//         let mut right_buffer = Vec::with_capacity(self.samples);

//         if self.start_index == 0 {
//             left_buffer.extend_from_slice(&self.left);
//             right_buffer.extend_from_slice(&self.right);
//         } else {
//             let (left_part, right_part) = self.left.split_at(self.start_index);
//             let (right_left_part, right_right_part) = self.right.split_at(self.start_index);

//             left_buffer.extend_from_slice(right_part);
//             left_buffer.extend_from_slice(left_part);

//             right_buffer.extend_from_slice(right_right_part);
//             right_buffer.extend_from_slice(right_left_part);
//         }

//         (left_buffer, right_buffer)
//     }
// }
