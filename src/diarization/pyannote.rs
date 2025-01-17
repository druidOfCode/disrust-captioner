use pyannote_rs::{EmbeddingExtractor, Segment};
use std::sync::{Arc, Mutex};

pub trait DiarizationBackend: Send + Sync {
    fn segment_audio(&self, audio: &[i16]) -> Result<Vec<Segment>, Box<dyn std::error::Error>>;
    fn embed_speaker(&self, audio: &[i16]) -> Vec<f32>;
}

pub struct PyannoteIntegration {
    extractor: EmbeddingExtractor,
    sample_rate: u32,
}

impl PyannoteIntegration {
    pub fn new(segmentation_model: &str, sample_rate: u32) -> Self {
        let extractor = EmbeddingExtractor::new(segmentation_model)
            .expect("Failed to load extractor");
        PyannoteIntegration { extractor, sample_rate }
    }
}

impl DiarizationBackend for PyannoteIntegration {
    fn segment_audio(&self, audio: &[i16]) -> Result<Vec<Segment>, Box<dyn std::error::Error>> {
        // For now, return a single segment containing all audio
        Ok(vec![Segment {
            start: 0.0,
            end: audio.len() as f64,
            samples: audio.to_vec() 
        }])
    }

    fn embed_speaker(&self, audio: &[f32]) -> Vec<f32> {
        let i16_data: Vec<i16> = audio.iter().map(|&f| (f * i16::MAX as f32) as i16).collect();
        self.extractor.extract(&i16_data, self.sample_rate)
            .unwrap_or_default()
    }
}

pub fn initialize_pyannote(segmentation_model: &str, sample_rate: u32) -> Arc<Mutex<impl DiarizationBackend>> {
    Arc::new(Mutex::new(PyannoteIntegration::new(segmentation_model, sample_rate)))
}
