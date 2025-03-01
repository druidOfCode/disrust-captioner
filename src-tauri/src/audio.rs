use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

// Global static to hold the audio data
pub static AUDIO_DATA: once_cell::sync::Lazy<Arc<Mutex<Vec<f32>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

// Global static to hold the recording state
pub static IS_RECORDING: once_cell::sync::Lazy<Arc<AtomicBool>> = 
    once_cell::sync::Lazy::new(|| Arc::new(AtomicBool::new(false)));

// Thread handle for the recording thread
pub static RECORDING_THREAD: once_cell::sync::Lazy<Arc<Mutex<Option<thread::JoinHandle<()>>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

// Global static to hold the sample rate
pub static SAMPLE_RATE: once_cell::sync::Lazy<Arc<Mutex<u32>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(16000)));

// Global static to hold the selected device name
pub static SELECTED_DEVICE: once_cell::sync::Lazy<Arc<Mutex<Option<String>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

// Global static to track if we're using system audio
pub static IS_SYSTEM_AUDIO: once_cell::sync::Lazy<Arc<AtomicBool>> = 
    once_cell::sync::Lazy::new(|| Arc::new(AtomicBool::new(false)));

// Set whether we're using system audio
pub fn set_audio_source(is_system: bool) {
    IS_SYSTEM_AUDIO.store(is_system, Ordering::SeqCst);
}

pub fn start_capture() -> Result<(), String> {
    // If already recording, return early
    if IS_RECORDING.load(Ordering::SeqCst) {
        return Err("Already recording".to_string());
    }

    // Clear previous audio data
    AUDIO_DATA.lock().unwrap().clear();
    
    // Set recording flag
    IS_RECORDING.store(true, Ordering::SeqCst);
    
    // Create a thread to handle the recording
    let audio_data = AUDIO_DATA.clone();
    let is_recording = IS_RECORDING.clone();
    let sample_rate = SAMPLE_RATE.clone();
    let selected_device = SELECTED_DEVICE.clone();
    
    let handle = thread::spawn(move || {
        let host = match cpal::default_host() {
            host => host,
        };
        
        // Get the selected device or default
        let device = if let Some(device_name) = selected_device.lock().unwrap().as_ref() {
            // Try to find the device by name
            let mut found_device = None;
            
            if let Ok(devices) = host.input_devices() {
                for device in devices {
                    if let Ok(name) = device.name() {
                        if name == *device_name {
                            found_device = Some(device);
                            break;
                        }
                    }
                }
            }
            
            match found_device {
                Some(device) => device,
                None => {
                    eprintln!("Selected device '{}' not found, using default", device_name);
                    match host.default_input_device() {
                        Some(device) => device,
                        None => {
                            eprintln!("No input device available");
                            is_recording.store(false, Ordering::SeqCst);
                            return;
                        }
                    }
                }
            }
        } else {
            // Use default device
            match host.default_input_device() {
                Some(device) => device,
                None => {
                    eprintln!("No input device available");
                    is_recording.store(false, Ordering::SeqCst);
                    return;
                }
            }
        };
        
        println!("Using input device: {:?}", device.name().unwrap_or_default());
        
        let config = match device.default_input_config() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error getting default config: {}", e);
                is_recording.store(false, Ordering::SeqCst);
                return;
            }
        };
        
        // Store the original sample rate
        *sample_rate.lock().unwrap() = config.sample_rate().0;
        
        println!("Original sample rate: {}", config.sample_rate().0);
        
        // Clone is_recording for the data callback
        let is_recording_data = is_recording.clone();
        // Clone is_recording for the error callback
        let is_recording_error = is_recording.clone();
        
        let stream = match device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                if is_recording_data.load(Ordering::SeqCst) {
                    audio_data.lock().unwrap().extend_from_slice(data);
                }
            },
            move |err| {
                eprintln!("Stream error: {}", err);
                is_recording_error.store(false, Ordering::SeqCst);
            },
            None,
        ) {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Error building stream: {}", e);
                is_recording.store(false, Ordering::SeqCst);
                return;
            }
        };
        
        if let Err(e) = stream.play() {
            eprintln!("Error playing stream: {}", e);
            is_recording.store(false, Ordering::SeqCst);
            return;
        }
        
        // Keep the stream alive as long as we're recording
        while is_recording.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // Stream will be dropped when this thread ends
    });
    
    // Store the thread handle
    *RECORDING_THREAD.lock().unwrap() = Some(handle);
    
    Ok(())
}

