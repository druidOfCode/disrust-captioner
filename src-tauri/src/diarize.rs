// Implementation for speaker diarization

use std::collections::HashMap;

// Structure to hold speaker segments
#[derive(Debug, Clone)]
pub struct SpeakerSegment {
    pub start_time: f32,
    pub end_time: f32,
    pub speaker_id: String,
}

// Structure to hold diarization results
#[derive(Debug, Clone)]
pub struct DiarizationResult {
    pub segments: Vec<SpeakerSegment>,
}

// Improved feature extraction for speaker identification
fn extract_features(samples: &[f32], sample_rate: u32) -> Vec<Vec<f32>> {
    // Use a more sophisticated feature extraction approach
    // This implementation uses multiple features per window:
    // 1. Energy (volume)
    // 2. Zero-crossing rate (frequency characteristic)
    // 3. Spectral centroid (brightness of sound)
    
    let window_size = (sample_rate as f32 * 0.025) as usize; // 25ms window
    let hop_size = (sample_rate as f32 * 0.010) as usize; // 10ms hop
    
    let mut feature_vectors = Vec::new();
    
    for i in (0..samples.len()).step_by(hop_size) {
        if i + window_size > samples.len() {
            break;
        }
        
        let window = &samples[i..i+window_size];
        let mut features = Vec::new();
        
        // Feature 1: Energy (volume)
        let energy: f32 = window.iter().map(|&s| s * s).sum();
        features.push(energy.sqrt());
        
        // Feature 2: Zero-crossing rate
        let mut zcr = 0.0;
        for j in 1..window.len() {
            if (window[j] >= 0.0 && window[j-1] < 0.0) || 
               (window[j] < 0.0 && window[j-1] >= 0.0) {
                zcr += 1.0;
            }
        }
        zcr /= window.len() as f32;
        features.push(zcr);
        
        // Feature 3: Spectral centroid approximation
        // This is a simplified version - a real implementation would use FFT
        let mut weighted_sum = 0.0;
        let mut sum = 0.0;
        for j in 0..window.len() {
            let weight = j as f32 / window.len() as f32; // Frequency proxy
            weighted_sum += window[j].abs() * weight;
            sum += window[j].abs();
        }
        let spectral_centroid = if sum > 0.0 { weighted_sum / sum } else { 0.0 };
        features.push(spectral_centroid);
        
        feature_vectors.push(features);
    }
    
    feature_vectors
}

// Calculate distance between two feature vectors
fn feature_distance(a: &[f32], b: &[f32]) -> f32 {
    // Euclidean distance with feature weighting
    let weights = [0.6, 0.3, 0.1]; // Weights for energy, ZCR, spectral centroid
    
    let mut sum_sq = 0.0;
    for i in 0..a.len().min(b.len()).min(weights.len()) {
        let diff = a[i] - b[i];
        sum_sq += weights[i] * diff * diff;
    }
    
    sum_sq.sqrt()
}

// Improved clustering algorithm for speaker identification
fn cluster_features(features: &[Vec<f32>], segment_times: &[(f32, f32)]) -> Vec<SpeakerSegment> {
    // Use a more robust clustering approach
    // This implementation uses a simple but effective method:
    // 1. Start with first segment as first speaker
    // 2. For each subsequent segment, compare to all existing speaker profiles
    // 3. Assign to closest speaker if distance is below threshold, otherwise create new speaker
    
    if features.is_empty() || segment_times.is_empty() {
        return Vec::new();
    }
    
    let mut segments = Vec::new();
    let mut speaker_profiles: HashMap<usize, Vec<Vec<f32>>> = HashMap::new();
    
    // Threshold for considering a new speaker
    let new_speaker_threshold = 0.5;
    
    // Minimum segments needed to consider a speaker stable
    let min_segments_for_stability = 3;
    
    // Process each segment
    for (i, feature) in features.iter().enumerate() {
        // Find the closest speaker profile
        let mut closest_speaker = 0;
        let mut min_distance = f32::MAX;
        let mut is_new_speaker = true;
        
        for (speaker_id, profile_features) in &speaker_profiles {
            // Calculate average distance to this speaker's profile
            let mut total_distance = 0.0;
            let mut count = 0;
            
            // Use only the most recent segments for comparison (recency bias)
            let start_idx = profile_features.len().saturating_sub(min_segments_for_stability);
            for profile_feature in &profile_features[start_idx..] {
                total_distance += feature_distance(feature, profile_feature);
                count += 1;
            }
            
            let avg_distance = total_distance / count as f32;
            
            if avg_distance < min_distance {
                min_distance = avg_distance;
                closest_speaker = *speaker_id;
                
                // If distance is below threshold, assign to this speaker
                if avg_distance < new_speaker_threshold {
                    is_new_speaker = false;
                }
            }
        }
        
        // Determine speaker ID
        let speaker_id = if is_new_speaker && speaker_profiles.len() < 8 {
            // Create a new speaker (limit to 8 speakers for UI colors)
            let new_id = speaker_profiles.len();
            speaker_profiles.insert(new_id, vec![feature.clone()]);
            new_id
        } else {
            // Add this segment to the closest speaker's profile
            speaker_profiles.entry(closest_speaker)
                .or_insert_with(Vec::new)
                .push(feature.clone());
            closest_speaker
        };
        
        // Create segment with consistent speaker ID
        segments.push(SpeakerSegment {
            start_time: segment_times[i].0,
            end_time: segment_times[i].1,
            speaker_id: format!("Speaker{}", speaker_id + 1),
        });
    }
    
    // Post-processing: smooth out speaker assignments
    smooth_speaker_assignments(&mut segments);
    
    segments
}

