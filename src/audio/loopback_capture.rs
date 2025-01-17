use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::Split, HeapRb};
use ringbuf::traits::{Consumer as _, Producer as _};
use std::sync::{Arc, Mutex};

use crate::diarization::pyannote::DiarizationBackend;
use crate::diarization::speaker_manager::SpeakerManager;
use crate::transcription::whisper_integration::TranscriptionBackend;

const BUFFER_SIZE: usize = 1024 * 16;
const PROCESS_INTERVAL: usize = 4096;

/// If `maybe_device` is Some, we use it. Otherwise, use the system's default *output* device.
/// Then we attempt to open it in loopback mode (Windows WASAPI only).
pub fn initialize_audio_device(
    maybe_device: Option<cpal::Device>,
) -> Result<cpal::Device, cpal::BuildStreamError> {
    let host = cpal::default_host();
    if let Some(dev) = maybe_device {
        Ok(dev)
    } else {
        // We'll pick the default OUTPUT device, hoping we can open it as an input loopback:
        host.default_output_device()
            .ok_or(cpal::BuildStreamError::StreamConfigNotSupported)
    }
}

/// Start capturing from the selected device in WASAPI loopback mode (if Windows).
/// This function calls build_input_stream(...) on the user-chosen *output* device.
pub fn start_audio_capture(
    device: cpal::Device,
    diarization: Arc<Mutex<dyn DiarizationBackend + Send + Sync>>,
    transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,
    speaker_manager: Arc<Mutex<SpeakerManager>>,
    callback: impl Fn(String, String) + Send + 'static,
) -> Result<cpal::Stream, Box<dyn std::error::Error + Send + Sync>> {
    // 1) Try to get an input config from this device (on Windows, it may be loopback).
    let config = device.default_input_config()?; 
    let sample_rate = config.sample_rate().0;

    // 2) Create a ring buffer for i16 audio
    let ring_buffer = HeapRb::<i16>::new(BUFFER_SIZE);
    let (mut producer, mut consumer) = ring_buffer.split();

    // 3) Background thread: pop from ring buffer, run diarization + transcription
    std::thread::spawn(move || {
        let mut buffer = vec![0i16; PROCESS_INTERVAL];
        loop {
            let count = consumer.pop_slice(&mut buffer);
            if count >= PROCESS_INTERVAL {
                if let Ok(mut diar) = diarization.lock() {
                    if let Ok(segments) = diar.segment_audio(&buffer) {
                        for segment in segments {
                            // Identify speaker
                            let speaker_id = {
                                let embedding = diar.embed_speaker(&segment.samples);
                                speaker_manager
                                    .lock()
                                    .unwrap()
                                    .identify_speaker(&embedding)
                            };

                            // Transcribe (convert i16 -> f32)
                            let float_samples: Vec<f32> = segment
                                .samples
                                .iter()
                                .map(|&x| x as f32 / i16::MAX as f32)
                                .collect();

                            if let Ok(mut trans) = transcription.lock() {
                                if let Ok(text) = trans.transcribe_audio(&float_samples, sample_rate) {
                                    callback(speaker_id.clone(), text);
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // 4) Build an INPUT stream from this device, passing None as the latency hint
    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| {
                producer.push_slice(data);
            },
            move |err| eprintln!("Error in audio stream: {}", err),
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data: &[u16], _| {
                let int_data: Vec<i16> = data
                    .iter()
                    .map(|&x| (x as i32 - (u16::MAX as i32 / 2)) as i16)
                    .collect();
                producer.push_slice(&int_data);
            },
            move |err| eprintln!("Error in audio stream: {}", err),
            None,
        ),
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                let int_data: Vec<i16> = data
                    .iter()
                    .map(|&x| (x * i16::MAX as f32) as i16)
                    .collect();
                producer.push_slice(&int_data);
            },
            move |err| eprintln!("Error in audio stream: {}", err),
            None,
        ),
        _ => unreachable!("Unsupported sample format"),
    }?;

    // 5) Start capturing
    stream.play()?;
    Ok(stream)
}
