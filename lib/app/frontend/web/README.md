# dfps_web_frontend (Actix UI)

Serves the HTMX/Tailwind pages and proxies to the backend.

Env (loaded with `app.web.frontend`):
- `DFPS_FRONTEND_LISTEN_ADDR` (default `127.0.0.1:8090`)
- `DFPS_API_BASE_URL` (default `http://127.0.0.1:8080`)
- `DFPS_API_CLIENT_TIMEOUT_SECS` (default `15`)
- `DFPS_DOCS_URL` (optional) – when set (e.g., `http://127.0.0.1:3000`), `/docs` redirects there so you can surface an mdBook server.

Run:
```bash
cd code
cargo run -p dfps_web_frontend --bin dfps_web_frontend
```
