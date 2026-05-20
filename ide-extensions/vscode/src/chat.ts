import * as vscode from 'vscode';
import { OmniClient } from './client';

export class ChatSidebarProvider implements vscode.WebviewViewProvider {
    private view?: vscode.WebviewView;
    private messages: Array<{ role: string; content: string }> = [];

    constructor(
        private readonly extensionUri: vscode.Uri,
        private readonly client: OmniClient
    ) {
        client.on('chat_message', (msg) => {
            this.addMessage('agent', msg.content || '');
        });

        client.on('agent_status', (msg) => {
            this.addMessage('system', `Status: ${msg.content}`);
        });
    }

    resolveWebviewView(
        webviewView: vscode.WebviewView,
        _context: vscode.WebviewViewResolveContext,
        _token: vscode.CancellationToken
    ) {
        this.view = webviewView;
        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [this.extensionUri],
        };

        webviewView.webview.html = this.getHtml();

        webviewView.webview.onDidReceiveMessage((message) => {
            if (message.type === 'chat') {
                this.addMessage('user', message.text);
                this.client.send({ type: 'chat_message', content: message.text });
            }
        });
    }

    private addMessage(role: string, content: string) {
        this.messages.push({ role, content });
        this.updateView();
    }

    private updateView() {
        if (!this.view) return;
        const html = this.messages.map(m => `
            <div class="message ${m.role}">
                <div class="header">${m.role === 'user' ? '🧑 You' : m.role === 'agent' ? '🤖 OmniCode' : '⚙️ System'}</div>
                <div class="content">${this.escapeHtml(m.content)}</div>
            </div>
        `).join('');

        this.view.webview.postMessage({ type: 'update', html });
    }

    private escapeHtml(text: string): string {
        return text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/\n/g, '<br>');
    }

    private getHtml(): string {
        return `<!DOCTYPE html>
<html>
<head>
<style>
    body { font-family: 'Segoe UI', sans-serif; padding: 8px; color: #ccc; background: #1e1e1e; }
    .message { margin: 8px 0; padding: 8px; border-radius: 4px; }
    .message.user { background: #2d2d2d; border-left: 3px solid #00FFA3; }
    .message.agent { background: #2d2d2d; border-left: 3px solid #8957FF; }
    .message.system { background: #252525; border-left: 3px solid #8899B4; font-style: italic; }
    .header { font-size: 11px; color: #888; margin-bottom: 4px; }
    .content { font-size: 13px; line-height: 1.4; }
    #input-area { display: flex; gap: 4px; margin-top: 12px; }
    #chat-input { flex: 1; background: #3c3c3c; border: 1px solid #555; color: #ccc; padding: 6px; border-radius: 4px; }
    #send-btn { background: #8957FF; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; }
</style>
</head>
<body>
    <div id="messages"></div>
    <div id="input-area">
        <input type="text" id="chat-input" placeholder="Ask OmniCode...">
        <button id="send-btn">Send</button>
    </div>
    <script>
        const vscode = acquireVsCodeApi();
        document.getElementById('send-btn').addEventListener('click', () => {
            const input = document.getElementById('chat-input');
            if (input.value.trim()) {
                vscode.postMessage({ type: 'chat', text: input.value });
                input.value = '';
            }
        });
        document.getElementById('chat-input').addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                document.getElementById('send-btn').click();
            }
        });
        window.addEventListener('message', (event) => {
            const msg = event.data;
            if (msg.type === 'update') {
                document.getElementById('messages').innerHTML = msg.html;
            }
        });
    </script>
</body>
</html>`;
    }
}
