// Implementation for transcription using whisper-rs 0.13.2

use whisper_rs::{WhisperContext, FullParams, WhisperContextParameters, SamplingStrategy};
use std::path::Path;

pub fn transcribe(samples: &[f32]) -> Result<String, String> {
    // Check if we have enough audio data
    if samples.is_empty() {
        return Ok("No audio data received.".to_string());
    }
    
    println!("Transcribing {} samples of audio data", samples.len());
    
    // Calculate audio duration in seconds
    let duration_sec = samples.len() as f32 / 16000.0;
    println!("Audio duration: {:.2} seconds", duration_sec);
    
    // If audio is too short, return early
    if duration_sec < 0.5 {
        println!("Audio too short for reliable transcription");
        return Ok("Audio too short for reliable transcription.".to_string());
    }
    
    // Create context parameters with default settings
    let params = WhisperContextParameters::default();
    
    // Check if the model file exists
    let model_path = "whisper-small.bin";
    if !Path::new(model_path).exists() {
        return Err(format!(
            "Model file '{}' not found. Please download it using one of the following methods:\n\n\
            1. Download from Hugging Face: https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin\n\
            2. Or use the following command in your terminal:\n\
               wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin -O whisper-small.bin\n\n\
            After downloading, place the file in the application directory: {}",
            model_path,
            std::env::current_dir().unwrap_or_default().display()
        ));
    }
    
    // Create context from model file
    let ctx = WhisperContext::new_with_params(
        model_path, // Use string path
        params
    ).map_err(|e| format!("Failed to load model: {}", e))?;
    
    // Create state for inference
    let mut state = ctx.create_state()
        .map_err(|e| format!("Failed to create state: {}", e))?;
    
    // Create parameters with default settings
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    
    // Set additional parameters if needed
    params.set_no_speech_thold(0.3); // Even lower threshold for detecting speech
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_language(Some("en")); // Force English language
    
    // Normalize the audio to ensure it's within the expected range
    let normalized_samples = normalize_audio(samples);
    
    // Calculate and print audio statistics
    let rms = calculate_rms(&normalized_samples);
    println!("Audio RMS: {:.6}", rms);
    
    // Run inference
    println!("Running inference on audio...");
    state.full(params, &normalized_samples)
        .map_err(|e| format!("Failed to run inference: {}", e))?;
    
    // Get number of segments
    let num_segments = state.full_n_segments()
        .map_err(|e| format!("Failed to get number of segments: {}", e))?;
    
    println!("Transcription produced {} segments", num_segments);
    
    let mut transcript = String::new();
    
    // Iterate through segments and collect text
    for i in 0..num_segments {
        let segment_text = state.full_get_segment_text(i)
            .map_err(|e| format!("Failed to get segment text: {}", e))?;
        println!("Segment {}: {}", i, segment_text);
        transcript.push_str(&segment_text);
        transcript.push(' ');
    }
    
    // If no text was transcribed, provide a helpful message
    if transcript.trim().is_empty() {
        println!("No speech detected in the audio");
        return Ok("No speech detected in the audio.".to_string());
    }
    
    println!("Final transcript: {}", transcript);
    Ok(transcript)
}

// Function to normalize audio to ensure it's within the expected range
fn normalize_audio(samples: &[f32]) -> Vec<f32> {
    // Find the maximum absolute value in the samples
    let max_abs = samples.iter()
        .map(|&s| s.abs())
        .fold(0.0f32, |a, b| a.max(b));
    
    println!("Maximum amplitude before normalization: {}", max_abs);
    
    // If the maximum absolute value is very small, return the original samples
    if max_abs < 1e-6 {
        println!("Audio is nearly silent, skipping normalization");
        return samples.to_vec();
    }
    
    // Normalize the samples to have a maximum absolute value of 0.95
    let scale = 0.95 / max_abs;
    println!("Applying normalization scale factor: {}", scale);
    samples.iter().map(|&s| s * scale).collect()
}

// Function to calculate the root mean square (RMS) of audio samples
fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    
    let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}