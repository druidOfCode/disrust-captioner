// src/main.rs

use std::error::Error;
use std::sync::{Arc, Mutex};

use disrust_captioner::diarization::pyannote::initialize_pyannote;
use disrust_captioner::diarization::speaker_manager::SpeakerManager;
use disrust_captioner::transcription::whisper_integration::initialize_whisper;
use disrust_captioner::ui::app::CaptionerApp;
use eframe::NativeOptions;

fn main() -> Result<(), Box<dyn Error>> {
    // 1) Create SpeakerManager, Diarization, and Transcription
    let speaker_manager = Arc::new(Mutex::new(SpeakerManager::new()));
    let diarization = initialize_pyannote("models/segmentation-3.0.onnx", 16000);
    let transcription = initialize_whisper("models/ggml-large-v3.bin");

    // 2) Launch the eframe UI
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Disrust Captioner",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(CaptionerApp::new(
                cc,
                speaker_manager.clone(),
                diarization.clone(),
                transcription.clone(),
            )))
        }),
    )?;

    Ok(())
}
