# Web Quickstart - DFPS Mapping Workbench

This runbook teaches new contributors how to run the DFPS web experience locally. It walks through the backend HTTP gateway, the frontend UI shell, the relevant environment variables, and the expected end-to-end flows so you can validate the FHIR -> NCIt pipeline in under ten minutes.

---

## 1. Components at a glance

| Component | Path | Crate | Purpose |
| --- | --- | --- | --- |
| Backend API | `code/lib/app/web/backend/api` | `dfps_api` | Axum HTTP gateway that exposes `/api/map-bundles`, `/metrics/summary`, and `/health` by delegating to `dfps_pipeline`. |
| Frontend UI | `code/lib/app/web/frontend` | `dfps_web_frontend` | Actix server that renders Tailwind/HTMX pages, proxies uploads/paste actions to the backend, and shows metrics/NoMatch explorer views. |
| Shared fixtures | `code/lib/platform/test_suite/src/regression.rs` | - | Contains helper functions that emit baseline FHIR bundles used in tests and manual runs. |

Both binaries live in the main Cargo workspace, so `cargo run -p <crate>` works anywhere under `code/`.

---

## 2. Prerequisites

- **Rust toolchain**: use the workspace-configured toolchain (see `.rust-toolchain.toml`). Install via `rustup`.
- **Cargo**: bundled with Rust. Needed to run both binaries and tests.
- **`jq` (optional)**: handy for inspecting JSON responses when using curl.
- **Browser**: any modern browser for the frontend UI.

---

## 3. Configure environment

Before launching the web surfaces, follow `docs/runbook/env-quickstart.md` to copy the appropriate `.env.<namespace>.example` files into `data/environment/` and set `DFPS_ENV` / `DFPS_ENV_DIR` as needed. Once the backend (`app.web.api`) and frontend (`app.web.frontend`) env files are in place, return here to start the services.

### Serve mdBook docs

If you want `/docs` in the frontend to redirect to an mdBook instance:

1. Build the book once: `cargo make docs`
2. Run it in another terminal: `cargo make docs-serve` (defaults to `http://127.0.0.1:3000`)
3. Export `DFPS_DOCS_URL=http://127.0.0.1:3000` before running the frontend so `/docs` redirects there.

## 4. Start the backend API

In terminal **A**:

```bash
cd code
cargo run -p dfps_api --bin dfps_api
```

What to expect:

- The server logs `starting web backend on 127.0.0.1:8080`.
- Endpoints available:
  - `POST http://127.0.0.1:8080/api/map-bundles`
  - `GET http://127.0.0.1:8080/metrics/summary`
  - `GET http://127.0.0.1:8080/health`

If the port is in use, pick another (e.g., `0.0.0.0:9090`) and remember to update `DFPS_API_BASE_URL` for the frontend.

---

## 5. Start the frontend UI

In terminal **B**:

```bash
cd code
DFPS_API_BASE_URL=http://127.0.0.1:8080 \
DFPS_FRONTEND_LISTEN_ADDR=127.0.0.1:8090 \
cargo run -p dfps_web_frontend --bin dfps_web_frontend
```

Key files:

- `src/routes.rs`: handles `/` (page render), `/map/paste`, `/map/upload`.
- `src/views.rs`: Maud templates for hero section, metrics dashboard, mapping results, and NoMatch explorer.
- `src/client.rs`: reqwest wrapper that speaks to the backend API.

When the server starts it logs `Listening on 127.0.0.1:8090`. Open `http://127.0.0.1:8090` to reach the UI.

---

## 6. Sample bundle payload

Create `sample-bundle.json` somewhere convenient:

```json
{
  "resourceType": "Bundle",
  "type": "collection",
  "entry": [
    {
      "resource": {
        "resourceType": "ServiceRequest",
        "id": "SR-1",
        "status": "active",
        "intent": "order",
        "code": {
          "coding": [
            {
              "system": "http://loinc.org",
              "code": "24606-6",
              "display": "FDG uptake"
            }
          ]
        },
        "subject": { "reference": "Patient/example" }
      }
    }
  ]
}
```

This mirrors the regression fixtures and drives an `AutoMapped` result that references NCIt concept `C1234`.

---

## 7. Example flows

### 7.1. Curl the backend

```bash
curl -X POST http://127.0.0.1:8080/api/map-bundles \
     -H "content-type: application/json" \
     --data-binary @sample-bundle.json | jq '.mapping_results'
```

Checklist:

- Response array contains at least one object with `state` == `AutoMapped`.
- If `jq` is unavailable, omit the pipe and inspect raw JSON.

### 7.2. Browse the frontend

1. Visit `http://127.0.0.1:8090`.
2. Scroll to “Paste Bundle JSON”, paste the sample JSON (or upload the file).
3. Submit. HTMX updates the results card without a full reload.
4. Observe:
   - Hero badges show backend health and metrics.
   - “MappingResult rows” table lists the NCIt concept, state badge, and `MappingResult.reason` (if any).
   - The NoMatch explorer populates when the backend emits `MappingState::NoMatch`.

If the backend is offline, the hero displays a red “Backend warning” card with troubleshooting hints.

---

## 8. Running tests

Frontend crate:

```bash
cargo test -p dfps_web_frontend
```

Included tests:

- `routes::tests::submitting_bundle_renders_mapping_rows` - spins up a mocked backend (wiremock) and exercises the `/map/paste` handler.
- `views::tests::render_page_shows_metrics_and_no_match_details` - snapshot-style assertion that the HTML includes key strings/states.
- `view_model` unit test verifying NoMatch derivation logic.

Backend integration tests live in `lib/platform/test_suite/tests/integration/web_api.rs`. Run them with:

```bash
cargo test -p dfps_test_suite --tests web_api
```

---

## 9. Troubleshooting

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| Frontend hero shows “Backend warning: Health endpoint unreachable” | Backend not running or wrong `DFPS_API_BASE_URL`. | Start `dfps_api` and confirm the URL matches the actual port. |
| Upload/paste returns “Backend error: status 500 ...” | Backend rejected the JSON (invalid FHIR or not an array/Bundle). | Validate the payload; compare with `sample-bundle.json` or use fixtures from `dfps_test_suite::regression`. |
| Curl works but frontend shows blank results | Frontend displays a friendly message when zero `MappingResult` rows come back. | Ensure the bundle includes codes under `stg_sr_code_exploded` by checking backend logs. |
| Port already in use errors | Another service bound to `8080` or `8090`. | Change the respective env vars to free ports. |

