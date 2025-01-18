use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::Split, HeapRb};
use ringbuf::traits::{Consumer as _, Producer as _};
use std::sync::{Arc, Mutex};

use crate::diarization::speaker_manager::SpeakerManager;
use crate::sherpa_onnx::SherpaOnnx;

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
    sherpa_onnx: Arc<Mutex<SherpaOnnx>>,
    speaker_manager: Arc<Mutex<SpeakerManager>>,
    callback: impl Fn(String, String, i64) + Send + 'static,
    audio_data_for_plot: Arc<Mutex<Vec<f32>>>, // Add parameter for audio data
) -> Result<cpal::Stream, Box<dyn std::error::Error + Send + Sync>> {
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0;
    let chunk_seconds = 5;
    let samples_per_chunk = (sample_rate as usize) * chunk_seconds;

    let ring_buffer = HeapRb::<i16>::new(BUFFER_SIZE);
    let (mut producer, mut consumer) = ring_buffer.split();

    let sherpa_onnx = sherpa_onnx.clone();

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

                    // Push data to the audio_data_for_plot vector
                    if let Ok(mut audio_data) = audio_data_for_plot.lock() {
                        audio_data.extend_from_slice(&float_samples);
                        if audio_data.len() > samples_per_chunk * 2 {
                            audio_data.drain(..samples_per_chunk);
                        }
                    }

                    let (tx, rx) = crossbeam_channel::unbounded();
                    let speaker_manager_clone = speaker_manager.clone();

                    if let Ok(mut sherpa_onnx_guard) = sherpa_onnx.lock() {
                        if let Err(e) = sherpa_onnx_guard.process_audio(&float_samples, sample_rate as i32, speaker_manager_clone, tx) {
                            eprintln!("Error processing audio: {:?}", e);
                        }
                    }

                    while let Ok((speaker, text, timestamp)) = rx.try_recv() {
                        callback(speaker, text, timestamp);
                    }

                    accum.clear();
                }
            }
        }
    });

    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device.build_output_stream(
            &config.into(),
            move |data: &mut [i16], _| {
                producer.push_slice(data);
            },
            move |err| eprintln!("Error in audio stream: {}", err),
            None,
        )?,
        cpal::SampleFormat::U16 => device.build_output_stream(
            &config.into(),
            move |data: &mut [u16], _| {
                let int_data: Vec<i16> = data
                    .iter()
                    .map(|&x| (x as i32 - (u16::MAX as i32 / 2)) as i16)
                    .collect();
                producer.push_slice(&int_data);
            },
            move |err| eprintln!("Error in audio stream: {}", err),
            None,
        )?,
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| {
                let int_data: Vec<i16> = data
                    .iter()
                    .map(|&x| (x * i16::MAX as f32) as i16)
                    .collect();
                producer.push_slice(&int_data);
            },
            move |err| eprintln!("Error in audio stream: {}", err),
            None,
        )?,
        _ => unreachable!("Unsupported sample format"),
    };

    stream.play()?;
    Ok(stream)
}