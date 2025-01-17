use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use ringbuf::traits::Split;
use ringbuf::producer::Producer;
use ringbuf::consumer::Consumer;
use crate::diarization::pyannote::DiarizationBackend;
use crate::transcription::whisper_integration::TranscriptionBackend;
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
    diarization: Arc<Mutex<dyn DiarizationBackend + Send + Sync>>,
    transcription: Arc<Mutex<dyn TranscriptionBackend + Send + Sync>>,
    speaker_manager: Arc<Mutex<SpeakerManager>>,
    callback: impl Fn(String, String) + Send + 'static,
) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
    let config = device.default_output_config().map_err(Box::new)?;
    let sample_rate = config.sample_rate().0;
    let ring_buffer = ringbuf::HeapRb::<i16>::new(BUFFER_SIZE);  // Changed to i16
    let (mut producer, mut consumer) = ring_buffer.split();

    std::thread::spawn(move || -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = vec![0i16; PROCESS_INTERVAL];
        loop {
            let count = consumer.pop_slice(&mut buffer);
            if count >= PROCESS_INTERVAL {
                if let Ok(diar) = diarization.lock() {
                    if let Ok(segments) = diar.segment_audio(&buffer) {
                        for segment in segments {
                            let speaker_embedding = diar.embed_speaker(&segment.samples);
                            let speaker_id = speaker_manager.lock()
                                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
                                .identify_speaker(&speaker_embedding);

                            if let Ok(mut trans) = transcription.lock() {
                                let float_samples: Vec<f32> = segment.samples.iter().map(|&x| x as f32 / i16::MAX as f32).collect();
                                if let Ok(text) = trans.transcribe_audio(&float_samples, sample_rate) {
                                    callback(speaker_id, text);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    });

    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device.build_output_stream(
            &config.into(),
            move |data: &mut [i16], _| { 
                producer.push_slice(data);
            },
            move |err| eprintln!("Error in audio stream: {}", err),
            None,
        ),
        cpal::SampleFormat::U16 => device.build_output_stream(
            &config.into(),
            move |data: &mut [u16], _| {
                let int_data: Vec<i16> = data.iter()
                    .map(|&x| (x as i16 - i16::MAX / 2) * 2)
                    .collect();
                producer.push_slice(&int_data);
            },
            move |err| eprintln!("Error in audio stream: {}", err),
            None,
        ),
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| { 
                let int_data: Vec<i16> = data.iter()
                    .map(|&x| (x * i16::MAX as f32) as i16)
                    .collect();
                producer.push_slice(&int_data);
            },
            move |err| eprintln!("Error in audio stream: {}", err),
            None,
        ),
        _ => unreachable!("Unknown sample format"),
    }?;

    stream.play()?;
    Ok(stream)
}
