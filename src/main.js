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
let diarizationToggle;
let speakerRenameModal;
let speakerRenameForm;
let speakerRenameInput;
let speakerRenameSubmit;
let speakerRenameCancel;

// State
let isRecording = false;
let silenceTimer = null;
let silenceTimeout = 2000; // 2 seconds of silence before transcribing
let lastAudioLevel = 0;
let audioLevelCheckInterval = null;
let isSystemAudio = false; // Track if we're using system audio
let useDiarization = false; // Track if diarization is enabled
let currentSpeakers = new Map(); // Map to store speaker names

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
  diarizationToggle = document.querySelector("#diarization-toggle");
  
  // Create speaker rename modal elements
  createSpeakerRenameModal();
  
  // Set up event listeners
  toggleRecordingBtn.addEventListener("click", toggleRecording);
  audioDeviceSelect.addEventListener("change", handleDeviceChange);
  micSourceBtn.addEventListener("change", () => setAudioSource('microphone'));
  systemSourceBtn.addEventListener("change", () => setAudioSource('system'));
  diarizationToggle.addEventListener("change", toggleDiarization);
  
  // Populate audio devices
  populateAudioDevices();
  
  // Show welcome message
  appendTranscript("Welcome to Disrust Captioner! Click 'Start Recording' to begin capturing audio.", true);
});

// Create speaker rename modal
function createSpeakerRenameModal() {
  // Create modal container
  speakerRenameModal = document.createElement('div');
  speakerRenameModal.className = 'modal';
  speakerRenameModal.id = 'speaker-rename-modal';
  speakerRenameModal.style.display = 'none';
  
  // Create modal content
  const modalContent = document.createElement('div');
  modalContent.className = 'modal-content';
  
  // Create form
  speakerRenameForm = document.createElement('form');
  speakerRenameForm.id = 'speaker-rename-form';
  
  // Create title
  const title = document.createElement('h2');
  title.textContent = 'Rename Speaker';
  
  // Create input
  const inputLabel = document.createElement('label');
  inputLabel.textContent = 'New Name:';
  inputLabel.htmlFor = 'speaker-name-input';
  
  speakerRenameInput = document.createElement('input');
  speakerRenameInput.type = 'text';
  speakerRenameInput.id = 'speaker-name-input';
  speakerRenameInput.required = true;
  
  // Create buttons
  const buttonContainer = document.createElement('div');
  buttonContainer.className = 'button-container';
  
  speakerRenameSubmit = document.createElement('button');
  speakerRenameSubmit.type = 'submit';
  speakerRenameSubmit.textContent = 'Save';
  speakerRenameSubmit.className = 'primary-button';
  
  speakerRenameCancel = document.createElement('button');
  speakerRenameCancel.type = 'button';
  speakerRenameCancel.textContent = 'Cancel';
  speakerRenameCancel.className = 'secondary-button';
  
  // Assemble modal
  buttonContainer.appendChild(speakerRenameSubmit);
  buttonContainer.appendChild(speakerRenameCancel);
  
  speakerRenameForm.appendChild(title);
  speakerRenameForm.appendChild(inputLabel);
  speakerRenameForm.appendChild(speakerRenameInput);
  speakerRenameForm.appendChild(buttonContainer);
  
  modalContent.appendChild(speakerRenameForm);
  speakerRenameModal.appendChild(modalContent);
  
  // Add to document
  document.body.appendChild(speakerRenameModal);
  
  // Set up event listeners
  speakerRenameForm.addEventListener('submit', handleSpeakerRename);
  speakerRenameCancel.addEventListener('click', () => {
    speakerRenameModal.style.display = 'none';
  });
  
  // Close modal when clicking outside
  window.addEventListener('click', (event) => {
    if (event.target === speakerRenameModal) {
      speakerRenameModal.style.display = 'none';
    }
  });
}

// Toggle diarization
async function toggleDiarization() {
  useDiarization = diarizationToggle.checked;
  
  if (useDiarization) {
    try {
      // Check what model is being used
      const modelStatus = await invoke("get_diarization_model_status");
      if (modelStatus === "advanced") {
        showStatusMessage("Advanced speaker diarization enabled (TitaNet model)");
      } else {
        showStatusMessage("Basic speaker diarization enabled (fallback model)");
      }
    } catch (error) {
      console.error("Failed to get model status:", error);
      showStatusMessage("Speaker diarization enabled");
    }
  } else {
    showStatusMessage("Speaker diarization disabled");
  }
  
  // Clear status after 3 seconds
  setTimeout(clearStatusMessage, 3000);
}

