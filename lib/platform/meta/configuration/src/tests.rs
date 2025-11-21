use crate::{
    EnvLoadError, EnvValueError, bool_var, config_paths, load_env, port_var, string_var,
    workspace_root,
};
use std::{
    env, fs,
    sync::{Mutex, OnceLock},
};
use tempfile::tempdir;

static ENV_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

fn env_guard() -> &'static Mutex<()> {
    ENV_GUARD.get_or_init(|| Mutex::new(()))
}

fn clear_env(keys: &[&str]) {
    for key in keys {
        unsafe { env::remove_var(key) };
    }
}

#[test]
fn workspace_root_discovers_parent_with_cargo_lock() {
    let _lock = env_guard().lock().unwrap();
    let original = env::current_dir().unwrap();
    let temp = tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("Cargo.lock"), b"").unwrap();
    let nested = root.join("nested/deeper");
    fs::create_dir_all(&nested).unwrap();
    env::set_current_dir(&nested).unwrap();
    clear_env(&["refractive_swan_WORKSPACE_ROOT"]);

    let resolved = workspace_root().expect("workspace root resolved");
    assert_eq!(resolved, root);

    env::set_current_dir(original).unwrap();
}

#[test]
fn workspace_root_errors_without_cargo_lock() {
    let _lock = env_guard().lock().unwrap();
    let original = env::current_dir().unwrap();
    let temp = tempdir().unwrap();
    env::set_current_dir(temp.path()).unwrap();
    clear_env(&["refractive_swan_WORKSPACE_ROOT"]);

    let err = workspace_root().expect_err("should fail without Cargo.lock");
    matches!(err, EnvLoadError::WorkspaceRootNotFound);

    env::set_current_dir(original).unwrap();
}

#[test]
fn workspace_root_respects_env_override() {
    let _lock = env_guard().lock().unwrap();
    let temp = tempdir().unwrap();
    let marker = temp.path();
    fs::write(marker.join("Cargo.lock"), b"").unwrap();
    unsafe {
        env::set_var("refractive_swan_WORKSPACE_ROOT", marker);
    }
    let resolved = workspace_root().expect("workspace root resolves via env");
    assert_eq!(resolved, marker);
    clear_env(&["refractive_swan_WORKSPACE_ROOT"]);
}

#[test]
fn config_paths_use_workspace_root_when_set() {
    let _lock = env_guard().lock().unwrap();
    let temp = tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("Cargo.lock"), b"").unwrap();
    fs::create_dir_all(root.join("data/environment")).unwrap();
    unsafe {
        env::set_var("refractive_swan_WORKSPACE_ROOT", root);
    }
    let paths = config_paths().expect("config paths");
    assert!(
        paths
            .env_dirs
            .iter()
            .any(|dir| dir.ends_with("data/environment"))
    );
    clear_env(&["refractive_swan_WORKSPACE_ROOT"]);
}

#[test]
fn config_paths_honors_env_dir_override() {
    let _lock = env_guard().lock().unwrap();
    let temp = tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("Cargo.lock"), b"").unwrap();
    fs::create_dir_all(root.join("custom")).unwrap();
    unsafe {
        env::set_var("refractive_swan_WORKSPACE_ROOT", root);
        env::set_var("refractive_swan_ENV_DIR", "custom");
    }

    let paths = config_paths().expect("paths");
    assert_eq!(paths.env_dirs.len(), 1);
    assert!(paths.env_dirs[0].ends_with("custom"));

    clear_env(&["refractive_swan_WORKSPACE_ROOT", "refractive_swan_ENV_DIR"]);
}

#[test]
fn resolves_refractive_swan_env_file_overrides_search_dirs() {
    let _lock = env_guard().lock().unwrap();
    let temp = tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("Cargo.lock"), b"").unwrap();
    fs::create_dir_all(root.join("data/environment")).unwrap();
    fs::write(root.join("custom.env"), b"SAMPLE=1").unwrap();
    unsafe {
        env::set_var("refractive_swan_WORKSPACE_ROOT", root);
        env::set_var("refractive_swan_ENV_FILE", "custom.env");
    }

    let outcome = load_env("platform.vector_store").expect("load env");
    assert_eq!(outcome.files.len(), 1);
    assert!(outcome.files[0].ends_with("custom.env"));

    clear_env(&["refractive_swan_WORKSPACE_ROOT", "refractive_swan_ENV_FILE"]);
}

#[test]
fn strict_mode_enforces_missing_files() {
    let _lock = env_guard().lock().unwrap();
    let temp = tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("Cargo.lock"), b"").unwrap();
    fs::create_dir_all(root.join("data/environment")).unwrap();
    unsafe {
        env::set_var("refractive_swan_WORKSPACE_ROOT", root);
        env::set_var("refractive_swan_ENV_STRICT", "1");
    }

    let err = load_env("platform.vector_store").expect_err("strict mode should fail");
    matches!(err, EnvLoadError::FileMissing { .. });

    unsafe { env::remove_var("refractive_swan_ENV_STRICT") };
    let outcome = load_env("platform.vector_store").expect("non-strict load succeeds");
    assert!(outcome.files.is_empty());

    clear_env(&["refractive_swan_WORKSPACE_ROOT"]);
}

#[test]
fn bool_var_parses_expected_values() {
    let _lock = env_guard().lock().unwrap();
    unsafe { env::set_var("BOOL_TEST", "off") };
    assert_eq!(bool_var("BOOL_TEST").unwrap(), Some(false));
    unsafe { env::set_var("BOOL_TEST", "1") };
    assert_eq!(bool_var("BOOL_TEST").unwrap(), Some(true));
    unsafe { env::remove_var("BOOL_TEST") };
    assert!(bool_var("BOOL_TEST").unwrap().is_none());
}

#[test]
fn port_var_rejects_zero() {
    let _lock = env_guard().lock().unwrap();
    unsafe { env::set_var("PORT_TEST", "0") };
    let err = port_var("PORT_TEST").expect_err("port 0 invalid");
    matches!(err, EnvValueError::InvalidPort { .. });
    unsafe { env::set_var("PORT_TEST", "8080") };
    assert_eq!(port_var("PORT_TEST").unwrap(), Some(8080));
    unsafe { env::remove_var("PORT_TEST") };
}

#[test]
fn string_var_trims_and_ignores_empty_values() {
    let _lock = env_guard().lock().unwrap();
    unsafe { env::set_var("STRING_TEST", "  value  ") };
    assert_eq!(
        string_var("STRING_TEST").unwrap(),
        Some("value".to_string())
    );
    unsafe { env::set_var("STRING_TEST", "   ") };
    assert!(string_var("STRING_TEST").unwrap().is_none());
    unsafe { env::remove_var("STRING_TEST") };
}
