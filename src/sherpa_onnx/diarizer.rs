use std::error::Error;
use sherpa_rs::diarize::{Diarize, DiarizeConfig, Segment};

pub struct SpeakerDiarizer {
    diarizer: Diarize,
}

impl SpeakerDiarizer {
    pub fn new(
        segment_model_path: &str,
        embedding_model_path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let config = DiarizeConfig {
            num_clusters: Some(5), // Adjust as needed
            ..Default::default()
        };

        let diarizer = Diarize::new(segment_model_path, embedding_model_path, config)?;

        Ok(Self { diarizer })
    }

    pub fn diarize(&mut self, samples: &[f32]) -> Result<Vec<Segment>, Box<dyn Error>> {
        let progress_callback = |n_computed_chunks: i32, n_total_chunks: i32| -> i32 {
            let progress = 100 * n_computed_chunks / n_total_chunks;
            println!("🗣️ Diarizing... {}% 🎯", progress);
            0
        };

        let segments = self.diarizer.compute(samples.to_vec(), Some(Box::new(progress_callback)))?;
        Ok(segments)
    }
}