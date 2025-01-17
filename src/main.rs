use std::sync::{Arc, Mutex};

mod audio;
mod diarization;
mod transcription;
mod ui;
mod config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let speaker_manager = Arc::new(Mutex::new(
        diarization::speaker_manager::SpeakerManager::new()
    ));
    
    let diarization = diarization::pyannote::initialize_pyannote(
        "models/segmentation-3.0.onnx",
        16000, // Standard sample rate for most speech models
    );
    
    let transcription = transcription::whisper_integration::initialize_whisper(
        "models/whisper-ggml-base.bin",
    );

    // Start the audio capture in a separate thread
    let audio_device = audio::loopback_capture::initialize_audio_device(None)?;
    let speaker_manager_clone = Arc::clone(&speaker_manager);
    let _stream = audio::loopback_capture::start_audio_capture(
        audio_device,
        Arc::clone(&diarization),
        Arc::clone(&transcription),
        speaker_manager_clone,
        |speaker, text| {
            // Handle new transcription
            println!("{}: {}", speaker, text);
        },
    )?;

    // Pass a None or Some(device) here after user selection
    let selected_device: Option<cpal::Device> = None; // <- from the UI

    // Launch UI
    ui::app::start_ui(diarization, transcription, selected_device);
    Ok(())
}
