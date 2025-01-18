use egui::Rgba;
use rand::Rng;
use sherpa_rs::diarize::Segment;
use std::collections::HashMap;

pub struct SpeakerManager {
    speakers: HashMap<String, String>,
    speaker_colors: HashMap<String, Rgba>,
}

impl SpeakerManager {
    pub fn new() -> Self {
        Self {
            speakers: HashMap::new(),
            speaker_colors: HashMap::new(),
        }
    }

    pub fn identify_speaker_from_segment(&mut self, segment: &Segment) -> String {
        let speaker_id = format!("Speaker_{}", segment.speaker);

        // Assign a random color to the speaker if it's a new speaker
        if !self.speaker_colors.contains_key(&speaker_id) {
            self.speaker_colors
                .insert(speaker_id.clone(), generate_random_color());
        }

        // Check if we have a custom name for this speaker
        if let Some(custom_name) = self.speakers.get(&speaker_id) {
            return custom_name.clone();
        }

        speaker_id
    }

    pub fn rename_speaker(&mut self, speaker_id: &str, new_name: &str) {
        self.speakers
            .insert(speaker_id.to_string(), new_name.to_string());
    }

    // Add a method to get the speaker's color
    pub fn get_speaker_color(&self, speaker_id: &str) -> Option<Rgba> {
        self.speaker_colors.get(speaker_id).cloned()
    }
}

// Function to generate a random color
fn generate_random_color() -> Rgba {
    let mut rng = rand::thread_rng();
    Rgba::from_rgb(rng.gen(), rng.gen(), rng.gen()) // Use Rgba::from_rgb
}
