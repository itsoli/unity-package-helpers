use std::collections::{BTreeMap, HashMap};
use std::fs;

use serde::{Deserialize, Serialize};

use package_lib::{open_reader, Result, Version};

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

pub fn update_manifest_packages(
    manifest_path: &str,
    packages: &HashMap<String, Version>,
) -> Result<()> {
    let reader = open_reader(manifest_path)?;
    let mut manifest: Manifest = serde_json::from_reader(reader)?;

    if let Some(dependencies) = &mut manifest.dependencies {
        let mut update_names = Vec::<(&str, &Version)>::new();
        for k in dependencies.keys() {
            if let Some((name, version)) = packages.get_key_value(k) {
                update_names.push((name, version));
            }
        }
        for (name, version) in update_names.into_iter() {
            dependencies.insert(name.to_string(), version.to_string());
        }
    }

    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    // println!("{}", manifest_json);
    fs::write(manifest_path, manifest_json).expect("Unable to write manifest");

    Ok(())
}
