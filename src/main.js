const { invoke } = window.__TAURI__.core;

// UI Elements
let toggleRecordingBtn;
let toggleRecordingText;
let recordingIndicator;
let transcriptEl;
let statusMessageEl;
let audioDeviceSelect;
let micSourceBtn;
let systemSourceBtn;

// State
let isRecording = false;
let silenceTimer = null;
let silenceTimeout = 2000; // 2 seconds of silence before transcribing
let lastAudioLevel = 0;
let audioLevelCheckInterval = null;
let isSystemAudio = false; // Track if we're using system audio

// Initialize the application
window.addEventListener("DOMContentLoaded", () => {
  // Get UI elements
  toggleRecordingBtn = document.querySelector("#toggle-recording");
  toggleRecordingText = toggleRecordingBtn.querySelector(".button-text");
  recordingIndicator = toggleRecordingBtn.querySelector(".recording-indicator");
  transcriptEl = document.querySelector("#transcript");
  statusMessageEl = document.querySelector("#status-message");
  audioDeviceSelect = document.querySelector("#audio-device");
  micSourceBtn = document.querySelector("#mic-source");
  systemSourceBtn = document.querySelector("#system-source");
  
  // Set up event listeners
  toggleRecordingBtn.addEventListener("click", toggleRecording);
  audioDeviceSelect.addEventListener("change", handleDeviceChange);
  micSourceBtn.addEventListener("click", () => setAudioSource('microphone'));
  systemSourceBtn.addEventListener("click", () => setAudioSource('system'));
  
  // Populate audio devices
  populateAudioDevices();
  
  // Show welcome message
  appendTranscript("Welcome to Disrust Captioner! Click 'Start Recording' to begin capturing audio.", true);
});

// Set audio source (microphone or system)
async function setAudioSource(source) {
  if (isRecording) {
    // Stop recording before changing source
    await stopRecording();
  }
  
  isSystemAudio = source === 'system';
  
  // Update UI
  if (isSystemAudio) {
    micSourceBtn.classList.remove('active');
    systemSourceBtn.classList.add('active');
    showStatusMessage("System audio mode activated. Make sure you have the proper virtual audio device set up.");
  } else {
    systemSourceBtn.classList.remove('active');
    micSourceBtn.classList.add('active');
    showStatusMessage("Microphone mode activated.");
  }
  
  // Tell the backend which source to use
  try {
    await invoke("set_audio_source", { isSystem: isSystemAudio });
  } catch (error) {
    console.error("Failed to set audio source:", error);
    showErrorMessage(`Failed to set audio source: ${error}`);
  }
  
  // Clear status after 3 seconds
  setTimeout(clearStatusMessage, 3000);
}

// Populate audio device dropdown
async function populateAudioDevices() {
  try {
    const devices = await invoke("get_input_devices");
    
    // Clear existing options except the default
    while (audioDeviceSelect.options.length > 1) {
      audioDeviceSelect.remove(1);
    }
    
    // Add devices to dropdown
    devices.forEach(device => {
      const option = document.createElement("option");
      option.value = device.id;
      option.textContent = device.name;
      audioDeviceSelect.appendChild(option);
    });
    
    // Select the first device
    if (devices.length > 0) {
      audioDeviceSelect.value = devices[0].id;
    }
  } catch (error) {
    console.error("Failed to get input devices:", error);
    showErrorMessage("Failed to get audio devices. Using default device.");
  }
}

// Toggle recording state
async function toggleRecording() {
  if (isRecording) {
    await stopRecording();
  } else {
    await startRecording();
  }
}

// Start recording
async function startRecording() {
  try {
    clearStatusMessage();
    
    // Check if we need to set up system audio
    if (isSystemAudio) {
      showStatusMessage("Starting system audio capture...");
      await invoke("start_recording_system");
    } else {
      await invoke("start_recording");
    }
    
    // Update UI
    isRecording = true;
    toggleRecordingBtn.classList.add("active");
    toggleRecordingText.textContent = "Stop Recording";
    
    // Start checking for silence
    startSilenceDetection();
    
    console.log("Recording started");
  } catch (error) {
    showErrorMessage(`Failed to start recording: ${error}`);
    console.error("Failed to start recording:", error);
  }
}

// Handle device selection change
async function handleDeviceChange() {
  const deviceId = audioDeviceSelect.value;
  
  try {
    // If "default" is selected, pass null to use the system default
    const selectedId = deviceId === "default" ? null : deviceId;
    await invoke("set_input_device", { deviceId: selectedId });
    
    // If we're currently recording, restart the recording with the new device
    if (isRecording) {
      await stopRecording(false); // Stop without updating UI
      await startRecording();
    }
  } catch (error) {
    console.error("Failed to set input device:", error);
    showErrorMessage(`Failed to set input device: ${error}`);
  }
}

