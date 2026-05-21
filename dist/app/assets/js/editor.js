class EditorManager {
    constructor(containerId = 'monaco-editor') {
        this.containerId = containerId;
        this.editor = null;
        this.currentFile = null;
        this.content = '';
        this.debounceTimer = null;
        this.remoteCursors = [];
    }

    async init() {
        if (typeof monaco !== 'undefined') {
            this.createEditor();
            return;
        }

        const script = document.createElement('script');
        script.src = 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min/vs/loader.min.js';
        script.onload = () => {
            require.config({
                paths: { vs: 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min/vs' }
            });
            require(['vs/editor/editor.main'], () => this.createEditor());
        };
        document.head.appendChild(script);
    }

    createEditor() {
        if (!document.getElementById(this.containerId)) return;

        monaco.editor.defineTheme('omnicode-dark', {
            base: 'vs-dark',
            inherit: true,
            rules: [
                { token: 'comment', foreground: '8899B4', fontStyle: 'italic' },
                { token: 'keyword', foreground: '8957FF' },
                { token: 'string', foreground: '00FFA3' },
                { token: 'number', foreground: 'FFD700' },
                { token: 'type', foreground: '00AAFF' },
                { token: 'function', foreground: 'FFD700' },
            ],
            colors: {
                'editor.background': '#0F1520',
                'editor.foreground': '#EBF0F5',
                'editor.lineHighlightBackground': '#141C2B',
                'editor.selectionBackground': '#1E2A3A',
                'editorCursor.foreground': '#00FFA3',
                'editorLineNumber.foreground': '#8899B4',
            }
        });

        monaco.editor.defineTheme('omnicode-light', {
            base: 'vs',
            inherit: true,
            rules: [
                { token: 'comment', foreground: '6A737D', fontStyle: 'italic' },
                { token: 'keyword', foreground: '6F42C1' },
                { token: 'string', foreground: '032F62' },
                { token: 'number', foreground: '005CC5' },
                { token: 'type', foreground: '0550AE' },
                { token: 'function', foreground: '6F42C1' },
            ],
            colors: {
                'editor.background': '#FFFFFF',
                'editor.foreground': '#24292E',
                'editor.lineHighlightBackground': '#F6F8FA',
                'editor.selectionBackground': '#0366D625',
                'editorCursor.foreground': '#24292E',
                'editorLineNumber.foreground': '#6A737D',
            }
        });

        const theme = window.app && window.app.state.theme === 'light' ? 'omnicode-light' : 'omnicode-dark';

        this.editor = monaco.editor.create(document.getElementById(this.containerId), {
            value: '// Welcome to OmniCode\n// Open a file from the sidebar or type code here\n',
            language: 'javascript',
            theme: theme,
            fontSize: window.app ? window.app.state.fontSize : 13,
            tabSize: window.app ? window.app.state.tabSize : 4,
            fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
            minimap: { enabled: false },
            automaticLayout: true,
            cursorBlinking: 'smooth',
            cursorSmoothCaretAnimation: 'on',
            smoothScrolling: true,
            scrollBeyondLastLine: false,
            padding: { top: 12 },
            bracketPairColorization: { enabled: true },
        });

        window.editor = this.editor;

        if (window.app) {
            window.app.on('stateChange', (state) => {
                if (state.theme) {
                    const theme = state.theme === 'light' ? 'omnicode-light' : 'omnicode-dark';
                    this.editor.updateOptions({ theme });
                }
            });
        }

        this.editor.onDidChangeModelContent(() => {
            clearTimeout(this.debounceTimer);
            this.debounceTimer = setTimeout(() => {
                const content = this.editor.getValue();
                if (this.currentFile) {
                    this.sendFileChange(content);
                }
            }, 200);
        });

        window.addEventListener('resize', () => {
            if (this.editor) this.editor.layout();
        });
    }

    loadFile(path, content) {
        this.currentFile = path;
        this.content = content;

        const lang = this.detectLanguage(path);
        const model = monaco.editor.createModel(content, lang);

        if (this.editor) {
            this.editor.setModel(model);
        }
    }

    detectLanguage(filename) {
        const ext = filename.split('.').pop().toLowerCase();
        const map = {
            'rs': 'rust',
            'ts': 'typescript',
            'tsx': 'typescript',
            'js': 'javascript',
            'jsx': 'javascript',
            'py': 'python',
            'go': 'go',
            'java': 'java',
            'c': 'c',
            'cpp': 'cpp',
            'h': 'c',
            'hpp': 'cpp',
            'rb': 'ruby',
            'html': 'html',
            'css': 'css',
            'json': 'json',
            'yaml': 'yaml',
            'yml': 'yaml',
            'toml': 'yaml',
            'md': 'markdown',
            'sql': 'sql',
            'sh': 'shell',
            'bash': 'shell',
            'rs': 'rust',
        };
        return map[ext] || 'plaintext';
    }

    sendFileChange(content) {
        if (window.sync) {
            window.sync.sendFileChange(this.currentFile, content);
        }
    }

    applyRemoteChange(path, content) {
        if (path === this.currentFile && this.editor) {
            const current = this.editor.getValue();
            if (current !== content) {
                const pos = this.editor.getPosition();
                this.editor.setValue(content);
                if (pos) this.editor.setPosition(pos);
            }
        }
    }

    showRemoteCursor(peerId, line, col) {
        if (!this.editor) return;

        const existing = this.remoteCursors.find(c => c.peerId === peerId);
        if (existing) existing.dispose();

        const decorations = this.editor.createDecorationsCollection([{
            range: new monaco.Range(line, col, line, col),
            options: {
                beforeContentClassName: 'remote-cursor',
                isWholeLine: false,
            }
        }]);

        this.remoteCursors.push({
            peerId,
            dispose: () => decorations.clear(),
        });
    }

    updateDiff(diffContent) {
        const diffViewer = document.getElementById('diff-viewer');
        if (!diffViewer) return;

        diffViewer.innerHTML = '';

        diffContent.split('\n').forEach(line => {
            const div = document.createElement('div');
            div.className = 'diff-line';

            if (line.startsWith('+')) {
                div.classList.add('added');
            } else if (line.startsWith('-')) {
                div.classList.add('removed');
            } else if (line.startsWith('@@')) {
                div.classList.add('header');
            }

            div.textContent = line;
            diffViewer.appendChild(div);
        });
    }

    showDiffViewer(diffContent) {
        const diffPane = document.getElementById('diff-pane');
        if (diffPane) diffPane.style.display = 'block';
        if (window.app) {
            window.app.state.diffVisible = true;
        }
        this.updateDiff(diffContent);
    }

    hideDiffViewer() {
        const diffPane = document.getElementById('diff-pane');
        if (diffPane) diffPane.style.display = 'none';
        if (window.app) {
            window.app.state.diffVisible = false;
        }
    }
}

export default EditorManager;
