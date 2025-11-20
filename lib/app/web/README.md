# DFPS Web Layer

This directory hosts shared assets for the HTTP surfaces:

- `lib/app/frontend/web` – Actix-based UI (HTMX) that talks to the backend.
- `lib/app/servers/api` – Axum-based API server that exposes mapping/eval/analytics routes.
- `lib/app/web/dto` – shared DTO re-exports ensuring the frontend client and backend handlers use the same payloads.

## DTO & config sharing

- DTOs: use the `dfps_web_dto` crate whenever you need analytics/cohort/eval/pipeline payloads in web code. It re-exports the `dfps_contracts` structs so we do not fork shapes across crates.
- Config: all web crates pull env via `dfps_configuration` namespaces:
  - Frontend: `app.web.frontend` (`DFPS_FRONTEND_*`).
  - API: `app.web.api` (`DFPS_API_*`, `DFPS_WAREHOUSE_*`).
  - Vector/datamart/etc. are passed in from the platform adapters.

## Ownership boundaries

- Backend routes depend on domain crates and platform adapters but never on Actix components.
- Frontend depends on the DTO crate + backend client; it should not recreate DTOs.
- Shared docs/README live here for the web layer.
