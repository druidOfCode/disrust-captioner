use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, WhisperContext};

pub struct WhisperIntegration {
    context: WhisperContext,
}

impl WhisperIntegration {
    pub fn new(model_path: &str) -> Self {
        let context = WhisperContext::new(model_path).expect("Failed to load whisper model");
        WhisperIntegration { context }
    }

    pub fn transcribe_audio(&self, audio: &[f32]) -> Result<String, Box<dyn std::error::Error>> {
        let mut params = FullParams::new(whisper_rs::SamplingStrategy::default());
        params.set_print_progress(false);
        params.set_print_timestamps(false);

        self.context.full(params, audio)?;
        let num_segments = self.context.full_n_segments()?;

        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = self.context.full_get_segment_text(i) {
                text.push_str(&segment);
                text.push(' ');
            }
        }

        Ok(text.trim().to_string())
    }
}

pub fn initialize_whisper(model_path: &str) -> Arc<Mutex<WhisperIntegration>> {
    Arc::new(Mutex::new(WhisperIntegration::new(model_path)))
}
