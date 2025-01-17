use std::error::Error;
use std::sync::{Arc, Mutex};

use disrust_captioner::diarization::speaker_manager::SpeakerManager;
use disrust_captioner::transcription::whisper_integration::initialize_whisper;
use disrust_captioner::ui::app::CaptionerApp;
use eframe::NativeOptions;

fn main() -> Result<(), Box<dyn Error>> {
    // 1) Initialize components
    let transcription = initialize_whisper("models/ggml-small.en-q8_0.bin");
    let speaker_manager = Arc::new(Mutex::new(SpeakerManager::new()));

    // 2) UI Launch
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Disrust Captioner (With Diarization)",
        native_options,
        Box::new(move |cc| {
            let app = CaptionerApp::new(cc, transcription.clone(), speaker_manager.clone());
            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}