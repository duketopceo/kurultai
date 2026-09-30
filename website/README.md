# website/ — optional Vite **preview** only

Canon Brain UI assets live in [`../ui/`](../ui/) and are embedded into the
`kurultai` daemon at compile time (`GET /ui`). This folder is not a product.

```bash
# terminal 1
kurultai daemon --port 8421
# terminal 2 — optional local preview of ../ui with /api proxy
cd website && npm install && npm run dev
```

Edit files under `website/src/`, then rebuild the embedded UI:

```bash
./scripts/build-ui.sh
```

This also cleans stale `ui/` hashed bundles and legacy landing-page assets so
they cannot be served by accident. Then rebuild the daemon so `/ui` picks up
changes.

Do not add a parallel dashboard under `website/`, `web/`, or another embedded
HTML string in Rust.
