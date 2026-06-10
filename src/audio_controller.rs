use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

struct AudioController {
    frequency: Arc<Mutex<f32>>, // Shared frequency for the audio tone
    amplitude: Arc<Mutex<f32>>, // Shared amplitude (volume control)
    stream: cpal::Stream,       // Audio stream
}

impl AudioController {
    // Creates a new AudioController with a given frequency and amplitude
    fn new(frequency: f32, amplitude: f32) -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device available");
        let config = device.default_output_config().unwrap();

        // Shared variables to control frequency and amplitude in real-time
        let frequency = Arc::new(Mutex::new(frequency));
        let amplitude = Arc::new(Mutex::new(amplitude));

        let frequency_clone = Arc::clone(&frequency);
        let amplitude_clone = Arc::clone(&amplitude);

        let sample_rate = config.sample_rate().0 as f32;
        let mut sample_clock = 0f32;

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let freq = *frequency_clone.lock().unwrap();
                    let amp = *amplitude_clone.lock().unwrap();
                    for sample in data.iter_mut() {
                        *sample = (sample_clock * freq * 2.0 * PI / sample_rate).sin() * amp;
                        sample_clock = (sample_clock + 1.0) % sample_rate;
                    }
                },
                |err| eprintln!("Error in audio stream: {}", err),
                None, // timeout (added in cpal 0.15)
            )
            .expect("Failed to create audio stream");

        stream.play().expect("Failed to play stream");

        Self { frequency, amplitude, stream }
    }

    // Set the frequency of the audio tone
    fn set_frequency(&self, freq: f32) {
        *self.frequency.lock().unwrap() = freq;
    }

    // Set the amplitude (volume) of the audio tone
    fn set_amplitude(&self, amp: f32) {
        *self.amplitude.lock().unwrap() = amp;
    }

    // Stop the audio stream (optional, as dropping stops it as well)
    fn stop(self) {
        drop(self.stream); // Drop the stream to stop playback
    }
}

pub fn main() {
    let controller = AudioController::new(440.0, 0.5); // 440 Hz (A4), moderate volume

    std::thread::sleep(std::time::Duration::from_secs(2));
    controller.set_frequency(880.0); // Change to 880 Hz (A5)

    std::thread::sleep(std::time::Duration::from_secs(2));
    controller.set_amplitude(0.2); // Lower volume

    std::thread::sleep(std::time::Duration::from_secs(2));
    controller.stop(); // Stop audio
}
