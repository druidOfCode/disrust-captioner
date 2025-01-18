use std::error::Error;
use std::sync::{Arc, Mutex};

use disrust_captioner::diarization::speaker_manager::SpeakerManager;
use disrust_captioner::sherpa_onnx::SherpaOnnx;
use disrust_captioner::ui::app::CaptionerApp;
use eframe::NativeOptions;

fn main() -> Result<(), Box<dyn Error>> {
    // 1) Initialize components
    // models\sherpa-onnx-whisper-tiny.en\tiny.en-tokens.txt

    let whisper_encoder_path = "models/sherpa-onnx-whisper-tiny/tiny_en-encoder.onnx";
    let whisper_decoder_path = "models/sherpa-onnx-whisper-tiny/tiny_en-decoder.onnx";
    let whisper_tokens_path = "models/sherpa-onnx-whisper-tiny/tiny_en-tokens.txt";
    let segmentation_model_path = "models/sherpa-onnx-pyannote-segmentation-3-0/model.onnx";
    let embedding_model_path = "models/3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx";

     // Create Arc<Mutex<SherpaOnnx>> here
     let sherpa_onnx = Arc::new(Mutex::new(SherpaOnnx::new(
        &whisper_encoder_path,
        &whisper_decoder_path,
        &whisper_tokens_path,
        &segmentation_model_path,
        &embedding_model_path,
    )?));
    let speaker_manager = Arc::new(Mutex::new(SpeakerManager::new()));

    // 2) UI Launch
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Disrust Captioner (With Diarization)",
        native_options,
        Box::new(move |cc| {
            // Pass Arc<Mutex<SherpaOnnx>> directly
            let app = CaptionerApp::new(cc, sherpa_onnx.clone(), speaker_manager.clone());
            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}