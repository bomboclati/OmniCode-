class MobileUI {
    constructor(app) {
        this.app = app;
        this.isMobile = window.innerWidth <= 768;
        this.touchStartX = 0;
        this.touchStartY = 0;
        this.touchEndX = 0;
        this.swipeThreshold = 80;
        this.currentMobileScreen = 'chat';
        this.bottomNavItems = ['chat', 'files', 'swarm', 'download'];
        this.allScreens = ['chat', 'files', 'swarm', 'reviews', 'incidents', 'docs', 'deps', 'decisions', 'skills', 'compliance', 'onboarding', 'deploy', 'download'];
        this.drawerOpen = false;
        this.collapsedSections = new Set();
        this.init();
    }

    init() {
        this.setupResizeListener();
        this.setupTouchGestures();
        this.setupBottomNav();
        this.setupDrawer();
        this.setupMobileOptimizations();
        if (this.isMobile) {
            this.buildMobileLayout();
        }
    }

    setupResizeListener() {
        window.addEventListener('resize', () => {
            const wasMobile = this.isMobile;
            this.isMobile = window.innerWidth <= 768;
            if (this.isMobile && !wasMobile) {
                this.buildMobileLayout();
            } else if (!this.isMobile && wasMobile) {
                this.removeMobileLayout();
            }
        });
    }

    setupTouchGestures() {
        const mainContent = document.getElementById('main-content');
        if (!mainContent) return;

        mainContent.addEventListener('touchstart', (e) => {
            this.touchStartX = e.changedTouches[0].screenX;
            this.touchStartY = e.changedTouches[0].screenY;
        }, { passive: true });

        mainContent.addEventListener('touchend', (e) => {
            this.touchEndX = e.changedTouches[0].screenX;
            const touchEndY = e.changedTouches[0].screenY;
            this.handleSwipe();
        }, { passive: true });

        mainContent.addEventListener('touchmove', (e) => {
        }, { passive: true });
    }

    handleSwipe() {
        const diffX = this.touchEndX - this.touchStartX;
        const diffY = Math.abs(this.touchStartY - (this.touchEndY || this.touchStartY));

        if (Math.abs(diffX) < this.swipeThreshold) return;
        if (diffY > Math.abs(diffX)) return;

        const currentIdx = this.allScreens.indexOf(this.currentMobileScreen);

        if (diffX > 0 && currentIdx > 0) {
            this.switchMobileScreen(this.allScreens[currentIdx - 1]);
            this.vibrate(10);
        } else if (diffX < 0 && currentIdx < this.allScreens.length - 1) {
            this.switchMobileScreen(this.allScreens[currentIdx + 1]);
            this.vibrate(10);
        }
    }

    vibrate(ms) {
        if (navigator.vibrate) {
            navigator.vibrate(ms);
        }
    }

    setupBottomNav() {
        document.addEventListener('click', (e) => {
            const tab = e.target.closest('.mobile-nav-tab');
            if (tab) {
                e.preventDefault();
                const screen = tab.dataset.screen;
                this.switchMobileScreen(screen);
                this.vibrate(5);
            }

            const moreBtn = e.target.closest('.mobile-more-btn');
            if (moreBtn) {
                e.preventDefault();
                this.toggleDrawer();
                this.vibrate(5);
            }

            const drawerItem = e.target.closest('.drawer-item');
            if (drawerItem) {
                e.preventDefault();
                const screen = drawerItem.dataset.screen;
                this.switchMobileScreen(screen);
                this.closeDrawer();
                this.vibrate(5);
            }

            const drawerClose = e.target.closest('.drawer-close') || e.target.closest('.drawer-backdrop');
            if (drawerClose) {
                this.closeDrawer();
            }

            const collapseHeader = e.target.closest('.section-collapse-header');
            if (collapseHeader) {
                const section = collapseHeader.dataset.section;
                this.toggleSection(section);
            }
        });
    }

    setupDrawer() {
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && this.drawerOpen) {
                this.closeDrawer();
            }
        });
    }

    setupMobileOptimizations() {
        const inputs = document.querySelectorAll('input, textarea, select');
        inputs.forEach(input => {
            input.addEventListener('focus', () => {
                if (this.isMobile) {
                    setTimeout(() => {
                        input.scrollIntoView({ behavior: 'smooth', block: 'center' });
                    }, 300);
                }
            });
        });

        document.addEventListener('gesturestart', (e) => {
            e.preventDefault();
        });
    }

    buildMobileLayout() {
        if (document.getElementById('mobile-bottom-nav')) return;

        const bottomNav = document.createElement('nav');
        bottomNav.id = 'mobile-bottom-nav';
        bottomNav.className = 'mobile-bottom-nav';
        bottomNav.innerHTML = `
            ${this.bottomNavItems.map(s => `
                <button class="mobile-nav-tab ${s === this.currentMobileScreen ? 'active' : ''}" data-screen="${s}">
                    ${this.getTabIcon(s)}
                    <span>${this.getTabLabel(s)}</span>
                </button>
            `).join('')}
            <button class="mobile-nav-tab mobile-more-btn" data-screen="more">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <circle cx="5" cy="12" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="19" cy="12" r="2"/>
                </svg>
                <span>More</span>
            </button>
        `;

        const app = document.getElementById('app');
        if (app) {
            app.appendChild(bottomNav);
        }

        this.buildDrawer();

        const sidebar = document.getElementById('sidebar');
        if (sidebar) {
            sidebar.style.display = 'none';
        }

        const panelTabs = document.getElementById('panel-tabs');
        if (panelTabs) {
            panelTabs.style.display = 'none';
        }

        this.updateMobileNavActive(this.currentMobileScreen);
    }

    removeMobileLayout() {
        const bottomNav = document.getElementById('mobile-bottom-nav');
        if (bottomNav) bottomNav.remove();

        const drawer = document.getElementById('mobile-drawer');
        if (drawer) drawer.remove();

        const sidebar = document.getElementById('sidebar');
        if (sidebar) {
            sidebar.style.display = '';
        }
    }

    buildDrawer() {
        if (document.getElementById('mobile-drawer')) return;

        const otherScreens = this.allScreens.filter(s => !this.bottomNavItems.includes(s));

        const drawer = document.createElement('div');
        drawer.id = 'mobile-drawer';
        drawer.className = 'mobile-drawer';
        drawer.innerHTML = `
            <div class="drawer-backdrop"></div>
            <div class="drawer-content">
                <div class="drawer-header">
                    <span class="drawer-title">More</span>
                    <button class="drawer-close" aria-label="Close drawer">
                        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                            <line x1="4" y1="4" x2="16" y2="16"/><line x1="16" y1="4" x2="4" y2="16"/>
                        </svg>
                    </button>
                </div>
                <div class="drawer-items">
                    ${otherScreens.map(s => `
                        <button class="drawer-item" data-screen="${s}">
                            ${this.getTabIcon(s)}
                            <span>${this.getTabLabel(s)}</span>
                        </button>
                    `).join('')}
                </div>
            </div>
        `;

        document.body.appendChild(drawer);
    }

    getTabIcon(screen) {
        const icons = {
            chat: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>',
            files: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>',
            swarm: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><circle cx="5" cy="6" r="2"/><circle cx="19" cy="6" r="2"/><circle cx="5" cy="18" r="2"/><circle cx="19" cy="18" r="2"/><line x1="7" y1="7" x2="10" y2="10"/><line x1="17" y1="7" x2="14" y2="10"/><line x1="7" y1="17" x2="10" y2="14"/><line x1="17" y1="17" x2="14" y2="14"/></svg>',
            reviews: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>',
            incidents: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>',
            docs: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/></svg>',
            deps: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>',
            decisions: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>',
            skills: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>',
            compliance: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>',
            onboarding: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>',
            deploy: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>',
            download: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>',
        };
        return icons[screen] || '';
    }

    getTabLabel(screen) {
        const labels = {
            chat: 'Chat',
            files: 'Files',
            swarm: 'Swarm',
            reviews: 'Reviews',
            incidents: 'Alerts',
            docs: 'Docs',
            deps: 'Deps',
            decisions: 'Decide',
            skills: 'Skills',
            compliance: 'Secure',
            onboarding: 'Guide',
            deploy: 'Deploy',
            download: 'Get App',
        };
        return labels[screen] || screen;
    }

    switchMobileScreen(screen) {
        this.currentMobileScreen = screen;
        this.updateMobileNavActive(screen);

        if (this.app && this.app.switchScreen) {
            this.app.switchScreen(screen);
        }

        if (screen === 'download' && this.app && this.app.renderDownloadScreen) {
            this.app.renderDownloadScreen();
        }

        window.scrollTo({ top: 0, behavior: 'smooth' });
    }

    updateMobileNavActive(screen) {
        document.querySelectorAll('.mobile-nav-tab').forEach(tab => {
            tab.classList.toggle('active', tab.dataset.screen === screen);
        });

        document.querySelectorAll('.drawer-item').forEach(item => {
            item.classList.toggle('active', item.dataset.screen === screen);
        });
    }

    toggleDrawer() {
        if (this.drawerOpen) {
            this.closeDrawer();
        } else {
            this.openDrawer();
        }
    }

    openDrawer() {
        const drawer = document.getElementById('mobile-drawer');
        if (drawer) {
            drawer.classList.add('open');
            this.drawerOpen = true;
        }
    }

    closeDrawer() {
        const drawer = document.getElementById('mobile-drawer');
        if (drawer) {
            drawer.classList.remove('open');
            this.drawerOpen = false;
        }
    }

    toggleSection(section) {
        if (this.collapsedSections.has(section)) {
            this.collapsedSections.delete(section);
        } else {
            this.collapsedSections.add(section);
        }

        const content = document.getElementById(`section-${section}`);
        const header = document.querySelector(`[data-section="${section}"]`);
        if (content) {
            content.style.display = this.collapsedSections.has(section) ? 'none' : '';
        }
        if (header) {
            header.classList.toggle('collapsed', this.collapsedSections.has(section));
        }
    }

    isSectionCollapsed(section) {
        return this.collapsedSections.has(section);
    }
}

window.MobileUI = MobileUI;

if (window.app) {
    window.mobileUI = new MobileUI(window.app);
} else {
    document.addEventListener('DOMContentLoaded', () => {
        setTimeout(() => {
            if (window.app) {
                window.mobileUI = new MobileUI(window.app);
            }
        }, 100);
    });
}
