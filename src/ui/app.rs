// app.rs

use std::sync::{Arc, Mutex};

use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Stream;

use crate::audio::loopback_capture;
use crate::config::persistence::Config;
use crate::diarization::pyannote::DiarizationBackend;
use crate::diarization::speaker_manager::SpeakerManager;
use crate::transcription::whisper_integration::TranscriptionBackend;

pub struct CaptionerApp {
    // Data / backends
    speaker_manager: Arc<Mutex<SpeakerManager>>,
    diarization: Arc<Mutex<dyn DiarizationBackend + Send + Sync>>,
    transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,

    // UI state
    is_running: bool,
    available_devices: Vec<cpal::Device>,
    selected_device: Option<usize>,

    // Stream & channels
    stream: Option<Stream>,
    tx: Sender<(String, String)>,
    rx: Receiver<(String, String)>,

    // Live data
    config: Config,
    transcription_history: Vec<(String, String)>,
}

impl CaptionerApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        speaker_manager: Arc<Mutex<SpeakerManager>>,
        diarization: Arc<Mutex<dyn DiarizationBackend + Send + Sync>>,
        transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,
    ) -> Self {
        // Setup device list
        let host = cpal::default_host();
        let available_devices = match host.devices() {
            Ok(iter) => iter.collect(),
            Err(_) => vec![],
        };

        // Create a channel for capturing (speaker, text) from the audio thread
        let (tx, rx) = unbounded();

        // Load existing config (speaker names, etc.)
        let config = Config::load("config.json");

        Self {
            speaker_manager,
            diarization,
            transcription,

            is_running: false,
            available_devices,
            selected_device: None,

            stream: None,
            tx,
            rx,

            config,
            transcription_history: Vec::new(),
        }
    }

    fn start_capture(&mut self) {
        if let Some(idx) = self.selected_device {
            let device = self.available_devices[idx].clone();

            let diar = Arc::clone(&self.diarization);
            let trans = Arc::clone(&self.transcription);
            let spk_man = Arc::clone(&self.speaker_manager);

            // We'll clone the tx so we can send from the capture thread
            let sender = self.tx.clone();
            let stream_result = loopback_capture::start_audio_capture(
                device,
                diar,
                trans,
                spk_man,
                move |speaker, text| {
                    // push (speaker, text) onto the channel
                    sender.send((speaker, text)).ok();
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
        // Dropping the Stream stops audio
        self.stream = None;
        self.is_running = false;
    }
}

impl eframe::App for CaptionerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain any new transcriptions from the channel
        while let Ok((speaker, text)) = self.rx.try_recv() {
            // The SpeakerManager might let the user rename speakers; for now we store raw
            self.transcription_history.push((speaker.clone(), text.clone()));

            // Keep history at a size limit
            if self.transcription_history.len() > 200 {
                self.transcription_history.remove(0);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Disrust Captioner");

            // Device selection
            ui.horizontal(|ui| {
                ui.label("Audio Device:");
                egui::ComboBox::from_label("")
                    .selected_text(match self.selected_device {
                        Some(idx) => {
                            self.available_devices[idx].name().unwrap_or("Unnamed Device".to_string())
                        }
                        None => "Select a device".to_string(),
                    })
                    .show_ui(ui, |ui| {
                        for (idx, device) in self.available_devices.iter().enumerate() {
                            let name = device.name().unwrap_or(format!("Device {}", idx));
                            ui.selectable_value(&mut self.selected_device, Some(idx), name);
                        }
                    });
            });

            // Start/Stop button
            if ui.button(if self.is_running { "Stop" } else { "Start" }).clicked() {
                if self.is_running {
                    self.stop_capture();
                } else {
                    self.start_capture();
                }
            }

            ui.separator();

            // Show speaker rename UI (example)
            ui.heading("Speakers");
            if let Ok(spk_man) = self.speaker_manager.lock() {
                // If you store speaker ID -> user name in speaker_manager, show them here
                // (Implementation up to you)
                // e.g. for each known ID, a text field to rename
            }

            ui.separator();

            // Transcription history
            ui.heading("Transcription");
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for (speaker, text) in &self.transcription_history {
                    ui.label(format!("{}: {}", speaker, text));
                }
            });
        });

        ctx.request_repaint();
    }
}
