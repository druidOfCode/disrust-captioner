use std::sync::{Arc, Mutex};
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy, WhisperState};

pub trait TranscriptionBackend: Send + Sync {
    fn transcribe_audio(&mut self, audio: &[f32], sample_rate: u32) -> Result<String, Box<dyn std::error::Error>>;
}

pub struct WhisperIntegration {
    context: WhisperContext,
    state: WhisperState,
}

impl WhisperIntegration {
    pub fn new(model_path: &str) -> Self {
        let context = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default()
        ).expect("Failed to load whisper model");
        let state = context.create_state().expect("Failed to create state");
        
        WhisperIntegration { context, state }
    }
}

impl TranscriptionBackend for WhisperIntegration {
    fn transcribe_audio(&mut self, audio: &[f32], sample_rate: u32) -> Result<String, Box<dyn std::error::Error>> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.to_sample_rate(sample_rate as f32);

        self.state.full(params, audio)?;
        let num_segments = self.state.full_n_segments()?;

        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = self.state.full_get_segment_text(i) {
                text.push_str(&segment);
                text.push(' ');
            }
        }

        Ok(text.trim().to_string())
    }
}

pub fn initialize_whisper(model_path: &str) -> Arc<Mutex<impl TranscriptionBackend>> {
    Arc::new(Mutex::new(WhisperIntegration::new(model_path)))
}
