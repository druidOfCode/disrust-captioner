use eframe::egui;
use std::sync::{Arc, Mutex};
use crate::diarization::pyannote::DiarizationBackend;
use crate::transcription::whisper_integration::TranscriptionBackend;
use crate::config::persistence::Config;
use cpal::{Device, traits::{DeviceTrait, HostTrait}};
use std::collections::HashMap;

#[allow(dead_code)]
pub struct CaptionerApp {
    diarization: Arc<Mutex<dyn DiarizationBackend + Send + Sync>>,
    transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,
    current_transcript: String,
    current_speaker: String,
    is_running: bool,
    available_devices: Vec<Device>,
    selected_device: Option<usize>,
    speaker_names: HashMap<String, String>,
    config: Config,
    transcription_history: Vec<(String, String)>, // (speaker, text)
}

impl CaptionerApp {
    fn new(
        _cc: &eframe::CreationContext<'_>,
        diarization: Arc<Mutex<dyn DiarizationBackend + Send + Sync>>,
        transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,
    ) -> Self {
        let host = cpal::default_host();
        let available_devices = host.devices()
            .map(|devices| devices.collect())
            .unwrap_or_default();
        
        Self {
            diarization,
            transcription,
            current_transcript: String::new(),
            current_speaker: String::new(),
            is_running: false,
            available_devices,
            selected_device: None,
            speaker_names: HashMap::new(),
            config: Config::load("config.json"),
            transcription_history: Vec::new(),
        }
    }

    pub fn get_selected_device(&self) -> Option<cpal::Device> {
        self.selected_device.and_then(|idx| self.available_devices.get(idx).cloned())
    }

    pub fn add_transcription(&mut self, speaker: String, text: String) {
        self.transcription_history.push((speaker.clone(), text.clone()));
        self.current_speaker = speaker;
        self.current_transcript = text;
        
        // Keep history at a reasonable size
        if self.transcription_history.len() > 100 {
            self.transcription_history.remove(0);
        }
    }
}

impl eframe::App for CaptionerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Disrust Captioner");
            
            // Device selection
            ui.horizontal(|ui| {
                ui.label("Audio Device:");
                egui::ComboBox::from_label("")
                    .selected_text(match &self.selected_device {
                        Some(idx) => self.available_devices[*idx].name().unwrap_or_default(),
                        None => "Select a device".to_string(),
                    })
                    .show_ui(ui, |ui| {
                        for (idx, device) in self.available_devices.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.selected_device,
                                Some(idx),
                                device.name().unwrap_or_default()
                            );
                        }
                    });
            });

            // Start/Stop button
            if ui.button(if self.is_running { "Stop" } else { "Start" }).clicked() {
                self.is_running = !self.is_running;
            }

            ui.separator();

            // Speaker list and naming
            ui.heading("Speakers");
            for (speaker_id, name) in &mut self.speaker_names {
                ui.horizontal(|ui| {
                    let mut edited_name = name.clone();
                    if ui.text_edit_singleline(&mut edited_name).changed() {
                        *name = edited_name.clone();
                        self.config.set_speaker_name(speaker_id.clone(), edited_name);
                        self.config.save("config.json");
                    }
                });
            }

            ui.separator();

            // Transcription history
            ui.heading("Transcription");
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for (speaker, text) in &self.transcription_history {
                    let speaker_name = self.speaker_names
                        .get(speaker)
                        .unwrap_or(speaker)
                        .clone();
                    ui.label(format!("{}: {}", speaker_name, text));
                }
            });

            // Request repaint frequently to update the UI
            ctx.request_repaint();
        });
    }
}

pub fn start_ui(
    diarization: Arc<Mutex<dyn DiarizationBackend + Send + Sync>>,
    transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,
    _selected_device: Option<cpal::Device>,
) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Disrust Captioner",
        native_options,
        Box::new(move |cc| Ok(Box::new(CaptionerApp::new(
            cc,
            diarization.clone(),
            transcription.clone(),
        )))),
    ).expect("Failed to start eframe");
}
