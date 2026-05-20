import * as vscode from 'vscode';

export interface WsMessage {
    type: string;
    content?: string;
    path?: string;
    timestamp?: string;
    [key: string]: unknown;
}

export class OmniClient {
    private ws: WebSocket | undefined;
    private url: string;
    private reconnectAttempts = 0;
    private maxRetries = 10;
    private listeners: Map<string, Array<(data: WsMessage) => void>> = new Map();
    private connected = false;
    private messageQueue: string[] = [];

    constructor(url: string) {
        this.url = url;
    }

    connect() {
        try {
            this.ws = new WebSocket(this.url);
        } catch (e) {
            console.error('WebSocket creation failed:', e);
            this.scheduleReconnect();
            return;
        }

        this.ws.onopen = () => {
            this.connected = true;
            this.reconnectAttempts = 0;
            this.flushQueue();
            vscode.window.setStatusBarMessage('$(link) OmniCode Connected', 3000);
            this.emit('connected', { type: 'connected' });
        };

        this.ws.onmessage = (event) => {
            try {
                const msg: WsMessage = JSON.parse(event.data.toString());
                this.emit(msg.type, msg);
            } catch (e) {
                console.warn('Failed to parse WS message:', e);
            }
        };

        this.ws.onclose = () => {
            this.connected = false;
            this.emit('disconnected', { type: 'disconnected' });
            this.scheduleReconnect();
        };

        this.ws.onerror = (err) => {
            console.error('WebSocket error:', err);
            vscode.window.setStatusBarMessage('$(warning) OmniCode Connection Error', 5000);
        };
    }

    private scheduleReconnect() {
        if (this.reconnectAttempts >= this.maxRetries) return;
        const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
        this.reconnectAttempts++;
        setTimeout(() => this.connect(), delay);
    }

    send(message: WsMessage) {
        const msg = JSON.stringify({
            ...message,
            timestamp: new Date().toISOString(),
        });

        if (this.connected && this.ws?.readyState === WebSocket.OPEN) {
            this.ws.send(msg);
        } else {
            this.messageQueue.push(msg);
        }
    }

    private flushQueue() {
        while (this.messageQueue.length > 0) {
            const msg = this.messageQueue.shift()!;
            if (this.ws?.readyState === WebSocket.OPEN) {
                this.ws.send(msg);
            }
        }
    }

    on(event: string, callback: (data: WsMessage) => void) {
        if (!this.listeners.has(event)) {
            this.listeners.set(event, []);
        }
        this.listeners.get(event)!.push(callback);
    }

    private emit(event: string, data: WsMessage) {
        const handlers = this.listeners.get(event);
        if (handlers) {
            handlers.forEach(cb => cb(data));
        }
    }

    disconnect() {
        if (this.ws) {
            this.ws.close();
            this.ws = undefined;
        }
        this.connected = false;
    }

    isConnected(): boolean {
        return this.connected;
    }
}
