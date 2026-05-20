class TerminalEmulator {
    constructor(containerId = 'terminal-emulator') {
        this.containerId = containerId;
        this.container = null;
        this.history = [];
        this.historyIndex = -1;
        this.currentLine = '';
        this.cursorVisible = true;
        this.prompt = '$ ';
        this.initialized = false;
    }

    init() {
        this.container = document.getElementById(this.containerId);
        if (!this.container) return;

        this.initialized = true;
        this.printWelcome();
        this.render();
        this.blinkCursor();

        document.addEventListener('keydown', (e) => {
            if (!this.initialized || this.container.style.display === 'none') return;

            if (e.key === 'Enter') {
                this.executeCommand(this.currentLine);
                this.currentLine = '';
                this.historyIndex = -1;
            } else if (e.key === 'Backspace') {
                this.currentLine = this.currentLine.slice(0, -1);
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                if (this.history.length > 0) {
                    this.historyIndex = Math.min(this.historyIndex + 1, this.history.length - 1);
                    this.currentLine = this.history[this.history.length - 1 - this.historyIndex] || '';
                }
            } else if (e.key === 'ArrowDown') {
                e.preventDefault();
                if (this.historyIndex > 0) {
                    this.historyIndex--;
                    this.currentLine = this.history[this.history.length - 1 - this.historyIndex] || '';
                } else {
                    this.historyIndex = -1;
                    this.currentLine = '';
                }
            } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
                this.currentLine += e.key;
            }

            this.render();
        });

        this.container.addEventListener('click', () => {
            if (this.initialized) {
                const input = this.container.querySelector('.terminal-input');
                if (input) input.focus();
            }
        });

        const commands = [
            'omni "build REST API"',
            'omni swarm "refactor auth module"',
            'omni sentinel watch',
            'omni heal',
            'omni deploy',
            'omni find "database connection"',
        ];

        let i = 0;
        const demoInterval = setInterval(() => {
            if (i < commands.length) {
                this.typeCommand(commands[i]);
                i++;
            } else {
                clearInterval(demoInterval);
            }
        }, 3000);
    }

    typeCommand(cmd) {
        let j = 0;
        const typeInterval = setInterval(() => {
            if (j < cmd.length) {
                this.currentLine += cmd[j];
                j++;
                this.render();
            } else {
                clearInterval(typeInterval);
                setTimeout(() => {
                    this.history.push(this.currentLine);
                    this.currentLine = '';
                    this.render();
                }, 500);
            }
        }, 50);
    }

    printWelcome() {
        const lines = [
            '╔══════════════════════════════════════════╗',
            '║         OmniCode Terminal v0.1.0         ║',
            '║  Autonomous AI Coding Agent              ║',
            '╚══════════════════════════════════════════╝',
            '',
            'Type a command or press ↑/↓ for history.',
            'Examples:',
            '  omni "implement feature"',
            '  omni swarm "task description"',
            '  omni heal',
            '  omni deploy',
            '',
        ];

        lines.forEach(line => {
            const div = document.createElement('div');
            div.className = 'terminal-line';
            div.textContent = line;
            this.container.appendChild(div);
        });
    }

    render() {
        if (!this.initialized || !this.container) return;

        const existing = this.container.querySelector('.terminal-input-line');
        if (existing) existing.remove();

        const line = document.createElement('div');
        line.className = 'terminal-input-line';

        const promptSpan = document.createElement('span');
        promptSpan.className = 'terminal-prompt';
        promptSpan.textContent = this.prompt;

        const textSpan = document.createElement('span');
        textSpan.className = 'terminal-text';
        textSpan.textContent = this.currentLine;

        const cursorSpan = document.createElement('span');
        cursorSpan.className = 'terminal-cursor';
        cursorSpan.textContent = this.cursorVisible ? '█' : ' ';

        line.appendChild(promptSpan);
        line.appendChild(textSpan);
        line.appendChild(cursorSpan);
        this.container.appendChild(line);
        this.container.scrollTop = this.container.scrollHeight;
    }

    blinkCursor() {
        setInterval(() => {
            this.cursorVisible = !this.cursorVisible;
            const cursor = this.container?.querySelector('.terminal-cursor');
            if (cursor) {
                cursor.textContent = this.cursorVisible ? '█' : ' ';
            }
        }, 530);
    }

    executeCommand(cmd) {
        const line = document.createElement('div');
        line.className = 'terminal-line';
        line.innerHTML = `<span class="terminal-prompt">${this.prompt}</span>${this.escapeHtml(cmd)}`;
        this.container.appendChild(line);

        if (cmd.trim()) {
            this.history.push(cmd.trim());
            this.historyIndex = -1;

            setTimeout(() => {
                this.showOutput(`Processing: ${cmd}`);
                setTimeout(() => this.showOutput('✓ Done'), 1500);
            }, 500);
        }
    }

    showOutput(text) {
        const div = document.createElement('div');
        div.className = 'terminal-line terminal-output';
        div.textContent = text;
        this.container.appendChild(div);
        this.container.scrollTop = this.container.scrollHeight;
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

export default TerminalEmulator;
