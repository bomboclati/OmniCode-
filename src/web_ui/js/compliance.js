class ComplianceViewer {
    constructor() {
        this.entries = [];
    }

    init() {
        this.loadLog();
    }

    async loadLog() {
        try {
            const resp = await fetch('/api/compliance/log');
            this.entries = await resp.json();
            this.render();
        } catch (e) {
            console.warn('Failed to load compliance log:', e);
        }
    }

    render() {
        const container = document.querySelector('.compliance-timeline');
        if (!container) return;

        container.innerHTML = '';

        if (this.entries.length === 0) {
            container.innerHTML = '<div class="empty-state">No compliance entries yet</div>';
            return;
        }

        this.entries.forEach(entry => {
            const item = document.createElement('div');
            item.className = 'compliance-entry';
            item.innerHTML = `
                <div class="entry-timestamp">${entry.timestamp || ''}</div>
                <div class="entry-content">
                    <span class="entry-action">${entry.action || 'Unknown action'}</span>
                    <span class="entry-actor">by ${entry.actor || 'unknown'}</span>
                    <span class="entry-status badge ${entry.status || ''}">${entry.status || 'logged'}</span>
                </div>
                <div class="entry-details">${entry.details || ''}</div>
            `;
            container.appendChild(item);
        });
    }

    async verifyChain() {
        try {
            const resp = await fetch('/api/compliance/verify');
            const result = await resp.json();
            return result.valid;
        } catch (e) {
            console.error('Chain verification failed:', e);
            return false;
        }
    }

    async exportReport(format = 'json') {
        try {
            const resp = await fetch(`/api/compliance/export?format=${format}`);
            const blob = await resp.blob();
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `compliance-report.${format}`;
            a.click();
            URL.revokeObjectURL(url);
        } catch (e) {
            console.error('Export failed:', e);
        }
    }

    renderVerifyButton() {
        const btn = document.createElement('button');
        btn.className = 'btn primary';
        btn.textContent = '🔗 Verify Chain';
        btn.addEventListener('click', async () => {
            btn.textContent = 'Verifying...';
            btn.disabled = true;
            const valid = await this.verifyChain();
            btn.textContent = valid ? '✅ Chain Valid' : '❌ Chain Invalid';
            btn.className = valid ? 'btn success' : 'btn error';
            setTimeout(() => {
                btn.textContent = '🔗 Verify Chain';
                btn.className = 'btn primary';
                btn.disabled = false;
            }, 3000);
        });
        return btn;
    }
}

const complianceViewer = new ComplianceViewer();
export default complianceViewer;