// Function to start capturing system audio
pub fn start_system_capture() -> Result<(), String> {
    // If already recording, return early
    if IS_RECORDING.load(Ordering::SeqCst) {
        return Err("Already recording".to_string());
    }

    // Clear previous audio data
    AUDIO_DATA.lock().unwrap().clear();
    
    // Set recording flag
    IS_RECORDING.store(true, Ordering::SeqCst);
    
    // Create a thread to handle the recording
    let audio_data = AUDIO_DATA.clone();
    let is_recording = IS_RECORDING.clone();
    let sample_rate = SAMPLE_RATE.clone();
    
    let handle = thread::spawn(move || {
        let host = match cpal::default_host() {
            host => host,
        };
        
        // Look for virtual audio devices (like BlackHole, VB-Audio, etc.)
        let mut virtual_device = None;
        let virtual_device_names = [
            "BlackHole", "VB-Audio", "CABLE Output", "Soundflower", "Virtual Audio", "Loopback"
        ];
        
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    // Check if the device name contains any of our virtual device keywords
                    if virtual_device_names.iter().any(|&vname| name.contains(vname)) {
                        println!("Found virtual audio device: {}", name);
                        virtual_device = Some(device);
                        break;
                    }
                }
            }
        }
        
        // Use the virtual device or fall back to default
        let device = match virtual_device {
            Some(device) => device,
            None => {
                eprintln!("No virtual audio device found, using default input device");
                match host.default_input_device() {
                    Some(device) => device,
                    None => {
                        eprintln!("No input device available");
                        is_recording.store(false, Ordering::SeqCst);
                        return;
                    }
                }
            }
        };
        
        println!("Using system audio device: {:?}", device.name().unwrap_or_default());
        
        let config = match device.default_input_config() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error getting default config: {}", e);
                is_recording.store(false, Ordering::SeqCst);
                return;
            }
        };
        
        // Store the original sample rate
        *sample_rate.lock().unwrap() = config.sample_rate().0;
        
        println!("Original sample rate: {}", config.sample_rate().0);
        
        // Clone is_recording for the data callback
        let is_recording_data = is_recording.clone();
        // Clone is_recording for the error callback
        let is_recording_error = is_recording.clone();
        
        let stream = match device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                if is_recording_data.load(Ordering::SeqCst) {
                    audio_data.lock().unwrap().extend_from_slice(data);
                }
            },
            move |err| {
                eprintln!("Stream error: {}", err);
                is_recording_error.store(false, Ordering::SeqCst);
            },
            None,
        ) {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Error building stream: {}", e);
                is_recording.store(false, Ordering::SeqCst);
                return;
            }
        };
        
        if let Err(e) = stream.play() {
            eprintln!("Error playing stream: {}", e);
            is_recording.store(false, Ordering::SeqCst);
            return;
        }
        
        // Keep the stream alive as long as we're recording
        while is_recording.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // Stream will be dropped when this thread ends
    });
    
    // Store the thread handle
    *RECORDING_THREAD.lock().unwrap() = Some(handle);
    
    Ok(())
}

pub fn stop_capture() -> Result<Vec<f32>, String> {
    // If not recording, return early
    if !IS_RECORDING.load(Ordering::SeqCst) {
        return Err("Not recording".to_string());
    }
    
    // Set recording flag to false
    IS_RECORDING.store(false, Ordering::SeqCst);
    
    // Wait for the recording thread to finish
    if let Some(handle) = RECORDING_THREAD.lock().unwrap().take() {
        // Ignore any errors from joining the thread
        let _ = handle.join();
    }
    
    // Get a copy of the audio data
    let audio_data = AUDIO_DATA.lock().unwrap().clone();
    let audio_data_len = audio_data.len(); // Store the length for later use
    
    // Get the original sample rate
    let original_sample_rate = *SAMPLE_RATE.lock().unwrap();
    
    // If the original sample rate is not 16kHz, resample the audio
    let processed_audio = if original_sample_rate != 16000 {
        println!("Resampling audio from {}Hz to 16000Hz", original_sample_rate);
        
        // Simple linear resampling
        resample(&audio_data, original_sample_rate, 16000)
    } else {
        // Use the original audio data
        audio_data
    };
    
    // Apply voice activity detection to trim silence
    let vad_audio = trim_silence(&processed_audio, 0.01, 0.5);
    
    // Print some audio statistics to help with debugging
    println!("Audio statistics:");
    println!("  Original length: {} samples", audio_data_len);
    println!("  Processed length: {} samples", processed_audio.len());
    println!("  After VAD: {} samples", vad_audio.len());
    
    if let Some(max_amplitude) = vad_audio.iter().map(|&x| x.abs()).max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)) {
        println!("  Maximum amplitude: {}", max_amplitude);
    }
    
    if vad_audio.len() < 8000 {  // Less than 0.5 seconds at 16kHz
        println!("Warning: Very short audio segment detected ({}ms)", vad_audio.len() * 1000 / 16000);
    }
    
    // Return the processed audio data
    Ok(vad_audio)
}

