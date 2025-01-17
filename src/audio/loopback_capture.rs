use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn initialize_audio_device() -> Result<cpal::Device, cpal::BuildStreamError> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to find default output device");
    Ok(device)
}

pub fn start_audio_capture(
    device: cpal::Device,
    diarization: Arc<Mutex<pyannote_rs::Diarization>>,
    transcription: Arc<Mutex<whisper_rs::Transcription>>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let config = device.default_input_format()?;
    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut diarization = diarization.lock().unwrap();
            let mut transcription = transcription.lock().unwrap();
            diarization.process_audio(data);
            transcription.process_audio(data);
        },
        move |err| {
            eprintln!("Error occurred on stream: {}", err);
        },
    )?;
    stream.play()?;
    Ok(stream)
}
