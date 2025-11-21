//! Compliance policy definitions for license-aware gating across mapping, CLI, and export surfaces.
//! See:
//! - docs/kanban/feature/mvp/020-license-compliance-layer.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-11--platform-compliance--export-gating-refractive_swan_compliance
//! - docs/system-design/clinical/fhir/concepts/terminology-layer.md
//! - docs/system-design/clinical/ncit/architecture.md

pub mod config;
mod policy;

pub use config::ComplianceConfig;
pub use policy::{
    ComplianceAction, ComplianceError, ComplianceMode, Policy, PolicyOverrides,
    assert_export_allowed,
};

/// Load a policy from environment variables and optional JSON/YAML override file.
pub fn load_policy_from_env() -> Result<Policy, ComplianceError> {
    ComplianceConfig::from_env()?.load_policy()
}

#[cfg(test)]
mod tests {
    use super::*;
    use refractive_swan_terminology::codesystem::LicenseTier;
    use std::{
        collections::BTreeMap,
        env, fs,
        sync::{Mutex, OnceLock},
    };

    static ENV_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_guard() -> &'static Mutex<()> {
        ENV_GUARD.get_or_init(|| Mutex::new(()))
    }

    fn set_env_var(key: &str, value: &str) {
        // SAFETY: tests serialize env mutation via a mutex to avoid races in parallel runs.
        unsafe { env::set_var(key, value) };
    }

    fn clear_env_var(key: &str) {
        // SAFETY: tests serialize env mutation via a mutex to avoid races in parallel runs.
        unsafe { env::remove_var(key) };
    }

    #[test]
    fn defaults_per_mode_allow_expected_tiers() {
        let internal = Policy::default_for_mode(ComplianceMode::Internal);
        assert!(internal.is_allowed(
            ComplianceAction::Map,
            refractive_swan_terminology::codesystem::LicenseTier::Licensed
        ));
        assert!(internal.is_allowed(
            ComplianceAction::Export,
            refractive_swan_terminology::codesystem::LicenseTier::InternalOnly
        ));

        let partner = Policy::default_for_mode(ComplianceMode::Partner);
        assert!(partner.is_allowed(
            ComplianceAction::Map,
            refractive_swan_terminology::codesystem::LicenseTier::Licensed
        ));
        assert!(!partner.is_allowed(
            ComplianceAction::Export,
            refractive_swan_terminology::codesystem::LicenseTier::InternalOnly
        ));

        let oss = Policy::default_for_mode(ComplianceMode::OpenSource);
        assert!(oss.is_allowed(
            ComplianceAction::Map,
            refractive_swan_terminology::codesystem::LicenseTier::Open
        ));
        assert!(!oss.is_allowed(
            ComplianceAction::Map,
            refractive_swan_terminology::codesystem::LicenseTier::Licensed
        ));
    }

    #[test]
    fn apply_overrides_updates_actions_and_tiers() {
        let base = Policy::default_for_mode(ComplianceMode::Internal);
        let mut overrides = PolicyOverrides::default();
        overrides.mode = Some(ComplianceMode::OpenSource);
        overrides.allowed_actions = Some(vec![ComplianceAction::Map]);
        overrides.allowed_tiers = Some(BTreeMap::from([(
            ComplianceAction::Map,
            vec![LicenseTier::Open],
        )]));

        let policy = base.apply_overrides(overrides);
        assert_eq!(policy.mode, ComplianceMode::OpenSource);
        assert!(policy.is_action_allowed(ComplianceAction::Map));
        assert!(!policy.is_action_allowed(ComplianceAction::Export));
        assert!(policy.is_allowed(ComplianceAction::Map, LicenseTier::Open));
        assert!(!policy.is_allowed(ComplianceAction::Map, LicenseTier::Licensed));
    }

    #[test]
    fn load_policy_from_env_respects_override_file() {
        let _lock = env_guard().lock().unwrap();

        clear_env_var("refractive_swan_COMPLIANCE_MODE");
        clear_env_var("refractive_swan_COMPLIANCE_POLICY_PATH");

        let tmp_path = env::temp_dir().join(format!(
            "refractive_swan-compliance-policy-{}.json",
            uuid::Uuid::new_v4()
        ));
        fs::write(
            &tmp_path,
            r#"{
                "mode": "partner",
                "allowed_actions": ["map", "export"],
                "allowed_tiers": {
                    "map": ["open"],
                    "export": ["open"]
                }
            }"#,
        )
        .unwrap();

        set_env_var("refractive_swan_COMPLIANCE_MODE", "internal");
        set_env_var(
            "refractive_swan_COMPLIANCE_POLICY_PATH",
            tmp_path.to_string_lossy().as_ref(),
        );

        let policy = load_policy_from_env().unwrap();
        assert_eq!(policy.mode, ComplianceMode::Partner);
        assert!(policy.is_action_allowed(ComplianceAction::Map));
        assert!(!policy.is_action_allowed(ComplianceAction::Ingest));
        assert!(policy.is_allowed(ComplianceAction::Map, LicenseTier::Open));
        assert!(!policy.is_allowed(ComplianceAction::Map, LicenseTier::Licensed));

        clear_env_var("refractive_swan_COMPLIANCE_MODE");
        clear_env_var("refractive_swan_COMPLIANCE_POLICY_PATH");
        let _ = fs::remove_file(&tmp_path);
    }

    #[test]
    fn json_policy_overrides_cover_all_actions() {
        let _lock = env_guard().lock().unwrap();
        let tmp_path = env::temp_dir().join(format!(
            "refractive_swan-compliance-policy-{}.json",
            uuid::Uuid::new_v4()
        ));
        fs::write(
            &tmp_path,
            r#"{
                "mode": "partner",
                "allowed_actions": ["ingest", "map", "export"],
                "allowed_tiers": {
                    "ingest": ["licensed", "open", "internal_only"],
                    "map": ["open"],
                    "export": ["open"]
                }
            }"#,
        )
        .unwrap();

        set_env_var(
            "refractive_swan_COMPLIANCE_POLICY_PATH",
            tmp_path.to_string_lossy().as_ref(),
        );

        let policy = load_policy_from_env().expect("json overrides apply");
        assert_eq!(policy.mode, ComplianceMode::Partner);
        assert!(policy.is_action_allowed(ComplianceAction::Ingest));
        assert!(policy.is_allowed(ComplianceAction::Ingest, LicenseTier::InternalOnly));
        assert!(policy.is_allowed(ComplianceAction::Map, LicenseTier::Open));
        assert!(!policy.is_allowed(ComplianceAction::Map, LicenseTier::Licensed));
        assert!(policy.is_allowed(ComplianceAction::Export, LicenseTier::Open));

        clear_env_var("refractive_swan_COMPLIANCE_POLICY_PATH");
        let _ = fs::remove_file(&tmp_path);
    }

    #[test]
    fn yaml_policy_overrides_apply() {
        let _lock = env_guard().lock().unwrap();
        let tmp_path = env::temp_dir().join(format!(
            "refractive_swan-compliance-policy-{}.yaml",
            uuid::Uuid::new_v4()
        ));
        fs::write(
            &tmp_path,
            r#"
mode: open_source
allowed_actions: ["map"]
allowed_tiers:
  map: ["open"]
"#,
        )
        .unwrap();

        let base = Policy::default_for_mode(ComplianceMode::Internal);
        let overrides = PolicyOverrides::from_path(&tmp_path).expect("yaml overrides");
        let policy = base.apply_overrides(overrides);
        assert_eq!(policy.mode, ComplianceMode::OpenSource);
        assert!(policy.is_action_allowed(ComplianceAction::Map));
        assert!(!policy.is_action_allowed(ComplianceAction::Export));
        assert!(policy.is_allowed(ComplianceAction::Map, LicenseTier::Open));
        assert!(!policy.is_allowed(ComplianceAction::Map, LicenseTier::Licensed));

        let _ = fs::remove_file(&tmp_path);
    }

    #[test]
    fn assert_export_allowed_blocks_forbidden_tier() {
        let policy = Policy::default_for_mode(ComplianceMode::OpenSource);
        let err = assert_export_allowed(&[LicenseTier::Licensed], &policy)
            .expect_err("licensed export should be blocked in OSS mode");
        matches!(err, ComplianceError::ExportNotAllowed { .. });
    }

    #[test]
    fn export_gating_enforces_license_tiers_per_mode() {
        let internal = Policy::default_for_mode(ComplianceMode::Internal);
        assert!(
            assert_export_allowed(
                &[
                    LicenseTier::Licensed,
                    LicenseTier::Open,
                    LicenseTier::InternalOnly
                ],
                &internal
            )
            .is_ok()
        );

        let partner = Policy::default_for_mode(ComplianceMode::Partner);
        assert!(assert_export_allowed(&[LicenseTier::Licensed], &partner).is_ok());
        assert!(assert_export_allowed(&[LicenseTier::Open], &partner).is_ok());
        let partner_err = assert_export_allowed(&[LicenseTier::InternalOnly], &partner)
            .expect_err("partner mode should block internal-only exports");
        matches!(partner_err, ComplianceError::ExportNotAllowed { .. });

        let oss = Policy::default_for_mode(ComplianceMode::OpenSource);
        assert!(assert_export_allowed(&[LicenseTier::Open], &oss).is_ok());
        let oss_err = assert_export_allowed(&[LicenseTier::Licensed], &oss)
            .expect_err("open source mode blocks licensed exports");
        matches!(oss_err, ComplianceError::ExportNotAllowed { .. });
    }

    #[test]
    fn load_policy_reports_parse_error_for_malformed_override() {
        let _lock = env_guard().lock().unwrap();
        clear_env_var("refractive_swan_COMPLIANCE_MODE");
        clear_env_var("refractive_swan_COMPLIANCE_POLICY_PATH");
        let tmp_path = env::temp_dir().join(format!(
            "refractive_swan-compliance-policy-invalid-{}.yaml",
            uuid::Uuid::new_v4()
        ));
        fs::write(&tmp_path, "mode: [not valid").unwrap();
        set_env_var(
            "refractive_swan_COMPLIANCE_POLICY_PATH",
            tmp_path.to_string_lossy().as_ref(),
        );
        let err = load_policy_from_env().expect_err("invalid overrides should fail");
        matches!(err, ComplianceError::PolicyPathParse { .. });
        clear_env_var("refractive_swan_COMPLIANCE_POLICY_PATH");
        let _ = fs::remove_file(&tmp_path);
    }

    #[test]
    fn load_policy_reports_missing_override_file() {
        let _lock = env_guard().lock().unwrap();
        let dir = env::temp_dir().join(format!(
            "refractive_swan-compliance-missing-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).unwrap();
        unsafe { env::set_var("refractive_swan_WORKSPACE_ROOT", &dir) };
        set_env_var("refractive_swan_COMPLIANCE_POLICY_PATH", "not_there.json");

        let err = load_policy_from_env().expect_err("missing policy file should error");
        match err {
            ComplianceError::PolicyPathIo { path, .. } => {
                assert!(path.ends_with("not_there.json"));
                assert!(path.starts_with(&dir));
            }
            other => panic!("expected PolicyPathIo, got {other:?}"),
        }

        clear_env_var("refractive_swan_COMPLIANCE_POLICY_PATH");
        unsafe { env::remove_var("refractive_swan_WORKSPACE_ROOT") };
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_policy_uses_workspace_root_for_relative_path() {
        let _lock = env_guard().lock().unwrap();
        let dir = env::temp_dir().join(format!(
            "refractive_swan-compliance-root-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).unwrap();
        let policy_path = dir.join("relative_policy.json");
        fs::write(
            &policy_path,
            r#"{ "mode": "partner", "allowed_actions": ["map"], "allowed_tiers": { "map": ["licensed", "open"] } }"#,
        )
        .unwrap();

        set_env_var(
            "refractive_swan_WORKSPACE_ROOT",
            dir.to_string_lossy().as_ref(),
        );
        set_env_var(
            "refractive_swan_COMPLIANCE_POLICY_PATH",
            "relative_policy.json",
        );

        let policy = load_policy_from_env().expect("relative policy loads");
        assert_eq!(policy.mode, ComplianceMode::Partner);

        clear_env_var("refractive_swan_WORKSPACE_ROOT");
        clear_env_var("refractive_swan_COMPLIANCE_POLICY_PATH");
        let _ = fs::remove_file(policy_path);
        let _ = std::fs::remove_dir(dir);
    }
}
