class SkillManager {
    constructor() {
        this.skills = [];
        this.marketplace = [];
    }

    init() {
        this.loadInstalled();
        this.loadMarketplace();
    }

    async loadInstalled() {
        try {
            const resp = await fetch('/api/skills');
            this.skills = await resp.json();
            this.renderSkills();
        } catch (e) {
            console.warn('Failed to load skills:', e);
        }
    }

    async loadMarketplace() {
        try {
            const resp = await fetch('https://api.omnicode.dev/skills');
            this.marketplace = await resp.json();
            this.renderMarketplace();
        } catch (e) {
            console.warn('Failed to load marketplace:', e);
        }
    }

    renderSkills() {
        const container = document.querySelector('.skills-grid');
        if (!container) return;

        container.innerHTML = '';
        this.skills.forEach(skill => {
            const card = this.createSkillCard(skill);
            container.appendChild(card);
        });
    }

    createSkillCard(skill) {
        const card = document.createElement('div');
        card.className = 'skill-card';
        card.innerHTML = `
            <div class="skill-header">
                <span class="skill-icon">🔧</span>
                <span class="skill-name">${skill.name || 'Unknown'}</span>
                <span class="skill-version">v${skill.version || '0.0.0'}</span>
            </div>
            <div class="skill-description">${skill.description || ''}</div>
            <div class="skill-actions">
                <button class="btn secondary small configure-btn">Configure</button>
                <button class="btn outline small uninstall-btn">Uninstall</button>
            </div>
        `;

        card.querySelector('.configure-btn').addEventListener('click', () => {
            if (window.app && window.app.showToast) {
                window.app.showToast('Configure: ' + (skill.name || 'Unknown'));
            }
        });

        card.querySelector('.uninstall-btn').addEventListener('click', () => {
            this.uninstallSkill(skill.name);
        });

        return card;
    }

    renderMarketplace() {
        const container = document.querySelector('.marketplace-grid');
        if (!container) return;

        container.innerHTML = '';
        this.marketplace.forEach(skill => {
            const card = document.createElement('div');
            card.className = 'marketplace-card';
            card.innerHTML = `
                <div class="skill-header">
                    <span class="skill-icon">📦</span>
                    <span class="skill-name">${skill.name}</span>
                    <span class="skill-author">by ${skill.author}</span>
                </div>
                <div class="skill-description">${skill.description}</div>
                <div class="skill-stats">
                    <span>⬇ ${skill.downloads || 0}</span>
                    <span>⭐ ${skill.rating || 0}</span>
                </div>
                <button class="btn primary small install-btn">Install</button>
            `;

            card.querySelector('.install-btn').addEventListener('click', () => {
                this.installSkill(skill.name);
            });

            container.appendChild(card);
        });
    }

    async installSkill(name) {
        try {
            await fetch('/api/skills/install', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name }),
            });
            if (window.app && window.app.showToast) {
                window.app.showToast('Installed: ' + name);
            }
            this.loadInstalled();
        } catch (e) {
            console.error('Failed to install skill:', e);
            if (window.app && window.app.showToast) {
                window.app.showToast('Install failed: ' + name);
            }
        }
    }

    async uninstallSkill(name) {
        try {
            await fetch('/api/skills/uninstall', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name }),
            });
            if (window.app && window.app.showToast) {
                window.app.showToast('Uninstalled: ' + name);
            }
            this.loadInstalled();
        } catch (e) {
            console.error('Failed to uninstall skill:', e);
            if (window.app && window.app.showToast) {
                window.app.showToast('Uninstall failed: ' + name);
            }
        }
    }

    showPublishForm() {
        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = `
            <div class="modal-content">
                <h3>Publish Skill</h3>
                <input type="text" id="skill-name" placeholder="Skill name" class="input">
                <input type="text" id="skill-version" placeholder="Version" class="input">
                <textarea id="skill-description" placeholder="Description" class="input" rows="3"></textarea>
                <input type="file" id="skill-file" accept=".tar.gz,.zip">
                <button class="btn primary" onclick="skillsManager.publish()">Publish</button>
                <button class="btn outline" onclick="this.closest('.modal').remove()">Cancel</button>
            </div>
        `;
        document.body.appendChild(modal);
    }

    async publish() {
        const name = document.getElementById('skill-name').value;
        const version = document.getElementById('skill-version').value;
        const description = document.getElementById('skill-description').value;
        const file = document.getElementById('skill-file').files[0];

        if (!name || !file) return;

        const formData = new FormData();
        formData.append('name', name);
        formData.append('version', version);
        formData.append('description', description);
        formData.append('file', file);

        try {
            await fetch('/api/skills/publish', { method: 'POST', body: formData });
            this.closeModals();
            this.loadMarketplace();
            if (window.app && window.app.showToast) {
                window.app.showToast('Published: ' + name);
            }
        } catch (e) {
            console.error('Failed to publish skill:', e);
            if (window.app && window.app.showToast) {
                window.app.showToast('Publish failed: ' + name);
            }
        }
    }

    closeModals() {
        document.querySelectorAll('.modal').forEach(m => m.remove());
    }
}

const skillsManager = new SkillManager();
export default skillsManager;
