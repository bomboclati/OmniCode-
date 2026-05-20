class DocsPanel {
    constructor() {
        this.content = '';
    }

    init() {
        this.loadDocs();
        this.setupRegenerate();
    }

    async loadDocs() {
        try {
            const resp = await fetch('/api/docs');
            this.content = await resp.text();
            this.render();
        } catch (e) {
            console.warn('Failed to load docs:', e);
        }
    }

    render() {
        const container = document.querySelector('.docs-content');
        if (!container) return;

        const html = this.markdownToHtml(this.content);
        container.innerHTML = `
            <div class="docs-toolbar">
                <button class="btn secondary small" onclick="docsPanel.regenerate()">🔄 Regenerate</button>
                <button class="btn secondary small" onclick="docsPanel.exportPdf()">📄 Export PDF</button>
            </div>
            <div class="docs-body">${html}</div>
        `;

        // Render Mermaid diagrams
        if (window.mermaid) {
            mermaid.run({ nodes: container.querySelectorAll('.mermaid') });
        }

        // Table of contents
        this.generateToc(container);
    }

    markdownToHtml(md) {
        let html = md
            .replace(/^### (.+)$/gm, '<h3>$1</h3>')
            .replace(/^## (.+)$/gm, '<h2>$1</h2>')
            .replace(/^# (.REFERENCE)$/gm, '<h1>$1</h1>')
            .replace(/`([^`]+)`/g, '<code>$1</code>')
            .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
            .replace(/^- (.+)$/gm, '<li>$1</li>')
            .replace(/(<li>.*<\/li>\n?)+/g, '<ul>$&</ul>')
            .replace(/^(\d+)\. (.+)$/gm, '<li>$2</li>')
            .replace(/(<li>.*<\/li>\n?)+/g, match => {
                if (!match.startsWith('<ul>')) return '<ol>' + match + '</ol>';
                return match;
            })
            .replace(/\n\n/g, '</p><p>')
            .replace(/^(.+)$/gm, (match) => {
                if (match.startsWith('<') || match.startsWith('```') || match.trim() === '') return match;
                return match;
            });

        html = '<p>' + html + '</p>';
        html = html.replace(/```mermaid\n([\s\S]*?)```/g, '<div class="mermaid">$1</div>');
        html = html.replace(/```(\w*)\n([\s\S]*?)```/g, '<pre><code>$2</code></pre>');

        return html;
    }

    generateToc(container) {
        const headings = container.querySelectorAll('h1, h2, h3');
        if (headings.length < 3) return;

        const toc = document.createElement('div');
        toc.className = 'docs-toc';
        toc.innerHTML = '<h4>Table of Contents</h4>';

        headings.forEach(h => {
            const id = h.textContent.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
            h.id = id;
            const link = document.createElement('a');
            link.href = '#' + id;
            link.textContent = h.textContent;
            link.style.paddingLeft = (parseInt(h.tagName[1]) - 1) * 12 + 'px';
            toc.appendChild(link);
        });

        container.prepend(toc);
    }

    setupRegenerate() {
        document.addEventListener('click', (e) => {
            if (e.target.closest('[data-action="regenerate-docs"]')) {
                this.regenerate();
            }
        });
    }

    async regenerate() {
        if (window.app && window.app.showToast) {
            window.app.showToast('Regenerating docs...');
        }
        try {
            const resp = await fetch('/api/docs/regenerate', { method: 'POST' });
            if (resp.ok) {
                this.loadDocs();
                if (window.app && window.app.showToast) {
                    window.app.showToast('Docs regenerated');
                }
            }
        } catch (e) {
            console.error('Failed to regenerate docs:', e);
            if (window.app && window.app.showToast) {
                window.app.showToast('Regenerate failed');
            }
        }
    }

    async exportPdf() {
        const printWindow = window.open('', '_blank');
        if (!printWindow) return;

        printWindow.document.write(`
            <html><head><title>OmniCode Documentation</title>
            <style>
                body { font-family: 'Segoe UI', sans-serif; padding: 40px; line-height: 1.6; color: #333; }
                pre { background: #f4f4f4; padding: 12px; border-radius: 4px; overflow-x: auto; }
                code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }
                h1 { color: #8957FF; }
                h2 { color: #00FFA3; }
                @media print { body { padding: 20px; } }
            </style></head>
            <body>${document.querySelector('.docs-body')?.innerHTML || ''}</body></html>
        `);
        printWindow.document.close();
        setTimeout(() => printWindow.print(), 500);
    }
}

const docsPanel = new DocsPanel();
export default docsPanel;
