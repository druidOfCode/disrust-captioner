use eframe::{egui, epi};
use std::sync::{Arc, Mutex};
use crate::diarization::pyannote::PyannoteIntegration;
use crate::transcription::whisper_integration::WhisperIntegration;

pub struct App {
    // Add necessary fields for the application state
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Disrust Captioner"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Disrust Captioner");
            // Add UI elements for displaying captions and speaker labels
        });
    }
}

pub fn start_ui(diarization: Arc<Mutex<PyannoteIntegration>>, transcription: Arc<Mutex<WhisperIntegration>>) {
    let app = App {
        // Initialize the application state
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
