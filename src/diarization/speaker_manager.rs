use std::collections::HashMap;

pub struct SpeakerManager {
    speakers: HashMap<Vec<f32>, String>,
    next_id: usize,
}

impl SpeakerManager {
    pub fn new() -> Self {
        Self {
            speakers: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn identify_speaker(&mut self, embedding: &Vec<f32>) -> String {
        // Simple cosine similarity check
        for (stored_embedding, speaker_id) in &self.speakers {
            if cosine_similarity(embedding, stored_embedding) > 0.85 {
                return speaker_id.clone();
            }
        }
        
        // New speaker found
        let new_id = format!("Speaker_{}", self.next_id);
        self.next_id += 1;
        self.speakers.insert(embedding.clone(), new_id.clone());
        new_id
    }
}

fn cosine_similarity(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot_product / (magnitude_a * magnitude_b)
}
