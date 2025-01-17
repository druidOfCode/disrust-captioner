use std::sync::{Arc, Mutex};

use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Stream;

use crate::audio::loopback_capture;
use crate::config::persistence::Config;
use crate::transcription::whisper_integration::TranscriptionBackend;

pub struct CaptionerApp {
    // We only store transcription backend now
    transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,

    is_running: bool,
    available_devices: Vec<cpal::Device>,
    selected_device: Option<usize>,

    stream: Option<Stream>,
    tx: Sender<(String, String)>,
    rx: Receiver<(String, String)>,

    config: Config,
    transcription_history: Vec<(String, String)>,
}

impl CaptionerApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,
    ) -> Self {
        let host = cpal::default_host();
        let available_devices = match host.devices() {
            Ok(iter) => iter.collect(),
            Err(_) => vec![],
        };

        let (tx, rx) = unbounded();
        let config = Config::load("config.json");

        Self {
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
            let trans = Arc::clone(&self.transcription);
            let sender = self.tx.clone();

            let stream_result = loopback_capture::start_audio_capture(
                device,
                trans,
                move |speaker, text| {
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
        self.stream = None;
        self.is_running = false;
    }
}

impl eframe::App for CaptionerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for new transcripts
        while let Ok((speaker, text)) = self.rx.try_recv() {
            self.transcription_history.push((speaker, text));
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
                for (speaker, text) in &self.transcription_history {
                    ui.label(format!("{}: {}", speaker, text));
                }
            });
        });

        ctx.request_repaint();
    }
}