// Set audio source (microphone or system)
async function setAudioSource(source) {
  if (isRecording) {
    // Stop recording before changing source
    await stopRecording();
  }
  
  isSystemAudio = source === 'system';
  
  // Update UI
  if (isSystemAudio) {
    micSourceBtn.checked = false;
    systemSourceBtn.checked = true;
    document.querySelector('label[for="mic-source"]').classList.remove('active');
    document.querySelector('label[for="system-source"]').classList.add('active');
    showStatusMessage("System audio mode activated. Make sure you have the proper virtual audio device set up.");
  } else {
    systemSourceBtn.checked = false;
    micSourceBtn.checked = true;
    document.querySelector('label[for="system-source"]').classList.remove('active');
    document.querySelector('label[for="mic-source"]').classList.add('active');
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
    showErrorMessage(`Failed to get input devices: ${error}`);
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
    // Start recording based on selected source
    if (isSystemAudio) {
      await invoke("start_recording_system");
    } else {
      await invoke("start_recording");
    }
    
    // Update UI
    isRecording = true;
    toggleRecordingText.textContent = "Stop Recording";
    recordingIndicator.classList.add("active");
    
    // Start silence detection
    startSilenceDetection();
    
    showStatusMessage("Recording started...");
  } catch (error) {
    console.error("Failed to start recording:", error);
    showErrorMessage(`Failed to start recording: ${error}`);
  }
}

// Handle device change
async function handleDeviceChange() {
  const deviceId = audioDeviceSelect.value;
  
  try {
    // If recording, stop first
    if (isRecording) {
      await stopRecording();
    }
    
    // Set the selected device
    await invoke("set_input_device", { deviceId: deviceId === "default" ? null : deviceId });
    
    showStatusMessage(`Audio device changed to: ${audioDeviceSelect.options[audioDeviceSelect.selectedIndex].text}`);
    
    // Clear status after 3 seconds
    setTimeout(clearStatusMessage, 3000);
  } catch (error) {
    console.error("Failed to change audio device:", error);
    showErrorMessage(`Failed to change audio device: ${error}`);
  }
}

// Stop recording
async function stopRecording(updateUI = true) {
  try {
    // Stop silence detection
    stopSilenceDetection();
    
    // Get transcription based on selected source and diarization setting
    let transcript;
    if (isSystemAudio) {
      if (useDiarization) {
        transcript = await invoke("stop_recording_system_with_diarization");
      } else {
        transcript = await invoke("stop_recording_system");
      }
    } else {
      if (useDiarization) {
        transcript = await invoke("stop_recording_with_diarization");
      } else {
        transcript = await invoke("stop_recording");
      }
    }
    
    // Update UI
    if (updateUI) {
      isRecording = false;
      toggleRecordingText.textContent = "Start Recording";
      recordingIndicator.classList.remove("active");
    }
    
    // Process and display transcript
    if (transcript) {
      if (useDiarization) {
        appendDiarizedTranscript(transcript);
      } else {
        appendTranscript(transcript);
      }
    }
    
    return transcript;
  } catch (error) {
    console.error("Failed to stop recording:", error);
    showErrorMessage(`Failed to stop recording: ${error}`);
    
    // Update UI even on error
    if (updateUI) {
      isRecording = false;
      toggleRecordingText.textContent = "Start Recording";
      recordingIndicator.classList.remove("active");
    }
    
    return null;
  }
}

// Start silence detection
function startSilenceDetection() {
  // Clear any existing interval
  if (audioLevelCheckInterval) {
    clearInterval(audioLevelCheckInterval);
  }
  
  // Set up interval to check audio level
  audioLevelCheckInterval = setInterval(() => {
    // This is a placeholder for actual audio level detection
    // In a real implementation, you would get this from the audio stream
    const currentAudioLevel = Math.random(); // Simulate random audio level
    
    // Check if audio level is below threshold (silence)
    if (currentAudioLevel < 0.1) {
      // If we were previously not in silence, start the timer
      if (lastAudioLevel >= 0.1 && !silenceTimer) {
        console.log("Silence detected, starting timer...");
        silenceTimer = setTimeout(async () => {
          // If we're still recording, transcribe the current audio
          if (isRecording) {
            console.log("Silence timeout reached, processing audio...");
            // Stop recording temporarily without updating UI
            const transcript = await stopRecording(false);
            
            // If we got a transcript, restart recording
            if (transcript) {
              console.log("Transcript received, restarting recording");
              await startRecording();
            }
          }
          
          // Clear the timer
          silenceTimer = null;
        }, silenceTimeout);
      }
    } else {
      // If we're not in silence, clear any existing timer
      if (silenceTimer) {
        console.log("Speech detected, clearing silence timer");
        clearTimeout(silenceTimer);
        silenceTimer = null;
      }
    }
    
    // Update last audio level
    lastAudioLevel = currentAudioLevel;
  }, 100); // Check every 100ms
}

// Stop silence detection
function stopSilenceDetection() {
  // Clear interval
  if (audioLevelCheckInterval) {
    clearInterval(audioLevelCheckInterval);
    audioLevelCheckInterval = null;
  }
  
  // Clear timer
  if (silenceTimer) {
    clearTimeout(silenceTimer);
    silenceTimer = null;
  }
}

