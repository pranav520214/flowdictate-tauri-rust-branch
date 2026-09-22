/**
 * FlowDictate — Overlay UI Logic
 *
 * Vanilla JS — no framework needed for this simple overlay.
 * Communicates with the Rust backend via Tauri IPC.
 */

// State machine
const STATES = {
    IDLE: 'idle',
    LISTENING: 'listening',
    PROCESSING: 'processing',
    SUCCESS: 'success',
    WARNING: 'warning',
    SECURITY: 'security',
};

const STATE_LABELS = {
    [STATES.IDLE]: 'Ready',
    [STATES.LISTENING]: 'Listening',
    [STATES.PROCESSING]: 'Processing',
    [STATES.SUCCESS]: 'Done',
    [STATES.WARNING]: 'Warning',
    [STATES.SECURITY]: 'Alert',
};

let currentState = STATES.IDLE;

const overlay = document.getElementById('overlay');
const stateLabel = document.querySelector('.state-label');
const waveformCanvas = document.getElementById('waveform');
const transcriptEl = document.getElementById('transcript');
const ctx = waveformCanvas.getContext('2d');

/**
 * Update the UI state.
 * @param {string} newState - One of STATES values
 * @param {string} [transcript] - Optional partial transcript text
 * @param {string} [textState='raw'] - 'raw', 'purifying', or 'purified'
 */
function setState(newState, transcript, textState = 'raw') {
    currentState = newState;
    overlay.dataset.state = newState;
    stateLabel.textContent = STATE_LABELS[newState] || 'Ready';

    if (transcript !== undefined) {
        transcriptEl.textContent = transcript;
        transcriptEl.className = 'transcript';
        if (textState === 'raw') transcriptEl.classList.add('text-raw');
        else if (textState === 'purifying') transcriptEl.classList.add('text-purifying');
        else if (textState === 'purified') transcriptEl.classList.add('text-purified');
    }
}

/**
 * Draw real waveform data on the canvas.
 * This renders actual microphone levels — not decorative fake waveforms.
 * @param {Float32Array} levels - Audio level samples
 */
function drawWaveform(levels) {
    const w = waveformCanvas.width;
    const h = waveformCanvas.height;
    const mid = h / 2;

    ctx.clearRect(0, 0, w, h);
    ctx.strokeStyle = '#22c55e';
    ctx.lineWidth = 1.5;
    ctx.beginPath();

    for (let i = 0; i < levels.length; i++) {
        const x = (i / levels.length) * w;
        const y = mid + levels[i] * mid;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
    }

    ctx.stroke();
}

/**
 * Clear the waveform display.
 */
function clearWaveform() {
    ctx.clearRect(0, 0, waveformCanvas.width, waveformCanvas.height);
}

// Initialize with idle state
setState(STATES.IDLE);

// TODO(milestone-5): Wire up Tauri event listeners for state changes
// TODO(milestone-5): Wire up real-time waveform data from backend
// TODO(milestone-5): Wire up partial transcript updates
