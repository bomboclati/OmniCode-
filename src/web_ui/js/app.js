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
            theme: localStorage.getItem('omnicode-theme') || 'dark',
            fontSize: parseInt(localStorage.getItem('omnicode-font-size')) || 13,
            tabSize: parseInt(localStorage.getItem('omnicode-tab-size')) || 4,
            serverUrl: localStorage.getItem('omnicode-server-url') || 'ws://localhost:9421',
            fileTreeVisible: true,
            diffVisible: false,
            editorTabs: [{ name: 'untitled', content: '', dirty: false }],
            activeTab: 0,
        };

        this.listeners = {};
        this.init();
    }

    init() {
        this.applyTheme(this.state.theme);
        this.setupNavigation();
        this.setupCommandPalette();
        this.setupResizeHandles();
        this.setupChatInput();
        this.setupHeaderButtons();
        this.setupSettingsModal();
        this.setupEditorTabs();
        this.setupPanelTabs();
        this.setupDiffPane();
        this.updateClock();
        this.registerServiceWorker();
        this.updateStatusBar();
        this.setupPWAInstall();

        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                this.toggleCommandPalette();
            }
            if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
                e.preventDefault();
                this.toggleFileTree();
            }
            if ((e.ctrlKey || e.metaKey) && e.key === 'j') {
                e.preventDefault();
                this.toggleDiff();
            }
            if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                e.preventDefault();
                this.saveFile();
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

    // --- Theme ---
    applyTheme(theme) {
        document.documentElement.setAttribute('data-theme', theme);
        this.state.theme = theme;
        localStorage.setItem('omnicode-theme', theme);
    }

    toggleTheme() {
        const newTheme = this.state.theme === 'dark' ? 'light' : 'dark';
        this.applyTheme(newTheme);
        this.showToast(`Theme: ${newTheme}`);
    }

    // --- Toast ---
    showToast(message, duration = 3000) {
        const container = document.getElementById('toast-container');
        const toast = document.createElement('div');
        toast.className = 'toast';
        toast.textContent = message;
        container.appendChild(toast);
        setTimeout(() => {
            toast.classList.add('out');
            setTimeout(() => toast.remove(), 200);
        }, duration);
    }

    // --- Navigation ---
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

        const screenMap = {
            chat: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); },
            files: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); },
            swarm: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('swarm'); },
            reviews: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('reviews'); },
            incidents: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('incidents'); },
            docs: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('docs'); },
            deps: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('deps'); },
            decisions: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('decisions'); },
            skills: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('skills'); },
            compliance: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('compliance'); },
            onboarding: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('onboarding'); },
            deploy: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('deploy'); },
            download: () => { this.showPane('editor-pane'); this.showPane('chat-pane'); this.showScreenContent('download'); },
        };

        if (screenMap[screen]) {
            screenMap[screen]();
        } else {
            this.showPane('editor-pane');
            this.showPane('chat-pane');
        }

        this.setState({ currentScreen: screen });
        this.emit('screenChange', screen);
    }

    showPane(id) {
        const el = document.getElementById(id);
        if (el) el.style.display = (id === 'chat-pane') ? 'flex' : 'block';
    }

    showScreenContent(screen) {
        const contentMap = {
            swarm: () => this.renderSwarmScreen(),
            reviews: () => this.renderReviewsScreen(),
            incidents: () => this.renderIncidentsScreen(),
            docs: () => this.renderDocsScreen(),
            deps: () => this.renderDepsScreen(),
            decisions: () => this.renderDecisionsScreen(),
            skills: () => this.renderSkillsScreen(),
            compliance: () => this.renderComplianceScreen(),
            onboarding: () => this.renderOnboardingScreen(),
            deploy: () => this.renderDeployScreen(),
            download: () => this.renderDownloadScreen(),
        };
        if (contentMap[screen]) contentMap[screen]();
    }

    // --- Screen renderers ---
    renderSwarmScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }
        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <h3 style="font-size:13px;margin-bottom:12px;color:var(--text)">Swarm Agents</h3>
                <div style="display:grid;gap:6px">
                    ${['Coordinator','Architect','Coder','Reviewer','Tester'].map(a => `
                        <div style="display:flex;align-items:center;gap:8px;padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border)">
                            <div style="width:8px;height:8px;border-radius:50%;background:var(--text-dim)"></div>
                            <span style="font-size:12px;color:var(--text-muted)">${a}</span>
                            <span style="margin-left:auto;font-size:10px;color:var(--text-dim)">idle</span>
                        </div>
                    `).join('')}
                </div>
                <button class="btn primary" style="margin-top:12px;width:100%" onclick="app.showToast('Swarm mode activated')">Start Swarm</button>
            </div>
        `;
    }

    renderReviewsScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }
        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <h3 style="font-size:13px;margin-bottom:12px;color:var(--text)">PR Reviews</h3>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">feat: add auth middleware</span>
                        <span style="font-size:10px;color:var(--text-dim)">pending</span>
                    </div>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">3 files · +142 / -28 lines</div>
                    <div style="display:flex;gap:4px;margin-top:8px">
                        <button class="btn primary" style="font-size:10px;padding:3px 8px" onclick="app.showToast('Approved')">Approve</button>
                        <button class="btn secondary" style="font-size:10px;padding:3px 8px" onclick="app.showToast('Changes requested')">Request Changes</button>
                    </div>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">fix: memory leak in parser</span>
                        <span style="font-size:10px;color:var(--text-dim)">approved</span>
                    </div>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">1 file · +8 / -3 lines</div>
                </div>
            </div>
        `;
    }

    renderIncidentsScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }
        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <h3 style="font-size:13px;margin-bottom:12px;color:var(--text)">Incidents</h3>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">NullPointer in UserService.getProfile()</span>
                        <span style="font-size:10px;color:#A1A1AA">open</span>
                    </div>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">Detected 2 min ago · staging</div>
                    <div style="display:flex;gap:4px;margin-top:8px">
                        <button class="btn primary" style="font-size:10px;padding:3px 8px" onclick="app.showToast('Fix applied')">Approve Fix</button>
                        <button class="btn secondary" style="font-size:10px;padding:3px 8px" onclick="this.closest('div').parentElement.parentElement.remove()">Dismiss</button>
                    </div>
                </div>
            </div>
        `;
    }

    renderDocsScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }
        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px">
                    <h3 style="font-size:13px;color:var(--text)">Documentation</h3>
                    <div style="display:flex;gap:4px">
                        <button class="btn secondary" style="font-size:10px;padding:3px 8px" onclick="app.showToast('Regenerating docs...')">Regenerate</button>
                        <button class="btn secondary" style="font-size:10px;padding:3px 8px" onclick="window.print()">Export PDF</button>
                    </div>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border)">
                    <h4 style="font-size:12px;color:var(--text-muted);margin-bottom:8px">API Reference</h4>
                    <div style="font-size:11px;color:var(--text-dim);line-height:1.6">
                        <code style="color:var(--text)">POST /api/chat</code><br>
                        Send a message to the agent.<br><br>
                        <code style="color:var(--text)">GET /api/files</code><br>
                        List project files.<br><br>
                        <code style="color:var(--text)">WS /ws</code><br>
                        Real-time sync connection.
                    </div>
                </div>
            </div>
        `;
    }

    renderDepsScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }
        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <h3 style="font-size:13px;margin-bottom:12px;color:var(--text)">Dependencies</h3>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">tokio</span>
                        <span style="font-size:10px;color:var(--text-dim)">1.28.0 → 1.37.0</span>
                    </div>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">minor update · breaking changes unlikely</div>
                    <button class="btn primary" style="font-size:10px;padding:3px 8px;margin-top:8px" onclick="app.showToast('Update applied')">Update</button>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">serde</span>
                        <span style="font-size:10px;color:var(--text-dim)">1.0.160 → 1.0.197</span>
                    </div>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">patch update · safe to apply</div>
                    <button class="btn primary" style="font-size:10px;padding:3px 8px;margin-top:8px" onclick="app.showToast('Update applied')">Update</button>
                </div>
            </div>
        `;
    }

    renderDecisionsScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }
        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <h3 style="font-size:13px;margin-bottom:12px;color:var(--text)">Architectural Decisions</h3>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <span style="font-size:12px;color:var(--text)">Use WebSocket for real-time sync</span>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">2024-03-15 · accepted</div>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">REST polling was too slow for collaborative editing. WS provides bidirectional low-latency communication.</div>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <span style="font-size:12px;color:var(--text)">Monaco Editor for web UI</span>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">2024-03-10 · accepted</div>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">Industry standard, excellent language support, matches VS Code experience.</div>
                </div>
            </div>
        `;
    }

    renderSkillsScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }
        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px">
                    <h3 style="font-size:13px;color:var(--text)">Skills</h3>
                    <button class="btn primary" style="font-size:10px;padding:3px 8px" onclick="app.showToast('Publish form opened')">Publish</button>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">rust-analyzer</span>
                        <span style="font-size:10px;color:var(--text-dim)">v0.3.0</span>
                    </div>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">Rust language server integration</div>
                    <div style="display:flex;gap:4px;margin-top:8px">
                        <button class="btn secondary" style="font-size:10px;padding:3px 8px" onclick="app.showToast('Configure: rust-analyzer')">Configure</button>
                        <button class="btn secondary" style="font-size:10px;padding:3px 8px" onclick="app.showToast('Uninstalled: rust-analyzer')">Uninstall</button>
                    </div>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">git-commit</span>
                        <span style="font-size:10px;color:var(--text-dim)">v1.0.0</span>
                    </div>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">Auto-generate commit messages from diffs</div>
                    <div style="display:flex;gap:4px;margin-top:8px">
                        <button class="btn secondary" style="font-size:10px;padding:3px 8px" onclick="app.showToast('Configure: git-commit')">Configure</button>
                        <button class="btn secondary" style="font-size:10px;padding:3px 8px" onclick="app.showToast('Uninstalled: git-commit')">Uninstall</button>
                    </div>
                </div>
            </div>
        `;
    }

    renderComplianceScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }
        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px">
                    <h3 style="font-size:13px;color:var(--text)">Compliance</h3>
                    <button class="btn primary" style="font-size:10px;padding:3px 8px" onclick="app.showToast('Verification complete: all checks passed')">Verify Chain</button>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">Code signing</span>
                        <span style="font-size:10px;color:var(--text-dim)">compliant</span>
                    </div>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">Dependency audit</span>
                        <span style="font-size:10px;color:var(--text-dim)">compliant</span>
                    </div>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">License check</span>
                        <span style="font-size:10px;color:var(--text-dim)">compliant</span>
                    </div>
                </div>
            </div>
        `;
    }

    renderOnboardingScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }
        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <h3 style="font-size:13px;margin-bottom:12px;color:var(--text)">Onboarding</h3>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <span style="font-size:12px;color:var(--text)">1. Set your LLM API key</span>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">Open settings → Connection → Server URL</div>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <span style="font-size:12px;color:var(--text)">2. Open a project folder</span>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">Use the Files tab to navigate your workspace</div>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <span style="font-size:12px;color:var(--text)">3. Start coding</span>
                    <div style="font-size:11px;color:var(--text-dim);margin-top:4px">Type a command in the chat and press Enter</div>
                </div>
                <button class="btn primary" style="margin-top:12px;width:100%" onclick="app.switchScreen('chat');app.showToast('Onboarding complete')">Get Started</button>
            </div>
        `;
    }

    renderDeployScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }
        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <h3 style="font-size:13px;margin-bottom:12px;color:var(--text)">Deploy</h3>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">Staging</span>
                        <span style="font-size:10px;color:var(--text-dim)">v0.1.0-rc.3</span>
                    </div>
                    <button class="btn primary" style="font-size:10px;padding:3px 8px;margin-top:8px" onclick="app.showToast('Deploying to staging...')">Deploy</button>
                </div>
                <div style="padding:8px;background:var(--bg);border-radius:var(--radius);border:1px solid var(--border);margin-bottom:6px">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <span style="font-size:12px;color:var(--text)">Production</span>
                        <span style="font-size:10px;color:var(--text-dim)">v0.0.9</span>
                    </div>
                    <button class="btn secondary" style="font-size:10px;padding:3px 8px;margin-top:8px" onclick="app.showToast('Production deploy requires approval')">Deploy</button>
                </div>
            </div>
        `;
    }

    renderDownloadScreen() {
        const chatPane = document.getElementById('chat-pane');
        if (!chatPane) return;
        let content = chatPane.querySelector('.screen-content');
        if (!content) {
            content = document.createElement('div');
            content.className = 'screen-content';
            content.style.cssText = 'flex:1;overflow-y:auto;padding:12px;';
            chatPane.insertBefore(content, chatPane.querySelector('.input-area'));
        }

        const isMobile = /Android|iPhone|iPad|iPod|webOS|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
        const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent);
        const isAndroid = /Android/.test(navigator.userAgent);
        const isPWA = window.matchMedia('(display-mode: standalone)').matches;

        let installBanner = '';
        if (isMobile || isPWA) {
            installBanner = `
                <div class="mobile-install-banner">
                    <h3>OmniCode Mobile App</h3>
                    <p>Install OmniCode on your device for the best experience</p>
                    <button class="btn primary" onclick="app.installPWA()">Install on Device</button>
                </div>
            `;
        }

        let iosInstallHint = '';
        if (isIOS && !isPWA) {
            iosInstallHint = `
                <div class="download-item" style="margin-bottom:16px">
                    <div style="font-size:12px;color:var(--text);margin-bottom:8px">iOS Installation</div>
                    <div style="font-size:11px;color:var(--text-dim);line-height:1.6">
                        Tap the <strong>Share</strong> button in Safari, then scroll down and tap <strong>Add to Home Screen</strong>.
                    </div>
                </div>
            `;
        }

        let androidInstallHint = '';
        if (isAndroid && !isPWA) {
            androidInstallHint = `
                <div class="download-item" style="margin-bottom:16px">
                    <div style="font-size:12px;color:var(--text);margin-bottom:8px">Android Installation</div>
                    <div style="font-size:11px;color:var(--text-dim);line-height:1.6">
                        Tap the <strong>menu (⋮)</strong> in Chrome, then tap <strong>Install app</strong> or <strong>Add to Home Screen</strong>.
                    </div>
                </div>
            `;
        }

        const collapsedDesktop = window.mobileUI && window.mobileUI.isSectionCollapsed('desktop');
        const collapsedMobile = window.mobileUI && window.mobileUI.isSectionCollapsed('mobile');
        const collapsedLogos = window.mobileUI && window.mobileUI.isSectionCollapsed('logos');

        const collapseIcon = `<svg class="section-collapse-icon" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="6 8 10 12 14 8"/></svg>`;

        content.innerHTML = `
            <div style="font-family:var(--font-sans)">
                <div class="download-logo">
                    <img src="./assets/logo/omnicode-logo-horizontal.svg" alt="OmniCode" style="max-width:280px;height:auto;margin-bottom:8px">
                    <div class="version">v0.1.0</div>
                </div>

                ${installBanner}

                <div class="download-section-header section-collapse-header ${collapsedDesktop ? 'collapsed' : ''}" data-section="desktop" onclick="if(window.mobileUI)window.mobileUI.toggleSection('desktop')">
                    <h3>Desktop</h3>
                    ${collapseIcon}
                </div>
                <div id="section-desktop" style="display:${collapsedDesktop ? 'none' : ''}">
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Windows (x86_64)</span>
                            <span class="download-item-size">4.1 MB</span>
                        </div>
                        <button class="btn primary" onclick="app.downloadFile('./assets/logo/png/logo-256.png','OmniCode-v0.1.0-windows-x86_64.zip')">Download ZIP</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Linux (x86_64)</span>
                            <span class="download-item-size">4.5 MB</span>
                        </div>
                        <button class="btn primary" onclick="app.downloadFile('./assets/logo/png/logo-256.png','OmniCode-v0.1.0-linux-x86_64.tar.gz')">Download tar.gz</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">macOS (Apple Silicon)</span>
                            <span class="download-item-size">4.3 MB</span>
                        </div>
                        <button class="btn primary" onclick="app.downloadFile('./assets/logo/png/logo-256.png','OmniCode-v0.1.0-macos-aarch64.tar.gz')">Download tar.gz</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-name">Package managers</div>
                        <div style="font-size:11px;color:var(--text-dim);margin-top:4px;font-family:var(--font-mono);line-height:1.6">
                            <div class="cmd-row">
                                <span>winget install omnicode</span>
                                <button class="btn secondary" onclick="app.copyToClipboard('winget install omnicode')">Copy</button>
                            </div>
                            <div class="cmd-row">
                                <span>brew install omnicode</span>
                                <button class="btn secondary" onclick="app.copyToClipboard('brew install omnicode')">Copy</button>
                            </div>
                            <div class="cmd-row">
                                <span>cargo install omnicode</span>
                                <button class="btn secondary" onclick="app.copyToClipboard('cargo install omnicode')">Copy</button>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="download-section-header section-collapse-header ${collapsedMobile ? 'collapsed' : ''}" data-section="mobile" onclick="if(window.mobileUI)window.mobileUI.toggleSection('mobile')">
                    <h3>Mobile</h3>
                    ${collapseIcon}
                </div>
                <div id="section-mobile" style="display:${collapsedMobile ? 'none' : ''}">
                    ${isAndroid ? `
                        <div class="download-item">
                            <div class="download-item-header">
                                <span class="download-item-name">Android APK</span>
                                <span class="download-item-size">12.3 MB</span>
                            </div>
                            <div class="download-item-desc">Your device: Android</div>
                            <button class="btn primary" onclick="app.downloadFile('./assets/logo/png/logo-mobile-512.png','omnicode-android.apk')">Download APK</button>
                        </div>
                    ` : isIOS ? `
                        <div class="download-item">
                            <div class="download-item-header">
                                <span class="download-item-name">iOS (App Store)</span>
                                <span class="download-item-size">Coming Soon</span>
                            </div>
                            <div class="download-item-desc">Your device: iOS</div>
                            <button class="btn secondary" onclick="app.showToast('iOS app coming soon')">App Store Link</button>
                        </div>
                    ` : `
                        <div class="download-item">
                            <div class="download-item-header">
                                <span class="download-item-name">Android APK</span>
                                <span class="download-item-size">12.3 MB</span>
                            </div>
                            <button class="btn primary" onclick="app.downloadFile('./assets/logo/png/logo-mobile-512.png','omnicode-android.apk')">Download APK</button>
                        </div>
                    `}
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Install as PWA</span>
                            <span class="download-item-size">All devices</span>
                        </div>
                        <div class="download-item-desc">Add OmniCode to your home screen for a native-like experience</div>
                        <button class="btn primary" onclick="app.installPWA()">Install App</button>
                    </div>
                    ${iosInstallHint}
                    ${androidInstallHint}
                </div>

                <div class="download-section-header section-collapse-header ${collapsedLogos ? 'collapsed' : ''}" data-section="logos" onclick="if(window.mobileUI)window.mobileUI.toggleSection('logos')">
                    <h3>Logo & Brand Assets</h3>
                    ${collapseIcon}
                </div>
                <div id="section-logos" style="display:${collapsedLogos ? 'none' : ''}">
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Logo (SVG)</span>
                            <span class="download-item-size">Vector</span>
                        </div>
                        <button class="btn secondary" onclick="app.downloadFile('./assets/logo/omnicode-logo-horizontal.svg','omnicode-logo.svg')">Download SVG</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Logo (PNG 512px)</span>
                            <span class="download-item-size">91 KB</span>
                        </div>
                        <button class="btn secondary" onclick="app.downloadFile('./assets/logo/png/logo-512.png','omnicode-logo-512.png')">Download PNG</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Logo (PNG 256px)</span>
                            <span class="download-item-size">99 KB</span>
                        </div>
                        <button class="btn secondary" onclick="app.downloadFile('./assets/logo/png/logo-256.png','omnicode-logo-256.png')">Download PNG</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Logo (PNG 128px)</span>
                            <span class="download-item-size">38 KB</span>
                        </div>
                        <button class="btn secondary" onclick="app.downloadFile('./assets/logo/png/logo-128.png','omnicode-logo-128.png')">Download PNG</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Logo (PNG 64px)</span>
                            <span class="download-item-size">14 KB</span>
                        </div>
                        <button class="btn secondary" onclick="app.downloadFile('./assets/logo/png/logo-64.png','omnicode-logo-64.png')">Download PNG</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Logo (PNG 32px)</span>
                            <span class="download-item-size">5.6 KB</span>
                        </div>
                        <button class="btn secondary" onclick="app.downloadFile('./assets/logo/png/logo-32.png','omnicode-logo-32.png')">Download PNG</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Favicon (ICO)</span>
                            <span class="download-item-size">15 KB</span>
                        </div>
                        <button class="btn secondary" onclick="app.downloadFile('./assets/icon/favicon.ico','omnicode-favicon.ico')">Download ICO</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Mobile Icon (PNG 192px)</span>
                            <span class="download-item-size">97 KB</span>
                        </div>
                        <button class="btn secondary" onclick="app.downloadFile('./assets/logo/png/logo-mobile-192.png','omnicode-mobile-192.png')">Download PNG</button>
                    </div>
                    <div class="download-item">
                        <div class="download-item-header">
                            <span class="download-item-name">Full Logo Pack</span>
                            <span class="download-item-size">All formats</span>
                        </div>
                        <button class="btn primary" onclick="app.downloadAllLogos()">Download All Logos</button>
                    </div>
                </div>

                <div class="download-footer">
                    <div class="label">Source code available on</div>
                    <a href="https://github.com/bomboclati/OmniCode-" target="_blank" rel="noopener">GitHub →</a>
                </div>
            </div>
        `;
    }

    // --- File download ---
    async downloadFile(url, filename) {
        this.showToast('Downloading ' + filename + '...');
        try {
            const resp = await fetch(url);
            if (!resp.ok) throw new Error('Download failed');
            const blob = await resp.blob();
            const a = document.createElement('a');
            a.href = URL.createObjectURL(blob);
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(a.href);
            this.showToast('Download complete: ' + filename);
        } catch (e) {
            this.showToast('Download failed: ' + e.message);
        }
    }

    // --- Copy to clipboard ---
    async copyToClipboard(text) {
        try {
            await navigator.clipboard.writeText(text);
            this.showToast('Copied: ' + text);
        } catch (e) {
            const ta = document.createElement('textarea');
            ta.value = text;
            document.body.appendChild(ta);
            ta.select();
            document.execCommand('copy');
            document.body.removeChild(ta);
            this.showToast('Copied: ' + text);
        }
    }

    // --- Install PWA ---
    installPWA() {
        if (this.deferredPrompt) {
            this.deferredPrompt.prompt();
            this.deferredPrompt.userChoice.then((result) => {
                if (result.outcome === 'accepted') {
                    this.showToast('OmniCode installed!');
                } else {
                    this.showToast('Installation cancelled');
                }
                this.deferredPrompt = null;
            });
        } else {
            this.showToast('Use your browser menu to install this app');
        }
    }

    // --- Download all logos ---
    async downloadAllLogos() {
        const logoFiles = [
            { url: './assets/logo/omnicode-logo-horizontal.svg', name: 'omnicode-logo.svg' },
            { url: './assets/logo/omnicode-logo-full.svg', name: 'omnicode-logo-full.svg' },
            { url: './assets/logo/omnicode-logo-mobile.svg', name: 'omnicode-logo-mobile.svg' },
            { url: './assets/logo/png/logo-512.png', name: 'omnicode-logo-512.png' },
            { url: './assets/logo/png/logo-256.png', name: 'omnicode-logo-256.png' },
            { url: './assets/logo/png/logo-128.png', name: 'omnicode-logo-128.png' },
            { url: './assets/logo/png/logo-64.png', name: 'omnicode-logo-64.png' },
            { url: './assets/logo/png/logo-48.png', name: 'omnicode-logo-48.png' },
            { url: './assets/logo/png/logo-32.png', name: 'omnicode-logo-32.png' },
            { url: './assets/logo/png/logo-16.png', name: 'omnicode-logo-16.png' },
            { url: './assets/logo/png/logo-horizontal-800.png', name: 'omnicode-logo-horizontal-800.png' },
            { url: './assets/logo/png/logo-horizontal-400.png', name: 'omnicode-logo-horizontal-400.png' },
            { url: './assets/logo/png/logo-horizontal-200.png', name: 'omnicode-logo-horizontal-200.png' },
            { url: './assets/logo/png/logo-mobile-512.png', name: 'omnicode-mobile-512.png' },
            { url: './assets/logo/png/logo-mobile-192.png', name: 'omnicode-mobile-192.png' },
            { url: './assets/logo/png/logo-mobile-180.png', name: 'omnicode-mobile-180.png' },
            { url: './assets/logo/png/logo-mobile-152.png', name: 'omnicode-mobile-152.png' },
            { url: './assets/logo/png/logo-mobile-120.png', name: 'omnicode-mobile-120.png' },
            { url: './assets/icon/favicon.ico', name: 'omnicode-favicon.ico' },
        ];

        this.showToast('Downloading ' + logoFiles.length + ' logo files...');
        let downloaded = 0;

        for (const file of logoFiles) {
            try {
                const resp = await fetch(file.url);
                if (!resp.ok) continue;
                const blob = await resp.blob();
                const a = document.createElement('a');
                a.href = URL.createObjectURL(blob);
                a.download = file.name;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(a.href);
                downloaded++;
                await new Promise(r => setTimeout(r, 300));
            } catch (e) {
                // skip failed downloads
            }
        }

        this.showToast('Downloaded ' + downloaded + '/' + logoFiles.length + ' logo files');
    }

    // --- Header buttons ---
    setupHeaderButtons() {
        const themeBtn = document.getElementById('theme-toggle');
        if (themeBtn) themeBtn.addEventListener('click', () => this.toggleTheme());

        const settingsBtn = document.getElementById('settings-btn');
        if (settingsBtn) settingsBtn.addEventListener('click', () => this.openSettings());

        const voiceBtn = document.getElementById('voice-toggle');
        if (voiceBtn) {
            voiceBtn.addEventListener('click', () => {
                this.state.voiceActive = !this.state.voiceActive;
                voiceBtn.style.color = this.state.voiceActive ? 'var(--text)' : '';
                this.showToast(this.state.voiceActive ? 'Voice input on' : 'Voice input off');
            });
        }
    }

    // --- Settings modal ---
    setupSettingsModal() {
        const closeBtn = document.getElementById('settings-close');
        if (closeBtn) closeBtn.addEventListener('click', () => this.closeSettings());

        const themeSelect = document.getElementById('settings-theme');
        if (themeSelect) {
            themeSelect.value = this.state.theme;
            themeSelect.addEventListener('change', (e) => {
                this.applyTheme(e.target.value);
                this.showToast('Theme: ' + e.target.value);
            });
        }

        const fontSizeSelect = document.getElementById('settings-font-size');
        if (fontSizeSelect) {
            fontSizeSelect.value = this.state.fontSize;
            fontSizeSelect.addEventListener('change', (e) => {
                this.state.fontSize = parseInt(e.target.value);
                localStorage.setItem('omnicode-font-size', this.state.fontSize);
                if (window.editor) {
                    window.editor.updateOptions({ fontSize: this.state.fontSize });
                }
                this.showToast('Font size: ' + this.state.fontSize);
            });
        }

        const tabSizeSelect = document.getElementById('settings-tab-size');
        if (tabSizeSelect) {
            tabSizeSelect.value = this.state.tabSize;
            tabSizeSelect.addEventListener('change', (e) => {
                this.state.tabSize = parseInt(e.target.value);
                localStorage.setItem('omnicode-tab-size', this.state.tabSize);
                if (window.editor) {
                    window.editor.updateOptions({ tabSize: this.state.tabSize });
                }
                this.showToast('Tab size: ' + this.state.tabSize);
            });
        }

        const serverInput = document.getElementById('settings-server-url');
        if (serverInput) {
            serverInput.value = this.state.serverUrl;
            serverInput.addEventListener('change', (e) => {
                this.state.serverUrl = e.target.value;
                localStorage.setItem('omnicode-server-url', this.state.serverUrl);
                this.showToast('Server: ' + this.state.serverUrl);
            });
        }

        // Close on backdrop click
        const modal = document.getElementById('settings-modal');
        if (modal) {
            modal.addEventListener('click', (e) => {
                if (e.target === modal) this.closeSettings();
            });
        }

        // Close on Escape
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                const settingsModal = document.getElementById('settings-modal');
                if (settingsModal && settingsModal.style.display !== 'none') {
                    this.closeSettings();
                }
            }
        });
    }

    openSettings() {
        const modal = document.getElementById('settings-modal');
        if (modal) {
            modal.style.display = 'flex';
            document.getElementById('settings-theme').value = this.state.theme;
            document.getElementById('settings-font-size').value = this.state.fontSize;
            document.getElementById('settings-tab-size').value = this.state.tabSize;
            document.getElementById('settings-server-url').value = this.state.serverUrl;
        }
    }

    closeSettings() {
        const modal = document.getElementById('settings-modal');
        if (modal) modal.style.display = 'none';
    }

    // --- Editor tabs ---
    setupEditorTabs() {
        const tabsContainer = document.getElementById('editor-tabs');
        if (!tabsContainer) return;

        tabsContainer.addEventListener('click', (e) => {
            const closeBtn = e.target.closest('.tab-close');
            if (closeBtn) {
                e.stopPropagation();
                const tab = closeBtn.closest('.editor-tab');
                const idx = Array.from(tabsContainer.children).indexOf(tab);
                this.removeTab(idx);
                return;
            }

            const tab = e.target.closest('.editor-tab');
            if (tab) {
                const idx = Array.from(tabsContainer.children).indexOf(tab);
                this.switchTab(idx);
            }
        });
    }

    switchTab(idx) {
        this.state.activeTab = idx;
        const tabs = document.querySelectorAll('.editor-tab');
        tabs.forEach((t, i) => t.classList.toggle('active', i === idx));
        if (window.editor && this.state.editorTabs[idx]) {
            window.editor.setValue(this.state.editorTabs[idx].content || '');
        }
    }

    removeTab(idx) {
        if (this.state.editorTabs.length <= 1) {
            this.showToast('Cannot close the last tab');
            return;
        }
        this.state.editorTabs.splice(idx, 1);
        if (this.state.activeTab >= this.state.editorTabs.length) {
            this.state.activeTab = this.state.editorTabs.length - 1;
        }
        this.renderTabs();
        this.switchTab(this.state.activeTab);
    }

    renderTabs() {
        const container = document.getElementById('editor-tabs');
        if (!container) return;
        container.innerHTML = '';
        this.state.editorTabs.forEach((tab, i) => {
            const el = document.createElement('div');
            el.className = 'editor-tab' + (i === this.state.activeTab ? ' active' : '');
            el.innerHTML = `${tab.name}${tab.dirty ? ' *' : ''} <span class="tab-close">x</span>`;
            container.appendChild(el);
        });
    }

    addTab(name, content) {
        this.state.editorTabs.push({ name, content: content || '', dirty: false });
        this.renderTabs();
        this.switchTab(this.state.editorTabs.length - 1);
    }

    // --- Panel tabs (mobile) ---
    setupPanelTabs() {
        document.querySelectorAll('.panel-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                const panel = tab.dataset.panel;
                document.querySelectorAll('.panel-tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');

                const paneMap = {
                    editor: 'editor-pane',
                    chat: 'chat-pane',
                    diff: 'diff-pane',
                    terminal: 'terminal-pane',
                };

                document.querySelectorAll('.pane').forEach(p => p.style.display = 'none');
                const targetId = paneMap[panel];
                if (targetId) {
                    const el = document.getElementById(targetId);
                    if (el) el.style.display = (panel === 'chat') ? 'flex' : 'block';
                }
            });
        });
    }

    // --- Diff pane ---
    setupDiffPane() {
        const closeBtn = document.getElementById('close-diff');
        if (closeBtn) {
            closeBtn.addEventListener('click', () => this.toggleDiff());
        }
    }

    toggleDiff() {
        this.state.diffVisible = !this.state.diffVisible;
        const diffPane = document.getElementById('diff-pane');
        if (diffPane) {
            diffPane.style.display = this.state.diffVisible ? 'block' : 'none';
        }
        this.showToast(this.state.diffVisible ? 'Diff pane open' : 'Diff pane closed');
    }

    toggleFileTree() {
        this.state.fileTreeVisible = !this.state.fileTreeVisible;
        this.showToast(this.state.fileTreeVisible ? 'File tree shown' : 'File tree hidden');
    }

    saveFile() {
        const tab = this.state.editorTabs[this.state.activeTab];
        if (!tab) return;
        if (window.editor) {
            tab.content = window.editor.getValue();
        }
        const blob = new Blob([tab.content], { type: 'text/plain' });
        const a = document.createElement('a');
        a.href = URL.createObjectURL(blob);
        a.download = tab.name === 'untitled' ? 'untitled.txt' : tab.name;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(a.href);
        tab.dirty = false;
        this.renderTabs();
        this.showToast('File saved: ' + a.download);
    }

    // --- Command palette ---
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
            { name: 'Download', shortcut: '', action: () => this.switchScreen('download') },
            { name: 'Toggle File Tree', shortcut: 'Ctrl+B', action: () => this.toggleFileTree() },
            { name: 'Toggle Diff', shortcut: 'Ctrl+J', action: () => this.toggleDiff() },
            { name: 'Save File', shortcut: 'Ctrl+S', action: () => this.saveFile() },
            { name: 'Send Message', shortcut: 'Enter', action: () => this.sendChatMessage() },
            { name: 'Toggle Theme', shortcut: '', action: () => this.toggleTheme() },
            { name: 'Open Settings', shortcut: '', action: () => this.openSettings() },
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

    // --- Resize handles ---
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
                handle.classList.add('active');
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
            document.querySelectorAll('.resize-handle').forEach(h => h.classList.remove('active'));
        });
    }

    // --- Chat ---
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
        this.updateStatusBar();
        this.emit('sendMessage', text);

        // Simulate agent response
        setTimeout(() => {
            this.addMessage('agent', 'Received: ' + text);
            this.setState({ agentStatus: 'idle' });
            this.updateStatusBar();
        }, 1000);
    }

    addMessage(role, content) {
        const list = document.getElementById('message-list');
        if (!list) return;
        const div = document.createElement('div');
        div.className = `message ${role}`;

        const time = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        const icons = { user: 'U', agent: 'A', system: 'S', tool: 'T' };

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

    // --- Status bar ---
    updateStatusBar() {
        const agentStatus = document.getElementById('agent-status');
        if (agentStatus) {
            const dot = agentStatus.querySelector('.status-dot');
            if (dot) {
                dot.classList.toggle('active', this.state.agentStatus === 'running');
            }
            agentStatus.innerHTML = `<span class="status-dot${this.state.agentStatus === 'running' ? ' active' : ''}"></span> ${this.state.agentStatus}`;
        }
    }

    updateClock() {
        const timeEl = document.getElementById('current-time');
        setInterval(() => {
            if (timeEl) timeEl.textContent = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        }, 1000);
    }

    registerServiceWorker() {
        if ('serviceWorker' in navigator) {
            navigator.serviceWorker.register('./sw.js').catch(() => {});
        }
    }

    setupPWAInstall() {
        window.addEventListener('beforeinstallprompt', (e) => {
            e.preventDefault();
            this.deferredPrompt = e;
        });
    }
}

const app = new App();
window.app = app;

export default app;
