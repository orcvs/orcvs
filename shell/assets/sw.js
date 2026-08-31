var cacheName = 'orcvs-pwa-v2';
var filesToCache = [
  './',
  './index.html',
  './shell.js',
  './shell_bg.wasm',
];

/* Start the service worker and cache all of the app's content */
self.addEventListener('install', function (e) {
  e.waitUntil(
    caches.open(cacheName).then(function (cache) {
      return cache.addAll(filesToCache);
    }).then(function () {
      return self.skipWaiting();
    })
  );
});

/* Remove artifacts cached by earlier service-worker releases. */
self.addEventListener('activate', function (e) {
  e.waitUntil(
    caches.keys().then(function (cacheNames) {
      return Promise.all(
        cacheNames
          .filter(function (name) {
            return name !== cacheName;
          })
          .map(function (name) {
            return caches.delete(name);
          })
      );
    }).then(function () {
      return self.clients.claim();
    })
  );
});

/* Serve cached content when offline */
self.addEventListener('fetch', function (e) {
  if (
    e.request.mode === 'navigate' ||
    e.request.url.endsWith('/shell.js') ||
    e.request.url.endsWith('/shell_bg.wasm')
  ) {
    e.respondWith(
      caches.open(cacheName).then(function (cache) {
        return fetch(e.request).then(function (response) {
          return cache.put(e.request, response.clone()).then(function () {
            return response;
          });
        }).catch(function () {
          return cache.match(e.request);
        });
      })
    );
    return;
  }

  e.respondWith(
    caches.open(cacheName).then(function (cache) {
      return cache.match(e.request).then(function (response) {
        return response || fetch(e.request);
      });
    })
  );
});
