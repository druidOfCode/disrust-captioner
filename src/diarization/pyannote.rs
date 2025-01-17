use pyannote_rs::{Pyannote, Segmentation, SpeakerEmbedding};
use std::sync::{Arc, Mutex};

pub struct PyannoteIntegration {
    pyannote: Pyannote,
}

impl PyannoteIntegration {
    pub fn new(segmentation_model: &str, speaker_model: &str) -> Self {
        let pyannote = Pyannote::new(segmentation_model, speaker_model).expect("Failed to initialize pyannote");
        PyannoteIntegration { pyannote }
    }

    pub fn segment_audio(&self, audio: &[f32]) -> Segmentation {
        self.pyannote.segment_audio(audio).expect("Failed to segment audio")
    }

    pub fn embed_speaker(&self, audio: &[f32]) -> SpeakerEmbedding {
        self.pyannote.embed_speaker(audio).expect("Failed to embed speaker")
    }
}

pub fn initialize_pyannote(segmentation_model: &str, speaker_model: &str) -> Arc<Mutex<PyannoteIntegration>> {
    Arc::new(Mutex::new(PyannoteIntegration::new(segmentation_model, speaker_model)))
}
