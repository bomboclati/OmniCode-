class DecisionSearch {
    constructor() {
        this.decisions = [];
        this.tags = [];
    }

    init() {
        this.loadDecisions();
        this.setupSearch();
    }

    async loadDecisions() {
        try {
            const resp = await fetch('/api/decisions');
            this.decisions = await resp.json();
            this.renderDecisions(this.decisions);
            this.extractTags();
        } catch (e) {
            console.warn('Failed to load decisions:', e);
        }
    }

    setupSearch() {
        const searchInput = document.querySelector('.decision-search input');
        if (!searchInput) return;

        searchInput.addEventListener('input', () => {
            const query = searchInput.value.toLowerCase();
            const filtered = this.decisions.filter(d =>
                (d.title || '').toLowerCase().includes(query) ||
                (d.summary || '').toLowerCase().includes(query)
            );
            this.renderDecisions(filtered);
        });
    }

    renderDecisions(decisions) {
        const container = document.querySelector('.decisions-list');
        if (!container) return;

        container.innerHTML = '';

        if (decisions.length === 0) {
            container.innerHTML = '<div class="empty-state">No decisions recorded yet</div>';
            return;
        }

        decisions.forEach(decision => {
            const item = document.createElement('div');
            item.className = 'decision-item';
            item.innerHTML = `
                <div class="decision-date">${decision.date ? decision.date.slice(0, 10) : ''}</div>
                <div class="decision-content">
                    <h3 class="decision-title">${decision.title || 'Untitled'}</h3>
                    <p class="decision-summary">${decision.summary || ''}</p>
                    ${decision.tags ? `<div class="decision-tags">
                        ${(JSON.parse(decision.tags) || []).map(t => `<span class="tag">${t}</span>`).join('')}
                    </div>` : ''}
                </div>
                <button class="btn secondary small" onclick="decisionSearch.showFull('${decision.id}')">View</button>
            `;
            container.appendChild(item);
        });
    }

    showFull(id) {
        const decision = this.decisions.find(d => d.id === id);
        if (!decision) return;

        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = `
            <div class="modal-content" style="width:700px;max-height:80vh;overflow-y:auto;">
                <h2>${decision.title || ''}</h2>
                <div class="decision-meta">
                    <span>📅 ${decision.date ? decision.date.slice(0, 10) : ''}</span>
                </div>
                <div class="decision-body">
                    <h4>Summary</h4>
                    <p>${decision.summary || ''}</p>
                    <h4>Full Context</h4>
                    <p>${decision.full_text || decision.context || ''}</p>
                </div>
                ${decision.pr_links ? `<div class="decision-links">
                    <h4>Related PRs</h4>
                    ${JSON.parse(decision.pr_links).map((link, i) =>
                        `<a href="${link}" target="_blank">PR ${i + 1}</a>`
                    ).join(', ')}
                </div>` : ''}
                <button class="btn outline" onclick="this.closest('.modal').remove()">Close</button>
            </div>
        `;
        document.body.appendChild(modal);
    }

    extractTags() {
        const tagSet = new Set();
        this.decisions.forEach(d => {
            try {
                const tags = JSON.parse(d.tags || '[]');
                tags.forEach(t => tagSet.add(t));
            } catch (e) {}
        });

        this.tags = Array.from(tagSet);
        this.renderTagCloud();
    }

    renderTagCloud() {
        const container = document.querySelector('.tag-cloud');
        if (!container || this.tags.length === 0) return;

        container.innerHTML = '';
        this.tags.forEach(tag => {
            const span = document.createElement('span');
            span.className = 'tag';
            span.textContent = tag;
            span.addEventListener('click', () => {
                const searchInput = document.querySelector('.decision-search input');
                if (searchInput) {
                    searchInput.value = tag;
                    searchInput.dispatchEvent(new Event('input'));
                }
            });
            container.appendChild(span);
        });
    }
}

const decisionSearch = new DecisionSearch();
export default decisionSearch;
