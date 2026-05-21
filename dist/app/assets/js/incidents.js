class IncidentDashboard {
    constructor() {
        this.incidents = [];
    }

    init() {
        this.setupWebSocket();
        this.render();
    }

    setupWebSocket() {
        if (window.sync) {
            window.sync.on('incident_alert', (data) => {
                this.addIncident(data);
            });
        }

        // Poll for incidents
        setInterval(() => this.loadIncidents(), 30000);
    }

    async loadIncidents() {
        try {
            const resp = await fetch('/api/incidents');
            this.incidents = await resp.json();
        } catch (e) {
            console.warn('Failed to load incidents, using demo data:', e);
            this.incidents = [
                { id: '1', timestamp: new Date(Date.now() - 120000).toISOString(), severity: 'warning', source: 'staging', message: 'NullPointer in UserService.getProfile()', fix_status: 'detected', stack_trace: 'thread 'main' panicked at src/services/user.rs:42:\ncalled `Option::unwrap()` on a `None` value' },
                { id: '2', timestamp: new Date(Date.now() - 3600000).toISOString(), severity: 'error', source: 'production', message: 'Database connection pool exhausted (max: 20)', fix_status: 'approved', stack_trace: 'Error: ConnectionPoolTimeout\n   at sqlx::pool::Pool::acquire (pool.rs:156)' },
                { id: '3', timestamp: new Date(Date.now() - 7200000).toISOString(), severity: 'critical', source: 'api', message: 'Rate limiter rejecting valid requests (false positive)', fix_status: 'resolved', correlated_commit: 'a1b2c3d' },
            ];
        }
        this.render();
    }

    addIncident(data) {
        this.incidents.unshift({
            id: Date.now().toString(),
            timestamp: new Date().toISOString(),
            severity: data.severity || 'warning',
            source: data.source || 'unknown',
            message: data.message || 'Unknown incident',
            ...data
        });

        this.render();
        this.showNotification(data);
    }

    showNotification(data) {
        const severityIcons = { critical: '🔴', error: '🟠', warning: '🟡' };
        const icon = severityIcons[data.severity] || '🟡';

        if ('Notification' in window && Notification.permission === 'granted') {
            new Notification(`${icon} ${data.severity}`, { body: data.message });
        }
    }

    render() {
        const container = document.querySelector('.incidents-list');
        if (!container) return;

        container.innerHTML = '';

        if (this.incidents.length === 0) {
            container.innerHTML = '<div class="empty-state">No active incidents ✅</div>';
            return;
        }

        this.incidents.forEach(incident => {
            const severityIcons = { critical: '🔴', error: '🟠', warning: '🟡' };
            const icon = severityIcons[incident.severity] || '🟡';

            const item = document.createElement('div');
            item.className = `incident-item severity-${incident.severity}`;
            item.innerHTML = `
                <div class="incident-header">
                    <span class="incident-severity">${icon}</span>
                    <span class="incident-time">${incident.timestamp ? new Date(incident.timestamp).toLocaleTimeString() : ''}</span>
                    <span class="incident-source">${incident.source || ''}</span>
                    <span class="incident-status badge">${incident.fix_status || 'detected'}</span>
                </div>
                <div class="incident-message">${incident.message || ''}</div>
                <div class="incident-actions">
                    <button class="btn primary small" onclick="incidentsDashboard.approveFix('${incident.id}')">✅ Approve Fix</button>
                    <button class="btn outline small" onclick="incidentsDashboard.dismiss('${incident.id}')">Dismiss</button>
                    <button class="btn secondary small" onclick="incidentsDashboard.showDetails('${incident.id}')">Details</button>
                </div>
            `;
            container.appendChild(item);
        });
    }

    approveFix(id) {
        const incident = this.incidents.find(i => i.id === id);
        if (incident) {
            incident.fix_status = 'approved';
            this.render();
            if (window.app && window.app.showToast) {
                window.app.showToast('Fix approved: ' + id);
            }
        }
    }

    dismiss(id) {
        this.incidents = this.incidents.filter(i => i.id !== id);
        this.render();
        if (window.app && window.app.showToast) {
            window.app.showToast('Incident dismissed');
        }
    }

    showDetails(id) {
        const incident = this.incidents.find(i => i.id === id);
        if (!incident) return;

        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = `
            <div class="modal-content" style="width:600px;">
                <h3>Incident Details</h3>
                <div class="incident-detail">
                    <p><strong>Time:</strong> ${incident.timestamp || ''}</p>
                    <p><strong>Severity:</strong> ${incident.severity}</p>
                    <p><strong>Source:</strong> ${incident.source}</p>
                    <p><strong>Message:</strong> ${incident.message}</p>
                    ${incident.stack_trace ? `<pre>${incident.stack_trace}</pre>` : ''}
                    ${incident.correlated_commit ? `<p><strong>Correlated Commit:</strong> ${incident.correlated_commit}</p>` : ''}
                </div>
                <button class="btn outline" onclick="this.closest('.modal').remove()">Close</button>
            </div>
        `;
        document.body.appendChild(modal);
    }
}

const incidentsDashboard = new IncidentDashboard();
export default incidentsDashboard;
