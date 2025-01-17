use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use pyannote_rs::{Pyannote};
use whisper_rs::Whisper;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod audio;
mod diarization;
mod transcription;
mod ui;
mod config;

fn main() {
    // Initialize the application
    let audio_device = audio::loopback_capture::initialize_audio_device().expect("Failed to initialize audio device");
    let pyannote = Pyannote::new("models/segmentation-3.0.onnx", "models/wespeaker_en_voxceleb_CAM++.onnx").expect("Failed to initialize pyannote");
    let whisper = Whisper::new("models/whisper-ggml-base.bin").expect("Failed to initialize Whisper");

    let diarization = Arc::new(Mutex::new(diarization::pyannote::PyannoteIntegration::new("models/segmentation-3.0.onnx", "models/wespeaker_en_voxceleb_CAM++.onnx")));
    let transcription = Arc::new(Mutex::new(transcription::whisper_integration::WhisperIntegration::new("models/whisper-ggml-base.bin")));

    // Start audio capture
    let stream = audio::loopback_capture::start_audio_capture(audio_device, diarization.clone(), transcription.clone()).expect("Failed to start audio capture");

    // Start the UI
    ui::start_ui(diarization, transcription);

    // Keep the application running
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
