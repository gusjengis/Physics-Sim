use crate::sound::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct AudioController {
    host: cpal::Host,
    device: cpal::Device,
    stream: cpal::Stream, // Audio stream
    pub sounds: Arc<Mutex<Vec<Sound>>>,
    pub active_sounds: Arc<Mutex<Vec<SoundInstance>>>,
}

impl AudioController {
    // Creates a new AudioController with a given frequency and amplitude
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device available");
        let config = device.default_output_config().unwrap();

        let sample_rate = config.sample_rate().0 as f32;
        let mut sample_clock = 0f32;

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                        // for i in 0..active_sounds {
                        //     let freq = frequency_clone.lock().unwrap()[i];
                        //     let amp = amplitude_clone.lock().unwrap()[i];
                        //     *sample += (sample_clock * freq * 2.0 * PI / sample_rate).sin() * amp;
                        // }
                        // sample_clock = (sample_clock + 1.0) % sample_rate;
                    }
                },
                |err| eprintln!("Error in audio stream: {}", err),
                None,
            )
            .expect("Failed to create audio stream");

        Self {
            host,
            device,
            stream,
            sounds: Arc::new(Mutex::new(vec![])),
            active_sounds: Arc::new(Mutex::new(vec![])),
        }
    }

    // fn build_stream(&mut self) {
    //     let config = self.device.default_output_config().unwrap();

    //     let sample_rate = config.sample_rate().0 as f32;
    //     let mut sample_clock = 0f32;

    //     let sounds_clone = Arc::clone(&self.sounds);
    //     let active_sounds_clone = Arc::clone(&self.active_sounds);

    //     self.stream = self
    //         .device
    //         .build_output_stream(
    //             &config.into(),
    //             move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    //                 for sample in data.iter_mut() {
    //                     *sample = 0.0;
    //                     let active_sounds = active_sounds_clone.lock().unwrap();
    //                     let sounds = sounds_clone.lock().unwrap();
    //                     for sound in active_sounds.iter() {
    //                         *sample += sounds[sound.sid].compute_sample(sample_rate, sample_clock);
    //                     }
    //                     sample_clock = (sample_clock + 1.0) % sample_rate;
    //                 }
    //             },
    //             |err| eprintln!("Error in audio stream: {}", err),
    //             None,
    //         )
    //         .expect("Failed to create audio stream");
    // }
    fn build_stream(&mut self) {
        let config = self.device.default_output_config().unwrap();
        let sample_rate = config.sample_rate().0 as f32;
        let mut sample_clock = 0f32;

        let sounds_clone = Arc::clone(&self.sounds);
        let active_sounds_clone = Arc::clone(&self.active_sounds);

        self.stream = self
            .device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for frame in data.chunks_mut(2) {
                        // Each frame has 2 samples: [left, right]
                        let mut left_sample = 0.0;
                        let mut right_sample = 0.0;

                        let active_sounds = active_sounds_clone.lock().unwrap();
                        let sounds = sounds_clone.lock().unwrap();

                        for sound in active_sounds.iter() {
                            let sample = sounds[sound.sid].compute_sample(sample_rate, sample_clock);

                            // Apply panning: sound.pan ranges from -1.0 (full left) to 1.0 (full right)
                            let left_gain = (1.0 - sound.pan) * sound.volume;
                            let right_gain = (1.0 + sound.pan) * sound.volume;

                            left_sample += sample * left_gain;
                            right_sample += sample * right_gain;
                        }

                        frame[0] = left_sample; // Left channel
                        frame[1] = right_sample; // Right channel

                        sample_clock = (sample_clock + 1.0) % sample_rate;
                    }
                },
                |err| eprintln!("Error in audio stream: {}", err),
                None,
            )
            .expect("Failed to create audio stream");
    }

    fn play(&mut self) {
        self.stream.play().expect("Failed to play stream");
    }

    fn push_sound(&self, sound: Sound) {
        self.sounds.lock().unwrap().push(sound);
    }

    fn play_sound(&self, sid: usize, pan: f32, volume: f32) {
        self.active_sounds.lock().unwrap().push(SoundInstance::new(sid, pan, volume));
    }

    // Stop the audio stream (optional, as dropping stops it as well)
    fn stop(self) {
        drop(self.stream); // Drop the stream to stop playback
    }
}

pub fn main() {
    // Spawn a new thread for the AudioController
    let controller_thread = thread::spawn(move || {
        let mut ac = AudioController::new();

        let mut ringtone_a = Sound::new();
        ringtone_a.push_source(Source::Sine(440.0, 1.0));
        // ringtone_a.push_source(Source::Sine(480.0, 1.0));

        let mut ringtone_b = Sound::new();
        // ringtone_b.push_source(Source::Sine(440.0, 1.0));
        ringtone_b.push_source(Source::Sine(480.0, 1.0));
        ac.push_sound(ringtone_a);
        ac.push_sound(ringtone_b);
        ac.play_sound(0, -1.0, 1.0);
        ac.play_sound(1, 1.0, 1.0);
        ac.build_stream();
        ac.play();
        std::thread::sleep(std::time::Duration::from_secs(100));

        //ac.stop();
    });
}
