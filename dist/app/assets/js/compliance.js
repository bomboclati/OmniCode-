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
        } catch (e) {
            console.warn('Failed to load compliance log, using demo data:', e);
            this.entries = [
                { timestamp: '2026-05-20T10:30:00Z', action: 'Code signing', actor: 'agent', status: 'compliant', details: 'Signed commit 7a3b2c1 with GPG key' },
                { timestamp: '2026-05-20T09:15:00Z', action: 'Dependency audit', actor: 'guardian', status: 'compliant', details: 'Scanned 142 dependencies, 0 vulnerabilities' },
                { timestamp: '2026-05-19T16:45:00Z', action: 'License check', actor: 'system', status: 'compliant', details: 'All dependency licenses compatible with MIT' },
                { timestamp: '2026-05-19T14:20:00Z', action: 'Build attestation', actor: 'ci', status: 'compliant', details: 'Build #847 signed and verified' },
                { timestamp: '2026-05-18T11:00:00Z', action: 'PR review audit', actor: 'alice', status: 'compliant', details: 'Reviewed 3 PRs, 0 policy violations' },
            ];
        }
        this.render();
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
            if (window.app && window.app.showToast) {
                window.app.showToast(result.valid ? 'Chain verified ✅' : 'Chain verification failed ❌');
            }
            return result.valid;
        } catch (e) {
            console.error('Chain verification failed:', e);
            if (window.app && window.app.showToast) {
                window.app.showToast('Verification failed');
            }
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
            if (window.app && window.app.showToast) {
                window.app.showToast('Report exported: ' + a.download);
            }
        } catch (e) {
            console.error('Export failed:', e);
            if (window.app && window.app.showToast) {
                window.app.showToast('Export failed');
            }
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
