# dfps_api (Axum backend)

Endpoints:
- `POST /api/map-bundles` - accepts a Bundle, array of Bundles, or NDJSON
- `GET /metrics/summary`
- `GET /health`

Env: loaded via `dfps_configuration::load_env("app.web.api")`.
Defaults: `DFPS_API_HOST=127.0.0.1`, `DFPS_API_PORT=8080`.

Run:
```bash
cd code
cargo run -p dfps_api --bin dfps_api
```