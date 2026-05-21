class SyncManager {
    constructor(url) {
        this.url = url || (window.app ? window.app.state.serverUrl : 'ws://localhost:9421');
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 3;
        this.messageQueue = [];
        this.handlers = {};
        this.connected = false;
        this._tryConnect();
    }

    _tryConnect() {
        try {
            this.ws = new WebSocket(this.url);
        } catch (e) {
            console.warn('Sync server not available — running in offline mode');
            return;
        }

        this.ws.onopen = () => {
            this.connected = true;
            this.reconnectAttempts = 0;
            this.flushQueue();
            this.emit('connected');
        };

        this.ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data);
                this.emit(msg.type, msg);
            } catch (e) {}
        };

        this.ws.onclose = () => {
            this.connected = false;
            this.emit('disconnected');
            this._scheduleReconnect();
        };

        this.ws.onerror = () => {
            console.warn('Sync server not available — running in offline mode');
            this.connected = false;
        };
    }

    _scheduleReconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) return;
        const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
        this.reconnectAttempts++;
        setTimeout(() => this._tryConnect(), delay);
    }



    send(message) {
        const msg = JSON.stringify({
            ...message,
            timestamp: message.timestamp || new Date().toISOString(),
        });

        if (this.connected && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(msg);
        } else {
            this.messageQueue.push(msg);
        }
    }

    flushQueue() {
        while (this.messageQueue.length > 0) {
            const msg = this.messageQueue.shift();
            if (this.ws.readyState === WebSocket.OPEN) {
                this.ws.send(msg);
            }
        }
    }

    on(event, handler) {
        if (!this.handlers[event]) this.handlers[event] = [];
        this.handlers[event].push(handler);
    }

    emit(event, data) {
        if (this.handlers[event]) {
            this.handlers[event].forEach(h => h(data));
        }
    }

    sendChatMessage(text) {
        this.send({ type: 'chat_message', content: text });
    }

    sendFileChange(path, content) {
        this.send({ type: 'file_change', path, content });
    }

    sendCursorUpdate(line, col) {
        this.send({ type: 'cursor_update', line, col });
    }

    disconnect() {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
        this.connected = false;
    }
}

export default SyncManager;
