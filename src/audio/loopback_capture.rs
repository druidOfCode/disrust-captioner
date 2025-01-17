use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use ringbuf::{RingBuffer, Producer, Consumer};
use crate::diarization::speaker_manager::SpeakerManager;

const BUFFER_SIZE: usize = 1024 * 16;
const PROCESS_INTERVAL: usize = 4096; // Process every 4096 samples

pub fn initialize_audio_device(
    maybe_device: Option<cpal::Device>
) -> Result<cpal::Device, cpal::BuildStreamError> {
    let host = cpal::default_host();
    if let Some(dev) = maybe_device {
        return Ok(dev);
    }
    let device = host.default_output_device()
        .expect("Failed to find default output device");
    Ok(device)
}

pub fn start_audio_capture(
    device: cpal::Device,
    diarization: Arc<Mutex<PyannoteIntegration>>,
    transcription: Arc<Mutex<WhisperIntegration>>,
    speaker_manager: Arc<Mutex<SpeakerManager>>,
    callback: impl Fn(String, String) + Send + 'static,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let config = device.default_output_config()?;
    let ring_buffer = RingBuffer::<f32>::new(BUFFER_SIZE);
    let (mut producer, mut consumer) = ring_buffer.split();

    // Process audio in a separate thread
    std::thread::spawn(move || {
        let mut buffer = vec![0.0f32; PROCESS_INTERVAL];
        loop {
            if let Ok(count) = consumer.pop_slice(&mut buffer) {
                if count >= PROCESS_INTERVAL {
                    if let Ok(diar) = diarization.lock() {
                        if let Ok(segments) = diar.segment_audio(&buffer) {
                            // Process each segment
                            for segment in segments {
                                let speaker_embedding = diar.embed_speaker(&buffer[segment.start..segment.end]);
                                let speaker_id = speaker_manager.lock()
                                    .unwrap()
                                    .identify_speaker(&speaker_embedding)
                                    .unwrap_or_else(|| format!("Speaker_{}", segment.speaker_id));

                                if let Ok(trans) = transcription.lock() {
                                    if let Ok(text) = trans.transcribe_audio(&buffer[segment.start..segment.end]) {
                                        callback(speaker_id, text);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::OutputCallbackInfo| {
            producer.push_slice(data);
        },
        move |err| eprintln!("Error in audio stream: {}", err),
    )?;

    stream.play()?;
    Ok(stream)
}
