const CACHE_NAME = 'omnicode-v1';
const ASSETS = [
  './',
  './assets/css/style.css',
  './assets/js/app.js',
  './assets/js/sync.js',
  './assets/js/editor.js',
  './assets/js/terminal.js',
  './assets/js/voice.js',
  './assets/js/collaboration.js',
  './assets/js/skills.js',
  './assets/js/reviews.js',
  './assets/js/docs.js',
  './assets/js/compliance.js',
  './assets/js/incidents.js',
  './assets/js/onboarding.js',
  './assets/js/deps.js',
  './assets/js/decisions.js',
  './assets/manifest.json',
  './assets/icon/favicon.svg',
  './assets/icon/favicon.ico',
  './assets/logo/omnicode-logo-mobile.svg',
  './assets/logo/omnicode-logo-horizontal.svg',
  './assets/logo/omnicode-logo-full.svg',
  './assets/logo/png/logo-512.png',
  './assets/logo/png/logo-256.png',
  './assets/logo/png/logo-192.png',
  './assets/logo/png/logo-180.png',
  './assets/logo/png/logo-152.png',
  './assets/logo/png/logo-128.png',
  './assets/logo/png/logo-120.png',
  './assets/logo/png/logo-64.png',
  './assets/logo/png/logo-48.png',
  './assets/logo/png/logo-32.png',
  './assets/logo/png/logo-16.png',
  './assets/logo/png/logo-mobile-512.png',
  './assets/logo/png/logo-mobile-192.png',
  './assets/logo/png/logo-mobile-180.png',
  './assets/logo/png/logo-mobile-152.png',
  './assets/logo/png/logo-mobile-120.png',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(ASSETS);
    })
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) => {
      return Promise.all(
        keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k))
      );
    })
  );
  self.clients.claim();
});

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') return;

  if (event.request.url.includes('/api/') || event.request.url.includes('/ws')) {
    event.respondWith(
      fetch(event.request).catch(() => {
        return new Response(
          JSON.stringify({ error: 'Offline', success: false }),
          { headers: { 'Content-Type': 'application/json' } }
        );
      })
    );
    return;
  }

  event.respondWith(
    caches.match(event.request).then((cached) => {
      if (cached) return cached;

      return fetch(event.request).then((response) => {
        if (response.ok) {
          const clone = response.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(event.request, clone);
          });
        }
        return response;
      }).catch(() => {
        if (event.request.destination === 'document') {
          return caches.match('./');
        }
        return new Response('Offline', { status: 503 });
      });
    })
  );
});

self.addEventListener('push', (event) => {
  if (!event.data) return;
  const data = event.data.json();
  self.registration.showNotification(data.title || 'OmniCode', {
    body: data.body || '',
    icon: './assets/logo/png/logo-128.png',
    badge: './assets/logo/png/logo-32.png',
    tag: data.tag || 'default',
  });
});