// Simple linear resampling function
fn resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return input.to_vec();
    }
    
    let ratio = to_rate as f64 / from_rate as f64;
    let output_size = (input.len() as f64 * ratio) as usize;
    let mut output = Vec::with_capacity(output_size);
    
    for i in 0..output_size {
        let src_idx = i as f64 / ratio;
        let src_idx_floor = src_idx.floor() as usize;
        let src_idx_ceil = src_idx.ceil() as usize;
        
        if src_idx_ceil >= input.len() {
            break;
        }
        
        let t = src_idx - src_idx_floor as f64;
        let sample = input[src_idx_floor] * (1.0 - t as f32) + input[src_idx_ceil] * t as f32;
        output.push(sample);
    }
    
    output
}

// Function to trim silence from the beginning and end of audio
fn trim_silence(audio: &[f32], threshold: f32, min_duration_sec: f32) -> Vec<f32> {
    if audio.is_empty() {
        return Vec::new();
    }
    
    let sample_rate = 16000; // We're working with 16kHz audio at this point
    let min_samples = (min_duration_sec * sample_rate as f32) as usize;
    
    // Find the first non-silent sample
    let mut start_idx = 0;
    while start_idx < audio.len() && audio[start_idx].abs() < threshold {
        start_idx += 1;
    }
    
    // Find the last non-silent sample
    let mut end_idx = audio.len() - 1;
    while end_idx > start_idx && audio[end_idx].abs() < threshold {
        end_idx -= 1;
    }
    
    // If we didn't find any non-silent samples, return the original audio
    if start_idx >= end_idx {
        println!("No non-silent audio detected, using original audio");
        return audio.to_vec();
    }
    
    // Add some padding around the speech (100ms before and after)
    let padding = (0.1 * sample_rate as f32) as usize;
    start_idx = start_idx.saturating_sub(padding);
    end_idx = (end_idx + padding).min(audio.len() - 1);
    
    // Ensure we have at least the minimum duration
    if end_idx - start_idx < min_samples {
        // Try to extend the end while keeping within bounds
        end_idx = (start_idx + min_samples).min(audio.len() - 1);
        
        // If still too short, try to extend the beginning
        if end_idx - start_idx < min_samples {
            start_idx = start_idx.saturating_sub(min_samples - (end_idx - start_idx));
        }
    }
    
    // Return the trimmed audio
    audio[start_idx..=end_idx].to_vec()
}

// Add this new function to get available input devices
pub fn get_input_devices() -> Result<Vec<(String, String)>, String> {
    let host = cpal::default_host();
    
    // Get available input devices
    let devices = match host.input_devices() {
        Ok(devices) => devices,
        Err(e) => return Err(format!("Error getting input devices: {}", e)),
    };
    
    // Collect device names and IDs
    let mut device_list = Vec::new();
    for device in devices {
        let name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
        // Use the name as the ID for now
        device_list.push((name.clone(), name));
    }
    
    // If no devices were found, return an error
    if device_list.is_empty() {
        return Err("No input devices found".to_string());
    }
    
    Ok(device_list)
}

// Function to set the selected device
pub fn set_selected_device(device_name: Option<String>) {
    let mut selected_device = SELECTED_DEVICE.lock().unwrap();
    *selected_device = device_name;
}

// Function to check if system audio is available
pub fn is_system_audio_available() -> bool {
    let host = cpal::default_host();
    
    // Look for virtual audio devices (like BlackHole, VB-Audio, etc.)
    let virtual_device_names = [
        "BlackHole", "VB-Audio", "CABLE Output", "Soundflower", "Virtual Audio", "Loopback"
    ];
    
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                // Check if the device name contains any of our virtual device keywords
                if virtual_device_names.iter().any(|&vname| name.contains(vname)) {
                    return true;
                }
            }
        }
    }
    
    false
}