// Smooth out speaker assignments to avoid rapid switching
fn smooth_speaker_assignments(segments: &mut [SpeakerSegment]) {
    if segments.len() < 3 {
        return;
    }
    
    // Use a sliding window approach to smooth speaker assignments
    for i in 1..segments.len() - 1 {
        let prev = &segments[i-1].speaker_id;
        let curr = &segments[i].speaker_id;
        let next = &segments[i+1].speaker_id;
        
        // If current segment is sandwiched between two segments with the same speaker,
        // and it's different, change it to match the surrounding segments
        if prev == next && curr != prev {
            segments[i].speaker_id = prev.clone();
        }
    }
}

// Main diarization function
pub fn diarize(samples: &[f32], sample_rate: u32) -> DiarizationResult {
    // Step 1: Segment the audio based on silence
    // For simplicity, we'll use fixed-size segments
    let segment_duration = 1.5; // 1.5 seconds per segment (shorter for better resolution)
    let samples_per_segment = (segment_duration * sample_rate as f32) as usize;
    
    let mut segment_times = Vec::new();
    let mut feature_vectors = Vec::new();
    
    // Skip segments with very low energy (silence)
    let silence_threshold = 0.01;
    
    for i in (0..samples.len()).step_by(samples_per_segment) {
        if i + samples_per_segment > samples.len() {
            break;
        }
        
        let start_time = i as f32 / sample_rate as f32;
        let end_time = (i + samples_per_segment) as f32 / sample_rate as f32;
        
        // Extract segment samples
        let segment_samples = &samples[i..i+samples_per_segment];
        
        // Check if segment is silence
        let energy: f32 = segment_samples.iter().map(|&s| s * s).sum::<f32>() / segment_samples.len() as f32;
        if energy < silence_threshold {
            continue; // Skip silent segments
        }
        
        segment_times.push((start_time, end_time));
        
        // Extract features for this segment
        let features = extract_features(segment_samples, sample_rate);
        
        // Use average of features for the segment
        if !features.is_empty() {
            let mut avg_features = vec![0.0; features[0].len()];
            for feature in &features {
                for (i, &value) in feature.iter().enumerate() {
                    avg_features[i] += value;
                }
            }
            
            for value in &mut avg_features {
                *value /= features.len() as f32;
            }
            
            feature_vectors.push(avg_features);
        }
    }
    
    // Step 2: Cluster the features to identify speakers
    let speaker_segments = cluster_features(&feature_vectors, &segment_times);
    
    DiarizationResult {
        segments: speaker_segments,
    }
}

// Function to combine diarization results with transcription
pub fn combine_with_transcription(
    diarization: &DiarizationResult,
    transcription: &str,
    timestamps: &[(f32, f32)]
) -> String {
    let mut result = String::new();
    
    // Split transcription into words
    let words: Vec<&str> = transcription.split_whitespace().collect();
    
    // Assume timestamps correspond to words
    // In a real implementation, you would need to align them properly
    if words.len() != timestamps.len() {
        return format!("Error: Mismatch between words ({}) and timestamps ({})", 
                      words.len(), timestamps.len());
    }
    
    // Create a vector of (speaker, word) pairs
    let mut speaker_word_pairs = Vec::new();
    
    // Assign speaker to each word based on timestamp
    for (i, &word) in words.iter().enumerate() {
        let word_time = timestamps[i].0;
        
        // Find which speaker segment this word belongs to
        let mut speaker = String::from("Unknown");
        for segment in &diarization.segments {
            if word_time >= segment.start_time && word_time <= segment.end_time {
                speaker = segment.speaker_id.clone();
                break;
            }
        }
        
        // Add to result
        speaker_word_pairs.push((speaker, word.to_string()));
    }
    
    // Join segments with appropriate formatting
    let mut current_speaker = String::new();
    let mut current_text = String::new();
    
    for (speaker, word) in speaker_word_pairs {
        if speaker != current_speaker {
            // New speaker, start a new paragraph
            if !current_text.is_empty() {
                result.push_str(&format!("{}: {}\n\n", current_speaker, current_text.trim()));
                current_text.clear();
            }
            current_speaker = speaker;
        }
        
        current_text.push_str(&format!("{} ", word));
    }
    
    // Add the last segment
    if !current_text.is_empty() {
        result.push_str(&format!("{}: {}\n", current_speaker, current_text.trim()));
    }
    
    result
} 