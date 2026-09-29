# ui/ — Brain UI source (daemon `GET /ui`)

Single Brain UI surface. Files here are embedded into the `kurultai` binary
(`src/http/ui.rs`) and served at `http://127.0.0.1:8421/ui`.

- Entry: `brain.html` (explorer + synapse map)
- Legacy `/ui/index.html` (and css/js) 301 to `/ui/`

Optional Vite preview: see `website/README.md`. Do not create a second dashboard
tree (`website/` product, `web/` auth portal as brain UI, or `DASHBOARD_HTML` forks).
