# DTO Layer

This directory groups the crates that expose **surface-specific DTO adapters**.
Each subdirectory is named after the surface (e.g., `web/`) and contains a
crate that re-exports the canonical contracts from `lib/domain/contracts`
without leaking unrelated payloads into that surface.

## Current crates

- `lib/dto/web` (`dfps_web_dto`) – the Actix frontend and Axum API both depend
  on this crate so analytics/cohort/eval/pipeline payloads stay in sync without
  duplicating structs in each app crate.

New DTO contexts should live under `lib/dto/<surface>` following the same
pattern: no env/config logic, only lightweight adapters around the canonical
domain contracts.
