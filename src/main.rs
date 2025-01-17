use std::error::Error;

use disrust_captioner::transcription::whisper_integration::initialize_whisper;
use disrust_captioner::ui::app::CaptionerApp;
use eframe::NativeOptions;

fn main() -> Result<(), Box<dyn Error>> {
    // 1) We only create a transcription backend now
    let transcription = initialize_whisper("models/ggml-small.en-q8_0.bin");

    // 2) Launch the UI
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Disrust Captioner (No Diarization)",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(CaptionerApp::new(
                cc,
                transcription.clone(),
            )))
        }),
    )?;

    Ok(())
}
