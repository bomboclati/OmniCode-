class CollaborationUI {
    constructor() {
        this.peers = [];
        this.sessionCode = null;
        this.isHost = false;
    }

    init() {
        this.setupSessionDialogs();
    }

    setupSessionDialogs() {
        const shareBtn = document.querySelector('[data-action="share"]');
        if (shareBtn) {
            shareBtn.addEventListener('click', () => this.createSession());
        }

        const joinBtn = document.querySelector('[data-action="join"]');
        if (joinBtn) {
            joinBtn.addEventListener('click', () => this.showJoinDialog());
        }
    }

    createSession() {
        const code = this.generateCode();
        this.sessionCode = code;
        this.isHost = true;

        if (window.app && window.app.showToast) {
            window.app.showToast('Session created: ' + code);
        }

        const modal = this.createModal(`
            <h3>Collaboration Session</h3>
            <p>Share this code with others:</p>
            <div class="session-code">${code}</div>
            <p>Waiting for peers to join...</p>
            <button class="btn outline" onclick="this.closest('.modal').remove()">Close</button>
        `);
        document.body.appendChild(modal);

        if (window.sync) {
            window.sync.send({ type: 'collaboration_join', code });
        }
    }

    showJoinDialog() {
        const modal = this.createModal(`
            <h3>Join Collaboration Session</h3>
            <input type="text" id="join-code" placeholder="Enter session code..." class="input">
            <button class="btn primary" onclick="collaboration.joinSession(document.getElementById('join-code').value)">Join</button>
            <button class="btn outline" onclick="this.closest('.modal').remove()">Cancel</button>
        `);
        document.body.appendChild(modal);

        setTimeout(() => {
            document.getElementById('join-code')?.focus();
        }, 100);
    }

    joinSession(code) {
        if (!code) return;
        this.sessionCode = code;
        this.isHost = false;

        if (window.app && window.app.showToast) {
            window.app.showToast('Joining session: ' + code);
        }

        this.closeModals();
        if (window.sync) {
            window.sync.send({ type: 'collaboration_join', code });
        }
    }

    addPeer(peerId, name) {
        if (!this.peers.find(p => p.id === peerId)) {
            this.peers.push({ id: peerId, name: name || `Peer ${peerId.slice(0, 4)}` });
            this.updatePeerList();
        }
    }

    removePeer(peerId) {
        this.peers = this.peers.filter(p => p.id !== peerId);
        this.updatePeerList();
    }

    updatePeerList() {
        const container = document.getElementById('peer-avatars');
        if (!container) return;

        container.innerHTML = '';
        this.peers.forEach(peer => {
            const avatar = document.createElement('div');
            avatar.className = 'peer-avatar';
            avatar.title = peer.name;
            avatar.textContent = peer.name.charAt(0).toUpperCase();
            avatar.style.background = this.getColorForPeer(peer.id);
            container.appendChild(avatar);
        });

        const countEl = document.getElementById('peer-count');
        if (countEl) {
            countEl.textContent = `${this.peers.length} peers`;
        }
    }

    getColorForPeer(id) {
        const colors = ['#00FFA3', '#8957FF', '#FFD700', '#FF4444', '#00AAFF', '#FFAA00'];
        let hash = 0;
        for (let i = 0; i < id.length; i++) {
            hash = ((hash << 5) - hash) + id.charCodeAt(i);
        }
        return colors[Math.abs(hash) % colors.length];
    }

    generateCode() {
        const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
        let code = '';
        for (let i = 0; i < 8; i++) {
            code += chars.charAt(Math.floor(Math.random() * chars.length));
        }
        return code;
    }

    createModal(html) {
        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = `<div class="modal-content">${html}</div>`;
        modal.addEventListener('click', (e) => {
            if (e.target === modal) modal.remove();
        });
        return modal;
    }

    closeModals() {
        document.querySelectorAll('.modal').forEach(m => m.remove());
    }
}

const collaboration = new CollaborationUI();
export default collaboration;
