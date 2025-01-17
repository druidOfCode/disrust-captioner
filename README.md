# disrust-captioner

Local, Offline **Captioning** & **Speaker Labeling** (in Rust), capturing Discord audio via loopback or a virtual audio device. 

> **Note**: We use **two** main components:
> 1. **pyannote-rs** (Segmentation + Speaker Identification) for **who** is talking  
> 2. **Whisper** (or another ASR) for **what** is being said  

## Table of Contents

1. [Overview](#overview)  
2. [Key Components](#key-components)  
3. [Setup & Installation](#setup--installation)  
4. [How It Works](#how-it-works)  
5. [Usage](#usage)  
6. [FAQ](#faq)  
7. [Roadmap](#roadmap)  

---

## 1. Overview

**disrust-captioner** aims to provide real-time (or near real-time) **captions** for voice chats—particularly Discord voice channels—while also labeling each speaker. Everything runs locally on your machine, **offline**, so you’re not tied to any cloud APIs or monthly fees.

- **Capture audio** from Discord via loopback  
- **Segment & label** each speaker (using pyannote-rs)  
- **Transcribe** the speaker’s words (using Whisper)  
- **Display** text lines: “**Alice:** I can’t hear you...”  
- **Rename** speakers on the fly, and persist those names for next session

---

## 2. Key Components

### A) pyannote-rs

[**pyannote-rs**](https://github.com/thewh1teagle/pyannote-rs) is used for **speaker diarization**—i.e., determining:

1. **When** speech occurs (using the `segmentation-3.0.onnx` model)  
2. **Who** is talking (using the `wespeaker_en_voxceleb_CAM++.onnx` model to generate speaker embeddings and comparing them with cosine similarity)

**Important**: pyannote-rs **does not** produce text. It only detects speech segments and identifies the speaker ID.

### B) Whisper (or Another ASR)

[**Whisper**](https://github.com/openai/whisper) (or `whisper.cpp`, `whisper-rs`) is used for **transcription**—i.e., figuring out **what** each speaker says. You need **an ASR engine** to produce the actual words for the captions. 

**Key point**: 
- **WeSpeaker** (from pyannote-rs) handles **who** is talking, not **what**.  
- **Whisper** handles **what** is being said.

---

## 3. Setup & Installation

### Prerequisites

- **Rust** (1.65 or newer)  
- **cpal** or a similar Rust library for audio capture (supports WASAPI loopback on Windows)  
- **pyannote-rs** for speaker diarization  
- **whisper.cpp**, **whisper-rs**, or the official Whisper Python binding (though we aim for a Rust-only solution)  
- A **loopback or virtual audio device** so we can capture Discord’s output

### Get the ONNX Models

Download the pyannote-rs models from [their releases](https://github.com/thewh1teagle/pyannote-rs/releases):

```bash
wget https://github.com/thewh1teagle/pyannote-rs/releases/download/v0.1.0/segmentation-3.0.onnx
wget https://github.com/thewh1teagle/pyannote-rs/releases/download/v0.1.0/wespeaker_en_voxceleb_CAM++.onnx
```

Also set up your **Whisper** model files (e.g., `ggml-base.bin` or `ggml-small.bin`, etc.) if you’re using `whisper.cpp`.

### Build & Run

In your project (`Cargo.toml`), ensure you have dependencies for `pyannote-rs`, your chosen ASR library (like `whisper-rs`), and your GUI framework (`egui`, `Iced`, etc.). Then:

```bash
cargo build --release
cargo run --release
```

---

## 4. How It Works

### 1) Audio Loopback

- We use **cpal** or a similar crate to capture the system’s output audio. On Windows with WASAPI, you can open the default output device in loopback mode.  
- Alternatively, use a virtual audio cable: set Discord to output to the cable, and capture that cable as an input.

### 2) Segment & Label Speakers (pyannote-rs)

- **segmentation-3.0** tells us the time ranges in which speech occurs.  
- For each speech segment, we use the **wespeaker** model to produce an **embedding**.  
- We compare embeddings to decide if it’s a previously known speaker or a new speaker (`person1`, `person2`, etc.). 

### 3) Transcription (Whisper)

- In parallel, for each speech segment (or for the entire chunk of audio that contains speech), we run **Whisper** to get the text.  
- We associate that text with the speaker label returned by pyannote-rs.

### 4) Display & Renaming

- We then show lines like:  
  - **person1 (0.0s - 2.0s):** “Hey guys, can you hear me?”  
  - **person2 (2.0s - 5.0s):** “Yes, I can!”  
- The UI allows you to rename `person1` → “Alice,” `person2` → “Bob.” This is saved in a JSON file so you don’t have to rename them every time.

---

## 5. Usage

1. **Launch disrust-captioner**.  
2. **Select the loopback device** from the app’s dropdown (or pass a CLI argument, depending on how you design it).  
3. Join a Discord voice channel; audio from your friends should be captured via loopback.  
4. As they speak, the app segments, identifies who is speaking (person1, person2, etc.), and transcribes.  
5. You see lines appear in the UI with speaker labels and text.  
6. Click on a speaker label to rename them if desired.  

---

## 6. FAQ

**Q1. Can’t pyannote-rs transcribe text too, via WeSpeaker?**  
A: **No.** WeSpeaker does **speaker embeddings**—_who_ is speaking, not _what_ they’re saying. You still need a dedicated **ASR engine** (e.g., Whisper) for transcription.

**Q2. Will it work on macOS or Linux?**  
A: The core logic is OS-independent, but capturing loopback audio might require platform-specific configs. Linux has `pulse` or `pipewire` loopback modules; macOS might need [BlackHole](https://github.com/ExistentialAudio/BlackHole) or similar.  

**Q3. What about overlapping speakers?**  
A: pyannote-rs can detect overlapping segments. We do our best, but perfect overlap labeling is still tricky. The UI might list partial overlaps if two people speak simultaneously.

**Q4. Is this truly real-time?**  
A: There’s a slight delay because we typically buffer 1–2 seconds of audio (or up to ~10 seconds in some diarization setups). For most conversational use, that’s acceptable.

---

## 7. Roadmap

1. **Always-On-Top Window / Overlay**  
   - Currently, we just show a small Rust GUI you can place beside Discord. We plan to explore a game-style overlay later.
2. **Optimized Chunking**  
   - Fine-tune how we chunk audio for segmentation and transcription, reducing delay and CPU/GPU load.
3. **Speaker Database**  
   - Save embeddings + names so next time your friend joins, we instantly recognize them.
4. **Cross-Platform Support**  
   - Provide clear instructions for loopback on macOS and Linux.

---

**Enjoy your local, offline, speaker-labeled captions!** If you have any issues, suggestions, or improvements, open a pull request or file an issue in this repository. 

_**Remember**: pyannote-rs + WeSpeaker = who’s speaking; Whisper = what they’re saying._
