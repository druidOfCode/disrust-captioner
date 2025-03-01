mod audio;
mod transcribe;

use serde::Serialize;

// Define a struct for device info
#[derive(Serialize)]
struct DeviceInfo {
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
fn start_recording_system() -> Result<(), String> {
    audio::start_system_capture()
}

#[tauri::command]
fn stop_recording_system() -> Result<String, String> {
    let audio_data = audio::stop_capture()?;
    transcribe::transcribe(&audio_data)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_recording,
            stop_recording,
            start_recording_system,
            stop_recording_system,
            set_audio_source,
            is_system_audio_available,
            get_input_devices,
            set_input_device
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
