class DependencyPanel {
    constructor() {
        this.deps = [];
    }

    init() {
        this.loadDeps();
        setInterval(() => this.loadDeps(), 60000);
    }

    async loadDeps() {
        try {
            const resp = await fetch('/api/dependencies');
            this.deps = await resp.json();
            this.render();
        } catch (e) {
            console.warn('Failed to load dependencies:', e);
        }
    }

    render() {
        const container = document.querySelector('.deps-table');
        if (!container) return;

        container.innerHTML = '';

        if (this.deps.length === 0) {
            container.innerHTML = '<div class="empty-state">No dependency data</div>';
            return;
        }

        const table = document.createElement('table');
        table.className = 'deps-grid';

        table.innerHTML = `
            <thead>
                <tr>
                    <th>Package</th>
                    <th>Current</th>
                    <th>Latest</th>
                    <th>Status</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                ${this.deps.map(dep => `
                    <tr class="dep-row ${this.getRowClass(dep)}">
                        <td class="dep-name">${dep.package || dep.name || ''}</td>
                        <td>${dep.current_version || ''}</td>
                        <td>${dep.new_version || dep.latest_version || '—'}</td>
                        <td>${this.getStatusBadge(dep)}</td>
                        <td class="dep-actions">
                            ${dep.breaking_change ? '<span class="badge warning">⚠️ Breaking</span>' : ''}
                            ${dep.new_version ? `<button class="btn outline small" onclick="depsPanel.applyUpdate('${dep.package || dep.name}')">Update</button>` : ''}
                        </td>
                    </tr>
                `).join('')}
            </tbody>
        `;

        container.appendChild(table);
    }

    getRowClass(dep) {
        if (dep.new_version) return 'has-update';
        return '';
    }

    getStatusBadge(dep) {
        if (dep.security_advisory) return '<span class="badge error">🔴 Security</span>';
        if (dep.breaking_change) return '<span class="badge warning">🟡 Breaking</span>';
        if (dep.new_version) return '<span class="badge info">🔄 Update</span>';
        return '<span class="badge success">✅ Up to date</span>';
    }

    applyUpdate(name) {
        const dep = this.deps.find(d => (d.package || d.name) === name);
        if (!dep) return;

        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = `
            <div class="modal-content" style="width:500px;">
                <h3>Update ${name}</h3>
                <p>${dep.current_version} → ${dep.new_version}</p>
                ${dep.breaking_change ? '<p class="warning">⚠️ This is a breaking change update.</p>' : ''}
                ${dep.changelog_summary ? `<div class="changelog"><h4>Changelog</h4><p>${dep.changelog_summary}</p></div>` : ''}
                ${dep.affected_files?.length ? `
                    <div class="affected-files">
                        <h4>Affected Files</h4>
                        <ul>${dep.affected_files.map(f => `<li>${f}</li>`).join('')}</ul>
                    </div>
                ` : ''}
                <div class="modal-actions">
                    <button class="btn primary" onclick="depsPanel.confirmUpdate('${name}')">Apply Update</button>
                    <button class="btn outline" onclick="this.closest('.modal').remove()">Skip</button>
                </div>
            </div>
        `;
        document.body.appendChild(modal);
    }

    async confirmUpdate(name) {
        try {
            await fetch('/api/dependencies/update', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ package: name }),
            });
            this.closeModals();
            this.loadDeps();
        } catch (e) {
            console.error('Update failed:', e);
        }
    }

    closeModals() {
        document.querySelectorAll('.modal').forEach(m => m.remove());
    }
}

const depsPanel = new DependencyPanel();
export default depsPanel;
