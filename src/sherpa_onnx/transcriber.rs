use std::error::Error;
use sherpa_rs::whisper::{WhisperConfig, WhisperRecognizer};

pub struct WhisperTranscriber {
    recognizer: WhisperRecognizer,
}

impl WhisperTranscriber {
    pub fn new(
        encoder_path: &str,
        decoder_path: &str,
        tokens_path: &str,
        language: &str,
        provider: &str, // "cpu" or "cuda"
    ) -> Result<Self, Box<dyn Error>> {
        let config = WhisperConfig {
            encoder: encoder_path.into(),
            decoder: decoder_path.into(),
            tokens: tokens_path.into(),
            language: language.into(),
            provider: Some(provider.into()),
            ..Default::default()
        };

        let recognizer = WhisperRecognizer::new(config)?;

        Ok(Self { recognizer })
    }

    pub fn transcribe(&mut self, sample_rate: u32, samples: Vec<f32>) -> Result<String, Box<dyn Error>> {
        let result = self.recognizer.transcribe(sample_rate, samples);
        Ok(result.text)
    }
}