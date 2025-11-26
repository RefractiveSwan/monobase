# UI smoke tests

Playwright-based smoke tests exercise the most important frontend flows without requiring a full e2e environment. They expect the frontend + API to run locally (see `docs/runbook/frontend-workbench.md`).

## Prerequisites

- Node.js 18+
- Frontend/web server listening at `http://127.0.0.1:8090` (override via `FRONTEND_BASE_URL`).

## Install

```bash
cd tools/ui-smoke-tests
npm install
npx playwright install --with-deps
```

## Run

```bash
npm test
```

The default spec hits three flows:

1. Mapping workbench – loads sample template, verifies validation summary + download links.
2. Observability – checks that the log panel poll target renders.
3. Evaluation – ensures dataset catalog renders tiers and that the scheduler form posts queue updates.

Use `npm run codegen` to capture screenshots for docs/book or to tweak selectors interactively.