// Stop recording and transcribe
async function stopRecording(updateUI = true) {
  try {
    // Stop silence detection
    stopSilenceDetection();
    
    // Update UI if requested
    if (updateUI) {
      isRecording = false;
      toggleRecordingBtn.classList.remove("active");
      toggleRecordingText.textContent = "Start Recording";
      
      // Show processing message
      showStatusMessage("Processing audio...");
    }
    
    // Get transcription
    let transcript;
    if (isSystemAudio) {
      transcript = await invoke("stop_recording_system");
    } else {
      transcript = await invoke("stop_recording");
    }
    
    // Update transcript display if we got meaningful text and UI update is requested
    if (updateUI && transcript && !transcript.includes("[BLANK_AUDIO]")) {
      appendTranscript(transcript);
      
      // Clear status message
      clearStatusMessage();
    }
    
    console.log("Recording stopped");
    return transcript;
  } catch (error) {
    if (updateUI) {
      showErrorMessage(`Failed to stop recording: ${error}`);
    }
    console.error("Failed to stop recording:", error);
    throw error;
  }
}

// Start silence detection to automatically transcribe during gaps
function startSilenceDetection() {
  // This is a mock implementation since we can't directly measure audio levels from JS
  // In a real implementation, we would need to periodically check audio levels from Rust
  
  // For now, we'll just set a timer to transcribe every 10 seconds
  silenceTimer = setInterval(async () => {
    if (isRecording) {
      console.log("Detected silence, transcribing...");
      
      // Temporarily stop recording
      isRecording = false;
      
      try {
        // Get transcription without updating UI
        const transcript = await stopRecording(false);
        
        // If we got a meaningful transcript, display it
        if (transcript && !transcript.includes("No speech detected") && 
            !transcript.includes("Audio too short") && 
            !transcript.includes("[BLANK_AUDIO]")) {
          appendTranscript(transcript);
        }
        
        // Resume recording
        if (isSystemAudio) {
          await invoke("start_recording_system");
        } else {
          await invoke("start_recording");
        }
        isRecording = true;
      } catch (error) {
        console.error("Error during automatic transcription:", error);
        // Try to resume recording
        try {
          if (isSystemAudio) {
            await invoke("start_recording_system");
          } else {
            await invoke("start_recording");
          }
          isRecording = true;
        } catch (resumeError) {
          // If we can't resume, update UI to stopped state
          toggleRecordingBtn.classList.remove("active");
          toggleRecordingText.textContent = "Start Recording";
          showErrorMessage("Recording stopped due to an error");
        }
      }
    }
  }, 10000); // Check every 10 seconds
}

// Stop silence detection
function stopSilenceDetection() {
  if (silenceTimer) {
    clearInterval(silenceTimer);
    silenceTimer = null;
  }
  
  if (audioLevelCheckInterval) {
    clearInterval(audioLevelCheckInterval);
    audioLevelCheckInterval = null;
  }
}

// Append text to the transcript with proper formatting
function appendTranscript(text, isSystem = false) {
  if (!text || text.trim() === "") return;
  
  // Format timestamp
  const now = new Date();
  const timestamp = `[${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}]`;
  
  // Add new transcript with timestamp
  const newEntry = document.createElement("div");
  newEntry.className = "transcript-entry";
  
  const timestampSpan = document.createElement("span");
  timestampSpan.className = "timestamp";
  timestampSpan.textContent = timestamp;
  
  const textSpan = document.createElement("span");
  textSpan.className = isSystem ? "system-message" : "transcript-text";
  textSpan.textContent = text;
  
  newEntry.appendChild(timestampSpan);
  newEntry.appendChild(textSpan);
  
  // Add to transcript
  transcriptEl.appendChild(newEntry);
  
  // Scroll to bottom
  transcriptEl.scrollTop = transcriptEl.scrollHeight;
}

// Show error message
function showErrorMessage(message) {
  statusMessageEl.textContent = message;
  statusMessageEl.className = "status-message error";
}

// Show status message
function showStatusMessage(message) {
  statusMessageEl.textContent = message;
  statusMessageEl.className = "status-message success";
}

// Clear status message
function clearStatusMessage() {
  statusMessageEl.textContent = "";
  statusMessageEl.className = "status-message";
}
