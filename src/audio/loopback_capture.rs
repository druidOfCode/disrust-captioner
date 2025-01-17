use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::Split, HeapRb};
use ringbuf::traits::{Consumer as _, Producer as _};
use std::sync::{Arc, Mutex};

use crate::diarization::speaker_manager::SpeakerManager;
use crate::transcription::whisper_integration::TranscriptionBackend;

const BUFFER_SIZE: usize = 1024 * 16;
const PROCESS_INTERVAL: usize = 4096;

pub fn initialize_audio_device(
    maybe_device: Option<cpal::Device>,
) -> Result<cpal::Device, cpal::BuildStreamError> {
    let host = cpal::default_host();
    if let Some(dev) = maybe_device {
        Ok(dev)
    } else {
        host.default_output_device()
            .ok_or(cpal::BuildStreamError::StreamConfigNotSupported)
    }
}

pub fn start_audio_capture(
    device: cpal::Device,
    transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,
    speaker_manager: Arc<Mutex<SpeakerManager>>,
    callback: impl Fn(String, String, i64) + Send + 'static, // Include timestamp
) -> Result<cpal::Stream, Box<dyn std::error::Error + Send + Sync>> {
    // 1) Default input config
    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate().0;
    let chunk_seconds = 5;
    let samples_per_chunk = (sample_rate as usize) * chunk_seconds;

    // 2) Create ring buffer for i16
    let ring_buffer = HeapRb::<i16>::new(BUFFER_SIZE);
    let (mut producer, mut consumer) = ring_buffer.split();

    // 3) Spawn thread to transcribe
    std::thread::spawn(move || {
        let mut buffer = vec![0i16; PROCESS_INTERVAL];
        let mut accum = Vec::new();

        loop {
            let count = consumer.pop_slice(&mut buffer);
            if count > 0 {
                accum.extend_from_slice(&buffer[..count]);

                if accum.len() >= samples_per_chunk {
                    let float_samples: Vec<f32> = accum
                        .iter()
                        .map(|&x| x as f32 / i16::MAX as f32)
                        .collect();

                    let speaker_embedding = if let Ok(mut sm) = speaker_manager.lock() {
                        sm.get_or_create_speaker_embedding(&float_samples)
                    } else {
                        eprintln!("Error locking speaker manager");
                        Vec::new()
                    };

                    let speaker_id = if let Ok(mut sm) = speaker_manager.lock() {
                        sm.identify_speaker(&speaker_embedding)
                    } else {
                        eprintln!("Error locking speaker manager");
                        "Unknown".to_string()
                    };

                    if let Ok(mut trans) = transcription.lock() {
                        match trans.transcribe_audio(&float_samples, sample_rate) {
                            Ok(text) => {
                                let timestamp = chrono::Utc::now().timestamp();
                                callback(speaker_id, text, timestamp);
                            }
                            Err(e) => eprintln!("Whisper error: {:?}", e),
                        }
                    }

                    accum.clear();
                }
            }
        }
    });

    // 4) Build input stream
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