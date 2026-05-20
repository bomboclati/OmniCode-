class ReviewDashboard {
    constructor() {
        this.reviews = [];
    }

    init() {
        this.loadReviews();
    }

    async loadReviews() {
        try {
            const resp = await fetch('/api/reviews');
            this.reviews = await resp.json();
            this.render();
        } catch (e) {
            console.warn('Failed to load reviews:', e);
        }
    }

    render() {
        const container = document.querySelector('.reviews-list');
        if (!container) return;

        container.innerHTML = '';

        if (this.reviews.length === 0) {
            container.innerHTML = '<div class="empty-state">No pending reviews</div>';
            return;
        }

        this.reviews.forEach(review => {
            const item = document.createElement('div');
            item.className = 'review-item';
            item.innerHTML = `
                <div class="review-header">
                    <span class="review-id">#${review.id || '??'}</span>
                    <span class="review-title">${review.title || 'Untitled'}</span>
                    <span class="review-status badge ${review.status}">${review.status || 'pending'}</span>
                </div>
                <div class="review-meta">
                    <span>👤 ${review.author || 'unknown'}</span>
                    <span>📅 ${review.created_at || ''}</span>
                </div>
                <div class="review-actions">
                    <button class="btn primary small" onclick="reviewsDashboard.approve('${review.id}')">✅ Approve</button>
                    <button class="btn outline small" onclick="reviewsDashboard.requestChanges('${review.id}')">Request Changes</button>
                    <button class="btn outline small" onclick="reviewsDashboard.viewDiff('${review.id}')">View Diff</button>
                </div>
            `;
            container.appendChild(item);
        });
    }

    approve(id) {
        console.log(`Approving review: ${id}`);
        this.updateStatus(id, 'approved');
    }

    requestChanges(id) {
        console.log(`Requesting changes on: ${id}`);
        this.updateStatus(id, 'changes-requested');
    }

    viewDiff(id) {
        const review = this.reviews.find(r => r.id === id);
        if (!review) return;

        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = `
            <div class="modal-content" style="width:800px;max-width:95%;">
                <h3>${review.title}</h3>
                <div class="diff-viewer">
                    ${(review.diff || 'No diff available').split('\n').map(line =>
                        `<div class="diff-line ${line.startsWith('+') ? 'added' : line.startsWith('-') ? 'removed' : ''}">${line}</div>`
                    ).join('')}
                </div>
                <button class="btn outline" onclick="this.closest('.modal').remove()">Close</button>
            </div>
        `;
        document.body.appendChild(modal);
    }

    updateStatus(id, status) {
        const review = this.reviews.find(r => r.id === id);
        if (review) {
            review.status = status;
            this.render();
        }
    }
}

const reviewsDashboard = new ReviewDashboard();
export default reviewsDashboard;
