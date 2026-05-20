import * as vscode from 'vscode';

export class DiffViewProvider implements vscode.WebviewViewProvider {
    private view?: vscode.WebviewView;
    private diffContent = '';

    constructor() {}

    resolveWebviewView(
        webviewView: vscode.WebviewView,
        _context: vscode.WebviewViewResolveContext,
        _token: vscode.CancellationToken
    ) {
        this.view = webviewView;
        webviewView.webview.options = { enableScripts: true };
        webviewView.webview.html = this.getHtml();
    }

    setDiffContent(content: string) {
        this.diffContent = content;
        this.updateView();
    }

    private updateView() {
        if (!this.view) return;

        const lines = this.diffContent.split('\n').map(line => {
            let className = '';
            if (line.startsWith('+')) className = 'added';
            else if (line.startsWith('-')) className = 'removed';
            else if (line.startsWith('@@')) className = 'header';
            return `<div class="diff-line ${className}">${this.escapeHtml(line)}</div>`;
        }).join('');

        this.view.webview.postMessage({
            type: 'updateDiff',
            html: `<pre>${lines}</pre>`,
        });
    }

    private escapeHtml(text: string): string {
        return text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
    }

    private getHtml(): string {
        return `<!DOCTYPE html>
<html>
<head>
<style>
    body { font-family: 'Consolas', monospace; font-size: 12px; padding: 4px; background: #1e1e1e; color: #d4d4d4; }
    .diff-line { white-space: pre; padding: 1px 4px; }
    .diff-line.added { background: rgba(0,255,163,0.1); color: #00FFA3; }
    .diff-line.removed { background: rgba(255,68,68,0.1); color: #FF4444; }
    .diff-line.header { color: #8957FF; font-weight: bold; }
    pre { margin: 0; }
</style>
</head>
<body><div id="diff-container">Waiting for diff...</div>
<script>
    const vscode = acquireVsCodeApi();
    window.addEventListener('message', (event) => {
        if (event.data.type === 'updateDiff') {
            document.getElementById('diff-container').innerHTML = event.data.html;
        }
    });
</script>
</body>
</html>`;
    }
}
