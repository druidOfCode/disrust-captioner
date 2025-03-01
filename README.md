# Disrust Captioner

A Discord-inspired live captioning tool for voice calls and meetings. This application captures audio from your microphone or system audio and provides real-time transcription using the Whisper speech recognition model.

![Disrust Captioner Screenshot](screenshot.png)

## Features

- **Live Captioning**: Real-time transcription of speech with timestamps
- **Discord-Inspired UI**: Clean, modern interface with Discord's color scheme
- **Multiple Audio Sources**: 
  - Microphone input for your own voice
  - System audio capture for Discord calls and other applications
- **Device Selection**: Choose from available audio input devices
- **Automatic Transcription**: Periodically transcribes during conversation gaps
- **Voice Activity Detection**: Filters out silence for better transcription quality

## Setup Instructions

### Prerequisites

- Rust and Cargo installed
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites/) installed
- FFmpeg installed (required for audio processing)

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/disrust-captioner.git
   cd disrust-captioner
   ```

2. Install dependencies:
   ```
   cargo install tauri-cli
   ```

3. Build and run the application:
   ```
   cargo tauri dev
   ```

### System Audio Capture Setup

To capture system audio, you'll need to set up a virtual audio device:

#### macOS

1. Install [BlackHole](https://existential.audio/blackhole/):
   ```
   brew install blackhole-2ch
   ```

2. Open "Audio MIDI Setup" from Applications/Utilities
3. Create a Multi-Output Device:
   - Click the "+" button in the bottom left corner
   - Select "Create Multi-Output Device"
   - Check both "Built-in Output" and "BlackHole 2ch"
4. Set the Multi-Output Device as your default output in System Preferences > Sound

#### Windows

1. Install [VB-Cable](https://vb-audio.com/Cable/)
2. Set VB-Cable as your default playback device
3. In Discord, set the output device to VB-Cable

#### Linux

1. Use PulseAudio's built-in loopback module:
   ```
   pactl load-module module-loopback latency_msec=1
   ```

## Usage

1. Launch the application
2. Select your audio input device from the dropdown
3. Choose between microphone or system audio
4. Click "Start Recording" to begin capturing and transcribing
5. The transcript will appear in the main panel with timestamps
6. Click "Stop Recording" to end the session

## Development

This application is built with:

- [Tauri](https://tauri.app/) - Desktop application framework
- [Rust](https://www.rust-lang.org/) - Backend language
- [Whisper](https://github.com/openai/whisper) - Speech recognition model
- HTML/CSS/JavaScript - Frontend

## License

MIT License

## Changelog

### v0.2.0 (Current)
- Added Discord-inspired UI
- Implemented system audio capture
- Added audio source selection
- Improved transcript display with timestamps
- Enhanced error handling and user feedback

### v0.1.0
- Initial release
- Basic microphone capture and transcription
- Simple UI with recording controls
