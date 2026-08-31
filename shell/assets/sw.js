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
    })
  );
});

/* Serve cached content when offline */
self.addEventListener('fetch', function (e) {
  e.respondWith(
    caches.open(cacheName).then(function (cache) {
      return cache.match(e.request).then(function (response) {
        return response || fetch(e.request);
      });
    })
  );
});
