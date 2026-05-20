class App {
    constructor() {
        this.state = {
            currentScreen: 'chat',
            messages: [],
            chatInput: '',
            agentStatus: 'idle',
            gitBranch: 'main',
            modelName: 'gpt-4',
            peers: [],
            voiceActive: false,
            complianceMode: false,
        };

        this.listeners = {};
        this.init();
    }

    init() {
        this.setupNavigation();
        this.setupCommandPalette();
        this.setupResizeHandles();
        this.setupChatInput();
        this.updateClock();
        this.registerServiceWorker();

        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                this.toggleCommandPalette();
            }
        });
    }

    on(event, callback) {
        if (!this.listeners[event]) this.listeners[event] = [];
        this.listeners[event].push(callback);
    }

    emit(event, data) {
        if (this.listeners[event]) {
            this.listeners[event].forEach(cb => cb(data));
        }
    }

    setState(updates) {
        Object.assign(this.state, updates);
        this.emit('stateChange', this.state);
    }

    setupNavigation() {
        document.querySelectorAll('.sidebar-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                const screen = tab.dataset.screen;
                document.querySelectorAll('.sidebar-tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                this.switchScreen(screen);
            });
        });
    }

    switchScreen(screen) {
        document.querySelectorAll('.pane').forEach(p => p.style.display = 'none');
        if (screen === 'chat' || screen === 'files') {
            document.getElementById('editor-pane').style.display = 'block';
            document.getElementById('chat-pane').style.display = 'flex';
        }
        this.setState({ currentScreen: screen });
    }

    setupCommandPalette() {
        const palette = document.getElementById('command-palette');
        const search = document.getElementById('palette-search');
        const results = document.getElementById('palette-results');

        const commands = [
            { name: 'Chat', shortcut: 'F1', action: () => this.switchScreen('chat') },
            { name: 'Swarm Mode', shortcut: 'F2', action: () => this.switchScreen('swarm') },
            { name: 'Reviews', shortcut: 'F3', action: () => this.switchScreen('reviews') },
            { name: 'Incidents', shortcut: 'F4', action: () => this.switchScreen('incidents') },
            { name: 'Documentation', shortcut: 'F5', action: () => this.switchScreen('docs') },
            { name: 'Dependencies', shortcut: 'F6', action: () => this.switchScreen('deps') },
            { name: 'Decisions', shortcut: 'F7', action: () => this.switchScreen('decisions') },
            { name: 'Skills', shortcut: 'F8', action: () => this.switchScreen('skills') },
            { name: 'Compliance', shortcut: 'F9', action: () => this.switchScreen('compliance') },
            { name: 'Deploy', shortcut: '', action: () => this.switchScreen('deploy') },
            { name: 'Onboarding', shortcut: '', action: () => this.switchScreen('onboarding') },
            { name: 'Toggle File Tree', shortcut: 'Ctrl+B', action: () => {} },
            { name: 'Toggle Diff', shortcut: 'Ctrl+J', action: () => {} },
            { name: 'Save File', shortcut: 'Ctrl+S', action: () => {} },
            { name: 'Send Message', shortcut: 'Enter', action: () => this.sendChatMessage() },
        ];

        search.addEventListener('input', () => {
            const query = search.value.toLowerCase();
            results.innerHTML = '';
            const filtered = commands.filter(c => c.name.toLowerCase().includes(query));
            filtered.forEach(cmd => {
                const div = document.createElement('div');
                div.className = 'palette-item';
                div.innerHTML = `<span>${cmd.name}</span>${cmd.shortcut ? `<span class="shortcut">${cmd.shortcut}</span>` : ''}`;
                div.addEventListener('click', () => {
                    cmd.action();
                    this.toggleCommandPalette();
                });
                results.appendChild(div);
            });
        });

        search.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') this.toggleCommandPalette();
            if (e.key === 'Enter') {
                const first = results.querySelector('.palette-item');
                if (first) first.click();
            }
        });
    }

    toggleCommandPalette() {
        const palette = document.getElementById('command-palette');
        const search = document.getElementById('palette-search');
        if (palette.style.display === 'none') {
            palette.style.display = 'flex';
            search.value = '';
            search.focus();
            document.getElementById('palette-results').innerHTML = '';
        } else {
            palette.style.display = 'none';
        }
    }

    setupResizeHandles() {
        let isDragging = false;
        let currentHandle = null;
        let startX, startWidth;

        document.querySelectorAll('.resize-handle').forEach(handle => {
            handle.addEventListener('mousedown', (e) => {
                isDragging = true;
                currentHandle = handle;
                startX = e.clientX;
                const prev = handle.previousElementSibling;
                if (prev) startWidth = prev.offsetWidth;
                document.body.style.cursor = 'col-resize';
            });
        });

        document.addEventListener('mousemove', (e) => {
            if (!isDragging || !currentHandle) return;
            const diff = e.clientX - startX;
            const prev = currentHandle.previousElementSibling;
            if (prev) {
                prev.style.width = (startWidth + diff) + 'px';
            }
        });

        document.addEventListener('mouseup', () => {
            isDragging = false;
            currentHandle = null;
            document.body.style.cursor = '';
        });
    }

    setupChatInput() {
        const input = document.getElementById('chat-input');
        const sendBtn = document.getElementById('send-btn');

        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                this.sendChatMessage();
            }
        });

        sendBtn.addEventListener('click', () => this.sendChatMessage());
    }

    sendChatMessage() {
        const input = document.getElementById('chat-input');
        const text = input.value.trim();
        if (!text) return;

        this.addMessage('user', text);
        input.value = '';

        this.setState({ agentStatus: 'running' });
        this.emit('sendMessage', text);
    }

    addMessage(role, content) {
        const list = document.getElementById('message-list');
        const div = document.createElement('div');
        div.className = `message ${role}`;

        const time = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        const icons = { user: '🧑', agent: '🤖', system: '⚙️', tool: '🔧' };

        div.innerHTML = `
            <div class="timestamp"><span class="role-icon">${icons[role] || ''}</span>[${time}]</div>
            <div class="content">${this.formatMessage(content)}</div>
        `;

        list.appendChild(div);
        list.scrollTop = list.scrollHeight;
    }

    formatMessage(content) {
        return content
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/```(\w*)\n([\s\S]*?)```/g, '<pre><code>$2</code></pre>')
            .replace(/`([^`]+)`/g, '<code>$1</code>')
            .replace(/\n/g, '<br>');
    }

    updateClock() {
        const timeEl = document.getElementById('current-time');
        setInterval(() => {
            timeEl.textContent = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        }, 1000);
    }

    registerServiceWorker() {
        if ('serviceWorker' in navigator) {
            navigator.serviceWorker.register('/sw.js').catch(() => {});
        }
    }
}

const app = new App();
window.app = app;
export default app;
