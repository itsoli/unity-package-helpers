use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use package_lib::{Package, Result, Version, read_json, write_json};

#[derive(Serialize, Deserialize, Debug)]
struct ScopedRegistry {
    name: String,
    url: String,
    scopes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Manifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<BTreeMap<String, String>>,
    #[serde(rename = "enableLockFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_lock_file: Option<bool>,
    #[serde(rename = "resolutionStrategy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "scopedRegistries")]
    scoped_registries: Option<Vec<ScopedRegistry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    testables: Option<Vec<String>>,
}

/// Reads manifest at `manifest_path` and updates existing manifest dependencies to versions
/// specified by `packages`.
pub fn update_manifest_packages(manifest_path: &str, packages: &[Package]) -> Result<()> {
    let mut manifest: Manifest = read_json(manifest_path)?;

    let Some(dependencies) = &mut manifest.dependencies else {
        return Ok(()); // Perhaps return an error here?
    };

    let package_map = packages
        .iter()
        .map(|package| (package.name.as_str(), &package.version))
        .collect::<HashMap<&str, &Version>>();

    let mut update_names = Vec::<(&str, &Version)>::new();
    for k in dependencies.keys() {
        if let Some((name, version)) = package_map.get_key_value(k.as_str()) {
            update_names.push((name, version));
        }
    }
    for (name, version) in update_names.into_iter() {
        dependencies.insert(name.to_string(), version.to_string());
    }

    write_json(manifest_path, &manifest)?;

    Ok(())
}
