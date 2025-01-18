pub mod diarizer;
pub mod transcriber;

// Add these imports in `sherpa_onnx/mod.rs`
use crate::diarization::speaker_manager::SpeakerManager;
use crossbeam_channel::Sender;
use std::error::Error;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SherpaOnnx {
    pub transcriber: Arc<Mutex<transcriber::WhisperTranscriber>>,
    pub diarizer: Arc<Mutex<diarizer::SpeakerDiarizer>>,
}

impl SherpaOnnx {
    pub fn new(
        whisper_encoder_path: &str,
        whisper_decoder_path: &str,
        whisper_tokens_path: &str,
        segmentation_model_path: &str,
        embedding_model_path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let transcriber = Arc::new(Mutex::new(transcriber::WhisperTranscriber::new(
            whisper_encoder_path,
            whisper_decoder_path,
            whisper_tokens_path,
            "en", // Replace with the correct language code if needed
            "cpu", // Or "cuda" if you have a GPU
        )?));

        let diarizer = Arc::new(Mutex::new(diarizer::SpeakerDiarizer::new(
            segmentation_model_path,
            embedding_model_path,
        )?));

        Ok(Self {
            transcriber,
            diarizer,
        })
    }

    // Add a method to run both transcription and diarization
    pub fn process_audio(
        &mut self,
        samples: &[f32],
        sample_rate: i32,
        speaker_manager: Arc<Mutex<SpeakerManager>>,
        tx: Sender<(String, String, i64)>,
    ) -> Result<(), Box<dyn Error>> {
        // 1. Diarize
        let segments = self.diarizer.lock().unwrap().diarize(samples)?;

        // 2. Transcribe and assign speakers
        for segment in segments {
            let speaker_id = if let Ok(mut sm) = speaker_manager.lock() {
                sm.identify_speaker_from_segment(&segment)
            } else {
                "Unknown".to_string()
            };

            // Extract segment samples
            let start_sample = (segment.start * sample_rate as f32).round() as usize;
            let end_sample = (segment.end * sample_rate as f32).round() as usize;
            let segment_samples =
                &samples[start_sample.min(samples.len())..end_sample.min(samples.len())];

            // 3. Transcribe
            if let Ok(text) = self.transcriber.lock().unwrap()
                .transcribe(sample_rate as u32, segment_samples.to_vec())
            {
                let timestamp = chrono::Utc::now().timestamp(); // You might want to refine the timestamp based on the segment
                tx.send((speaker_id, text, timestamp))?;
            }
        }

        Ok(())
    }
}