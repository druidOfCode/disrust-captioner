mod audio;
mod transcribe;
mod diarize;

use serde::Serialize;

// Define a struct for device info
#[derive(Serialize)]
struct DeviceInfo {
    id: String,
    name: String,
}

// Define a struct for speaker info
#[derive(Serialize)]
struct SpeakerInfo {
    id: String,
    name: String,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_recording() -> Result<(), String> {
    audio::start_capture()
}

#[tauri::command]
fn stop_recording() -> Result<String, String> {
    let audio_data = audio::stop_capture()?;
    transcribe::transcribe(&audio_data)
}

#[tauri::command]
fn stop_recording_with_diarization() -> Result<String, String> {
    let audio_data = audio::stop_capture()?;
    transcribe::transcribe_with_diarization(&audio_data)
}

#[tauri::command]
fn start_recording_system() -> Result<(), String> {
    audio::start_system_capture()
}

#[tauri::command]
fn stop_recording_system() -> Result<String, String> {
    let audio_data = audio::stop_capture()?;
    transcribe::transcribe(&audio_data)
}

#[tauri::command]
fn stop_recording_system_with_diarization() -> Result<String, String> {
    let audio_data = audio::stop_capture()?;
    transcribe::transcribe_with_diarization(&audio_data)
}

#[tauri::command]
fn set_audio_source(is_system: bool) {
    audio::set_audio_source(is_system);
}

#[tauri::command]
fn is_system_audio_available() -> bool {
    audio::is_system_audio_available()
}

#[tauri::command]
fn get_input_devices() -> Result<Vec<DeviceInfo>, String> {
    let devices = audio::get_input_devices()?;
    
    // Convert to DeviceInfo structs
    let device_infos = devices
        .into_iter()
        .map(|(id, name)| DeviceInfo { id, name })
        .collect();
    
    Ok(device_infos)
}

#[tauri::command]
fn set_input_device(device_id: Option<String>) {
    audio::set_selected_device(device_id);
}

#[tauri::command]
fn rename_speaker(speaker_id: String, new_name: String) -> Result<SpeakerInfo, String> {
    // This is a placeholder function that would actually update the speaker names
    // in a real implementation. For now, we'll just log the request and return the updated info.
    println!("Renaming speaker {} to {}", speaker_id, new_name);
    
    // Return the updated speaker info
    Ok(SpeakerInfo {
        id: speaker_id,
        name: new_name,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_recording,
            stop_recording,
            stop_recording_with_diarization,
            start_recording_system,
            stop_recording_system,
            stop_recording_system_with_diarization,
            set_audio_source,
            is_system_audio_available,
            get_input_devices,
            set_input_device,
            rename_speaker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
