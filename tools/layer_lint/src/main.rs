use anyhow::{Context, Result, anyhow};
use camino::Utf8Path;
use guppy::{MetadataCommand, graph::PackageGraph};
use std::collections::{HashMap, HashSet};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    App,
    Domain,
    Platform,
}

fn main() -> Result<()> {
    let graph = MetadataCommand::new()
        .build_graph()
        .context("failed to build cargo metadata graph")?;

    let mut errors = Vec::new();
    errors.extend(check_dependency_layers(&graph));
    errors.extend(check_domain_env_usage()?);

    if errors.is_empty() {
        Ok(())
    } else {
        for error in &errors {
            eprintln!("[layer-lint] {error}");
        }
        Err(anyhow!(
            "layer lint failed with {} violation(s)",
            errors.len()
        ))
    }
}

fn check_dependency_layers(graph: &PackageGraph) -> Vec<String> {
    let mut layer_map: HashMap<&str, Layer> = HashMap::new();
    for package in graph.packages() {
        if let Some(layer) = classify_layer(&package) {
            layer_map.insert(package.name(), layer);
        }
    }

    let mut violations = Vec::new();
    for package in graph.packages() {
        let Some(source_layer) = layer_map.get(package.name()) else {
            continue;
        };

        for link in package.direct_links() {
            let target = link.to();
            let Some(target_layer) = layer_map.get(target.name()) else {
                continue;
            };
            if violates_layers(package.name(), target.name(), *source_layer, *target_layer) {
                violations.push(format!(
                    "crate '{}' ({:?}) must not depend on '{}' ({:?})",
                    package.name(),
                    source_layer,
                    target.name(),
                    target_layer
                ));
            }
        }
    }

    violations
}

fn classify_layer(package: &guppy::graph::PackageMetadata<'_>) -> Option<Layer> {
    if should_skip_package(package.name()) {
        return None;
    }
    if let Some(layer) = layer_override(package.name()) {
        return Some(layer);
    }
    classify_by_path(package.manifest_path())
}

fn classify_by_path(path: &Utf8Path) -> Option<Layer> {
    let display = path.as_str();
    if display.contains("/lib/app/") {
        Some(Layer::App)
    } else if display.contains("/lib/domain/") {
        Some(Layer::Domain)
    } else if display.contains("/lib/platform/") {
        Some(Layer::Platform)
    } else {
        None
    }
}

fn layer_override(name: &str) -> Option<Layer> {
    match name {
        "dfps_vector_store" => Some(Layer::Platform),
        "dfps_datamart" => Some(Layer::App),
        _ => None,
    }
}

fn should_skip_package(name: &str) -> bool {
    matches!(name, "dfps_test_suite")
}

const EDGE_ALLOWLIST: [(&str, &str); 2] = [
    ("dfps_compliance", "dfps_terminology"),
    ("dfps_observability", "dfps_core"),
];

fn violates_layers(source_name: &str, target_name: &str, source: Layer, target: Layer) -> bool {
    if EDGE_ALLOWLIST
        .iter()
        .any(|(src, dst)| src == &source_name && dst == &target_name)
    {
        return false;
    }
    match (source, target) {
        (Layer::Domain, Layer::App) => true,
        (Layer::Platform, Layer::App) => true,
        (Layer::Platform, Layer::Domain) => true,
        _ => false,
    }
}

fn check_domain_env_usage() -> Result<Vec<String>> {
    let root = Utf8Path::new("lib/domain");
    let mut offenders = Vec::new();
    if !root.exists() {
        return Ok(offenders);
    }

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "rs") {
            continue;
        }
        if path.to_string_lossy().contains("/src/bin/") {
            continue;
        }
        let contents = std::fs::read_to_string(path)?;
        if contains_env_usage(&contents) {
            offenders.push(path.display().to_string());
        }
    }

    if offenders.is_empty() {
        Ok(Vec::new())
    } else {
        let mut dedup = HashSet::new();
        let mut msgs = Vec::new();
        for path in offenders {
            if dedup.insert(path.clone()) {
                msgs.push(format!(
                    "domain code must not read std::env (found in {path})"
                ));
            }
        }
        Ok(msgs)
    }
}

fn contains_env_usage(contents: &str) -> bool {
    contents.contains("std::env::") || contents.contains("std :: env ::")
}
