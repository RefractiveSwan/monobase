 # Branching & Commit Conventions

## Branching
- Base: `main`; prefer **one card -> one feature branch**.

**Name**
```
<kind>/<card-id>-kebab-summary
```
Where `<kind>` ∈ {`feature`, `bugfix`, `chore`, `docs`, `spike`} and `<card-id>` is the epic/card ID in Kanban (e.g., `FP-07`, `MAP-03`).

**Branch ↔ Epic binding (required)**
- The **active Git branch must be recorded in the epic Kanban** you are working on.
- In the epic header, add or update:
  - `Branch: <branch-name>`
  - `Branch target version: <semver or Unreleased>`
- If the branch name does **not** contain the `<card-id>`, rename the branch to match the convention or note the exception in the epic.

**Epic header example**
```markdown
> Epic: FP-07 – Ingestion error surface
> Branch: feature/FP-07-ingestion-error-surface
> Branch target version: v0.2.1
> Status: INPROGRESS
> Introduced in: 2025-11-15
> Last updated in: v0.2.1
```

## Workflow

1. **TODO -> INPROGRESS**
   - Create the branch from `main` using the naming rule.
   - In the epic Kanban, ensure the header contains:
     - `Branch: <branch-name>`
     - `Branch target version: Unreleased` (or seed with the planned semver)
     - `Status: INPROGRESS`

2. **Implement**
   - Code + tests + docs as usual.

3. **INPROGRESS -> REVIEW or DONE (leaving INPROGRESS)**
   - **Update the branch target version in the epic** (metadata only; do **not** bump Cargo here):
     - **PATCH** - routine/internal change
     - **MINOR** - user‑visible addition
     - **MAJOR** - breaking change
   - Set `Last updated in` to that version.
   - Keep `Introduced in: 2025-11-15` until a release PR.

4. **Merge -> `main`**
   - Move the card to **DONE**.
   - In the epic header, mark it as **included in the upcoming release** (e.g., add `Release: Upcoming` or an equivalent line).
   - Add/refresh a `## [Unreleased]` item in `CHANGELOG.md` referencing the epic ID and the **Branch target version**.

> **Note:** The workspace SemVer in `Cargo.toml` is only bumped in a **release PR** that collects all “Upcoming” epics. Until then, the epic carries the **Branch target version** as intent.

## Commits

**Format**
```
<type>(<card-id>[:<scope>]): short imperative summary
```
`<type>` ∈ {`feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `ci`}

**Examples**
```
feat(FP-07:ingestion): add IngestionError enum and error mapping
fix(MAP-05:mapping): correct NCIt concept id for PET code 78815
docs(DM-05): document core domain model and invariants
```

**Body tips**
- Bullet points of cross‑cutting changes
- Mention updated docs/fixtures
- `Tests:` line with what ran
- If Kanban metadata changed, note it explicitly, e.g.:
  - `Kanban: record Branch=feature/FP-07-...`
  - `Kanban: set Branch target version -> v0.2.1`
  - `CHANGELOG: add Unreleased entry (FP-07)`

## Pre‑merge checklist
- Epic Kanban has `Branch:` recorded and `Branch target version` set
- Card moved to **DONE**; epic marked **Upcoming** (or equivalent)
- `CHANGELOG.md` updated under `[Unreleased]`
- `cargo make fmt` · `cargo make clippy` · `cargo make test` all passing
