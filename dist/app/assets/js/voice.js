class VoiceManager {
    constructor() {
        this.isListening = false;
        this.recognition = null;
        this.finalTranscript = '';
        this.interimTranscript = '';
        this.supported = false;
    }

    init() {
        const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
        if (!SpeechRecognition) {
            console.warn('Speech recognition not supported');
            return;
        }

        this.supported = true;
        this.recognition = new SpeechRecognition();
        this.recognition.continuous = true;
        this.recognition.interimResults = true;
        this.recognition.lang = 'en-US';

        this.recognition.onresult = (event) => {
            this.interimTranscript = '';
            this.finalTranscript = '';

            for (let i = event.resultIndex; i < event.results.length; i++) {
                const transcript = event.results[i][0].transcript;
                if (event.results[i].isFinal) {
                    this.finalTranscript += transcript;
                } else {
                    this.interimTranscript += transcript;
                }
            }

            const text = this.finalTranscript || this.interimTranscript;
            if (text) {
                this.onTranscript(text);
            }
        };

        this.recognition.onerror = (event) => {
            console.error('Speech recognition error:', event.error);
            if (event.error === 'not-allowed') {
                this.stop();
            }
        };

        this.recognition.onend = () => {
            if (this.isListening) {
                this.recognition.start();
            }
        };
    }

    toggle() {
        if (this.isListening) {
            this.stop();
        } else {
            this.start();
        }
    }

    start() {
        if (!this.supported || !this.recognition) {
            console.warn('Speech recognition not available');
            this.onUnsupported();
            return;
        }

        try {
            this.recognition.start();
            this.isListening = true;
            this.onStart();

            const indicator = document.getElementById('voice-indicator');
            if (indicator) indicator.style.display = 'inline';
        } catch (e) {
            console.error('Failed to start voice recognition:', e);
        }
    }

    stop() {
        if (this.recognition) {
            try {
                this.recognition.stop();
            } catch (e) {}
        }
        this.isListening = false;
        this.onStop();

        const indicator = document.getElementById('voice-indicator');
        if (indicator) indicator.style.display = 'none';
    }

    onStart() {
        if (window.app && window.app.showToast) {
            window.app.showToast('Voice input started');
        }
    }
    onStop() {
        if (window.app && window.app.showToast) {
            window.app.showToast('Voice input stopped');
        }
    }
    onTranscript(text) {
        const input = document.getElementById('chat-input');
        if (input) {
            input.value = text;
            input.focus();
        }
    }
    onUnsupported() {
        if (window.app && window.app.showToast) {
            window.app.showToast('Voice not supported in this browser');
        }
    }
}

export default VoiceManager;
