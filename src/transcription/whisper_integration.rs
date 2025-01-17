use std::error::Error;
use std::sync::{Arc, Mutex};
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperError,
    WhisperState,
};

// Custom error type for &str errors
struct SimpleError(String);

impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.to_string())
    }
}

pub trait TranscriptionBackend: Send + Sync {
    fn transcribe_audio(&mut self, audio: &[f32]) -> Result<String, Box<dyn Error + Send>>;
}

pub struct WhisperIntegration {
    context: WhisperContext,
    state: WhisperState,
}

impl WhisperIntegration {
    pub fn new(model_path: &str) -> Self {
        let context =
            WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
                .expect("Failed to load whisper model");
        let state = context.create_state().expect("Failed to create state");

        WhisperIntegration { context, state }
    }
}

impl TranscriptionBackend for WhisperIntegration {
    fn transcribe_audio(&mut self, audio: &[f32]) -> Result<String, Box<dyn Error + Send>> {
        if audio.is_empty() {
            return Err(Box::new(WhisperError::InitError) as Box<dyn Error + Send>);
        }

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        self.state
            .full(params, audio)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

        let num_segments = self
            .state
            .full_n_segments()
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

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
