# 06 — Cache the real WASM artifact names in the service worker

**What to fix:** `sw.js` caches `./shell.js` and `./shell_bg.wasm`. Trunk writes hashed names. The
two never match, `cache.addAll` rejects, and the service worker caches nothing.

**Status:** needs-triage

- [ ] The install handler completes and fills the cache.
- [ ] The application starts with no network.
- [ ] The tooling contract asserts something the build can satisfy.

## Comments

`shell/assets/sw.js:2-7` holds the list:

```js
var filesToCache = [
  './',
  './index.html',
  './shell.js',
  './shell_bg.wasm',
];
```

`shell/Trunk.toml` holds an empty `[build]` table, so Trunk uses its default file hashing. A build
from this branch wrote `shell-d13da21f30b80848.js` and `shell-d13da21f30b80848_bg.wasm` into
`shell/dist`. Neither name is in the list.

`cache.addAll` rejects if any request fails. The install handler passes that promise to
`e.waitUntil`, so one missing file fails the whole install. The result is not a partial cache. It is
no cache. The application has no offline support today, and it never had any.

`main` has the same defect with `./console.js` and `./console_bg.wasm`, so the crate split did not
cause it. The split did add a contract assertion for `'./shell_bg.wasm'`, and a later change added
one for `'./shell.js'`. Those assertions guard against a stale crate name, which is a real risk, but
they now assert names that the build never writes.

Triage is needed because the two repairs trade against each other.

Set `filehash = false` in `Trunk.toml`. The emitted names then match the list, and the contract
assertions become true. The cost is cache busting: a new build reuses the old file names, so a
browser can hold a stale script.

Or delete the two entries from `sw.js`. Hashing stays, and the install stops failing, so `./` and
`./index.html` are cached. The cost is that the script and the WASM module are not precached, which
is most of the payload.

A third repair keeps both properties and costs more work. Let the build write the file list into
`sw.js`, through a Trunk hook or a post-build step. Do not start here.

Whichever you choose, correct the contract assertions in the same change. An assertion that the
build cannot satisfy is worse than no assertion.
