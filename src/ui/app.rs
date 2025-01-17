use std::sync::{Arc, Mutex};

use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Stream;

use crate::audio::loopback_capture;
use crate::config::persistence::Config;
use crate::diarization::speaker_manager::SpeakerManager;
use crate::transcription::whisper_integration::TranscriptionBackend;

pub struct CaptionerApp {
    transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,
    speaker_manager: Arc<Mutex<SpeakerManager>>,
    is_running: bool,
    available_output_devices: Vec<cpal::Device>,
    selected_device: Option<usize>,

    stream: Option<Stream>,
    tx: Sender<(String, String, i64)>, // Include timestamp
    rx: Receiver<(String, String, i64)>, // Include timestamp

    config: Config,
    transcription_history: Vec<(String, String, i64)>, // Include timestamp

    edit_speaker_id: Option<String>, // ID of speaker being edited
    temp_name: String, // Temporary name during editing
}

impl CaptionerApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,
        speaker_manager: Arc<Mutex<SpeakerManager>>,
    ) -> Self {
        let host = cpal::default_host();
        
        // Collect output devices
        let available_output_devices = host.output_devices()
            .expect("Failed to get output devices")
            .collect::<Vec<_>>();

        let (tx, rx) = unbounded();
        let config = Config::load("config.json");

        Self {
            transcription,
            speaker_manager,
            is_running: false,
            available_output_devices,
            selected_device: None,

            stream: None,
            tx,
            rx,

            config,
            transcription_history: Vec::new(),

            edit_speaker_id: None,
            temp_name: String::new(),
        }
    }

    fn start_capture(&mut self) {
        if let Some(idx) = self.selected_device {
            let device = self.available_output_devices[idx].clone();
            let trans = Arc::clone(&self.transcription);
            let speaker_manager = Arc::clone(&self.speaker_manager);
            let sender = self.tx.clone();

            let stream_result = loopback_capture::start_audio_capture(
                device,
                trans,
                speaker_manager,
                move |speaker, text, timestamp| {
                    sender.send((speaker, text, timestamp)).ok();
                },
            );

            match stream_result {
                Ok(s) => {
                    self.stream = Some(s);
                    self.is_running = true;
                }
                Err(e) => {
                    eprintln!("Failed to start capture: {}", e);
                }
            }
        }
    }

    fn stop_capture(&mut self) {
        self.stream = None;
        self.is_running = false;
    }
}

impl eframe::App for CaptionerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for new transcripts
        while let Ok((speaker, text, timestamp)) = self.rx.try_recv() {
            self.transcription_history.push((speaker, text, timestamp));
            if self.transcription_history.len() > 200 {
                self.transcription_history.remove(0);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Disrust Captioner");

            // Device selection (Output Devices Only)
            ui.horizontal(|ui| {
                ui.label("Audio Device:");
                egui::ComboBox::from_label("")
                    .selected_text(match self.selected_device {
                        Some(idx) => {
                            self.available_output_devices[idx].name().unwrap_or("Unnamed Device".to_string())
                        }
                        None => "Select a device".to_string(),
                    })
                    .show_ui(ui, |ui| {
                        for (idx, device) in self.available_output_devices.iter().enumerate() {
                            let name = device.name().unwrap_or(format!("Device {}", idx));
                            ui.selectable_value(&mut self.selected_device, Some(idx), name);
                        }
                    });
            });

            // Start/Stop
            if ui.button(if self.is_running { "Stop" } else { "Start" }).clicked() {
                if self.is_running {
                    self.stop_capture();
                } else {
                    self.start_capture();
                }
            }

            ui.separator();
            ui.heading("Transcription");
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for (speaker_id, text, timestamp) in &self.transcription_history {
                    let speaker_name = self.config
                        .get_speaker_name(speaker_id)
                        .unwrap_or(speaker_id)
                        .clone();

                    // Display name (clickable to edit)
                    ui.horizontal(|ui| {
                        if ui.button(&speaker_name).clicked() {
                            self.edit_speaker_id = Some(speaker_id.clone());
                            self.temp_name = speaker_name.clone();
                        }

                        // Timestamp (consider formatting it nicely)
                        ui.label(format!("[{}] {}: {}", chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(chrono::NaiveDateTime::from_timestamp_opt(*timestamp, 0).unwrap(), chrono::Utc).format("%Y-%m-%d %H:%M:%S"), speaker_name, text));
                    });

                    // Handle editing mode
                    if let Some(editing_id) = &self.edit_speaker_id {
                        if editing_id == speaker_id {
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut self.temp_name);
                                if ui.button("Save").clicked() {
                                    if let Ok(mut sm) = self.speaker_manager.lock() {
                                        sm.rename_speaker(speaker_id, &self.temp_name);
                                    }
                                    self.config.set_speaker_name(speaker_id.clone(), self.temp_name.clone());
                                    self.config.save("config.json");
                                    self.edit_speaker_id = None;
                                }
                                if ui.button("Cancel").clicked() {
                                    self.edit_speaker_id = None;
                                }
                            });
                        }
                    }
                }
            });
        });

        ctx.request_repaint();
    }
}