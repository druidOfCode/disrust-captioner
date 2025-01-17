use pyannote_rs::{EmbeddingExtractor, Segment};
use std::sync::{Arc, Mutex};

pub trait DiarizationBackend: Send + Sync {
    fn segment_audio(&self, audio: &[i16]) -> Result<Vec<Segment>, Box<dyn std::error::Error>>;
    fn embed_speaker(&mut self, audio: &[i16]) -> Vec<f32>;
}

pub struct PyannoteIntegration {
    extractor: EmbeddingExtractor,
}

impl PyannoteIntegration {
    pub fn new(segmentation_model: &str, _sample_rate: u32) -> Self {
        let extractor = EmbeddingExtractor::new(segmentation_model)
            .expect("Failed to load extractor");
        PyannoteIntegration { extractor }
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

    fn embed_speaker(&mut self, audio: &[i16]) -> Vec<f32> {
        self.extractor.compute(audio)
            .map(|iter| iter.collect())
            .unwrap_or_else(|_| Vec::new())
    }
}

pub fn initialize_pyannote(segmentation_model: &str, sample_rate: u32) -> Arc<Mutex<dyn DiarizationBackend + Send + Sync>> {
    Arc::new(Mutex::new(PyannoteIntegration::new(segmentation_model, sample_rate)))
}