// Append transcript to the UI
function appendTranscript(text, isSystem = false) {
  // Create a new transcript entry
  const entry = document.createElement("div");
  entry.className = "transcript-entry";
  
  // Add timestamp
  const timestamp = document.createElement("div");
  timestamp.className = "timestamp";
  const now = new Date();
  timestamp.textContent = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`;
  
  // Add text
  const content = document.createElement("div");
  content.className = "content";
  content.textContent = text;
  
  // If this is a system message, add special styling
  if (isSystem) {
    entry.classList.add("system-message");
  }
  
  // Assemble entry
  entry.appendChild(timestamp);
  entry.appendChild(content);
  
  // Add to transcript
  transcriptEl.appendChild(entry);
  
  // Scroll to bottom
  transcriptEl.scrollTop = transcriptEl.scrollHeight;
}

// Append diarized transcript to the UI
function appendDiarizedTranscript(text) {
  // Split the text into lines
  const lines = text.split('\n');
  
  // Map to store speaker IDs to consistent color indices
  const speakerColorMap = new Map();
  let nextColorIndex = 0;
  
  // Process each line
  for (const line of lines) {
    if (line.trim() === '') continue;
    
    // Check if line has speaker format (Speaker: text)
    const match = line.match(/^([^:]+):\s*(.+)$/);
    if (match) {
      const speakerId = match[1].trim();
      const speakerText = match[2].trim();
      
      // Assign a consistent color index to this speaker
      if (!speakerColorMap.has(speakerId)) {
        speakerColorMap.set(speakerId, nextColorIndex % 8); // 8 colors available
        nextColorIndex++;
      }
      
      const colorIndex = speakerColorMap.get(speakerId);
      
      // Create a new transcript entry
      const entry = document.createElement("div");
      entry.className = "transcript-entry diarized";
      entry.dataset.colorIndex = colorIndex; // Store color index as data attribute
      
      // Add timestamp
      const timestamp = document.createElement("div");
      timestamp.className = "timestamp";
      const now = new Date();
      timestamp.textContent = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`;
      
      // Add speaker label
      const speaker = document.createElement("div");
      speaker.className = "speaker-label";
      speaker.style.color = `var(--discord-speaker${colorIndex + 1})`;
      speaker.textContent = speakerId;
      speaker.dataset.speakerId = speakerId;
      
      // Add rename button
      const renameBtn = document.createElement("button");
      renameBtn.className = "rename-button";
      renameBtn.textContent = "✏️";
      renameBtn.title = "Rename speaker";
      renameBtn.addEventListener("click", () => openSpeakerRenameModal(speakerId));
      
      // Add text
      const content = document.createElement("div");
      content.className = "content";
      content.textContent = speakerText;
      
      // Assemble entry
      speaker.appendChild(renameBtn);
      entry.appendChild(timestamp);
      entry.appendChild(speaker);
      entry.appendChild(content);
      
      // Add to transcript
      transcriptEl.appendChild(entry);
    } else {
      // Regular transcript line
      appendTranscript(line);
    }
  }
  
  // Scroll to bottom
  transcriptEl.scrollTop = transcriptEl.scrollHeight;
}

// Open speaker rename modal
function openSpeakerRenameModal(speakerId) {
  // Set current speaker ID as data attribute
  speakerRenameForm.dataset.speakerId = speakerId;
  
  // Set current name as default value
  const currentName = currentSpeakers.get(speakerId) || speakerId;
  speakerRenameInput.value = currentName;
  
  // Show modal
  speakerRenameModal.style.display = 'block';
  
  // Focus input
  speakerRenameInput.focus();
}

// Handle speaker rename form submission
async function handleSpeakerRename(event) {
  event.preventDefault();
  
  const speakerId = speakerRenameForm.dataset.speakerId;
  const newName = speakerRenameInput.value.trim();
  
  if (!newName) return;
  
  try {
    // Call backend to rename speaker
    const updatedSpeaker = await invoke("rename_speaker", { speakerId, newName });
    
    // Update local map
    currentSpeakers.set(speakerId, updatedSpeaker.name);
    
    // Update UI
    document.querySelectorAll(`.speaker-label[data-speaker-id="${speakerId}"]`).forEach(label => {
      label.textContent = updatedSpeaker.name;
      
      // Re-add the rename button
      const renameBtn = document.createElement("button");
      renameBtn.className = "rename-button";
      renameBtn.textContent = "✏️";
      renameBtn.title = "Rename speaker";
      renameBtn.addEventListener("click", () => openSpeakerRenameModal(speakerId));
      
      label.appendChild(renameBtn);
    });
    
    // Hide modal
    speakerRenameModal.style.display = 'none';
    
    showStatusMessage(`Renamed ${speakerId} to ${updatedSpeaker.name}`);
    
    // Clear status after 3 seconds
    setTimeout(clearStatusMessage, 3000);
  } catch (error) {
    console.error("Failed to rename speaker:", error);
    showErrorMessage(`Failed to rename speaker: ${error}`);
  }
}

// Show error message
function showErrorMessage(message) {
  statusMessageEl.textContent = message;
  statusMessageEl.classList.add("error");
  statusMessageEl.classList.add("visible");
}

// Show status message
function showStatusMessage(message) {
  statusMessageEl.textContent = message;
  statusMessageEl.classList.remove("error");
  statusMessageEl.classList.add("visible");
}

// Clear status message
function clearStatusMessage() {
  statusMessageEl.classList.remove("visible");
}
