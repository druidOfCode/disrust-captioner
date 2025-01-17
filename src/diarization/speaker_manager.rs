use std::collections::HashMap;

pub struct SpeakerManager {
    speakers: HashMap<String, String>,
    embeddings: Vec<(Vec<f32>, String)>,
    next_id: usize,
}

impl SpeakerManager {
    pub fn new() -> Self {
        Self {
            speakers: HashMap::new(),
            embeddings: Vec::new(),
            next_id: 1,
        }
    }

    // Placeholder for embedding generation - replace with actual Pyannote call
    pub fn get_or_create_speaker_embedding(&mut self, _audio_segment: &[f32]) -> Vec<f32> {
        // In a real implementation, you would use Pyannote to get embeddings here
        vec![0.1, 0.2, 0.3] // Placeholder
    }

    pub fn identify_speaker(&mut self, embedding: &Vec<f32>) -> String {
        // Find closest matching speaker
        for (stored_embedding, speaker_id) in &self.embeddings {
            if cosine_similarity(embedding, stored_embedding) > 0.85 {
                return self.speakers.get(speaker_id)
                    .unwrap_or(speaker_id)
                    .clone();
            }
        }
        
        // New speaker found
        let new_id = format!("Speaker_{}", self.next_id);
        self.next_id += 1;
        self.embeddings.push((embedding.clone(), new_id.clone()));
        self.speakers.insert(new_id.clone(), new_id.clone());
        new_id
    }

    pub fn rename_speaker(&mut self, speaker_id: &str, new_name: &str) {
        if let Some(name) = self.speakers.get_mut(speaker_id) {
            *name = new_name.to_string();
        }
    }
}

fn cosine_similarity(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }
    dot_product / (magnitude_a * magnitude_b)
}