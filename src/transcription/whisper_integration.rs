use whisper_rs::Whisper;
use std::sync::{Arc, Mutex};

pub struct WhisperIntegration {
    whisper: Whisper,
}

impl WhisperIntegration {
    pub fn new(model_path: &str) -> Self {
        let whisper = Whisper::new(model_path).expect("Failed to initialize Whisper");
        WhisperIntegration { whisper }
    }

    pub fn transcribe_audio(&self, audio: &[f32]) -> String {
        self.whisper.transcribe(audio).expect("Failed to transcribe audio")
    }
}

pub fn initialize_whisper(model_path: &str) -> Arc<Mutex<WhisperIntegration>> {
    Arc::new(Mutex::new(WhisperIntegration::new(model_path)))
}
