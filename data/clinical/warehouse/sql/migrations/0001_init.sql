-- Datamart schema bootstrap
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS dim_patient (
  patient_key INTEGER PRIMARY KEY,
  patient_id TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS dim_encounter (
  encounter_key INTEGER PRIMARY KEY,
  encounter_id TEXT NOT NULL,
  patient_key INTEGER NOT NULL REFERENCES dim_patient(patient_key)
);

CREATE TABLE IF NOT EXISTS dim_code (
  code_key INTEGER PRIMARY KEY,
  code_element_id TEXT NOT NULL,
  system TEXT,
  code TEXT,
  display TEXT
);

CREATE TABLE IF NOT EXISTS dim_ncit (
  ncit_key INTEGER PRIMARY KEY,
  ncit_id TEXT NOT NULL,
  preferred_name TEXT,
  semantic_group TEXT
);

CREATE TABLE IF NOT EXISTS fact_service_request (
  sr_id TEXT PRIMARY KEY,
  patient_key INTEGER NOT NULL REFERENCES dim_patient(patient_key),
  encounter_key INTEGER REFERENCES dim_encounter(encounter_key),
  code_key INTEGER NOT NULL REFERENCES dim_code(code_key),
  ncit_key INTEGER REFERENCES dim_ncit(ncit_key),
  status TEXT,
  intent TEXT,
  description TEXT,
  ordered_at TEXT
);
