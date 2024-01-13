use std::collections::HashMap;
use std::error;
use std::fs::File;
use std::io::BufReader;
use std::iter::Flatten;
use std::path::Path;
use std::result;

use jwalk::{DirEntryIter, WalkDir};
use serde::Deserialize;
use unicode_bom::Bom;

pub mod semver;

pub use semver::Version;

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

pub fn open_reader<P: AsRef<Path>>(path: P) -> Result<BufReader<File>> {
    let mut file = File::open(&path)?;
    let bom_len = Bom::from(&mut file).len();
    file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    if bom_len > 0 {
        reader.seek_relative(bom_len as i64)?;
    }
    Ok(reader)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
}

pub struct PackageIterator {
    it: Flatten<DirEntryIter<((), ())>>,
}

impl PackageIterator {
    pub fn new<P: AsRef<Path>>(packages_path: P) -> PackageIterator {
        let walk_dir = WalkDir::new(packages_path).process_read_dir(
            |_depth, _path, _read_dir_state, children| {
                let is_package = children.iter().any(|dir_entry_result| {
                    dir_entry_result
                        .as_ref()
                        .map(|dir_entry| {
                            dir_entry.file_type.is_file() && dir_entry.file_name == "package.json"
                        })
                        .unwrap_or(false)
                });
                if is_package {
                    children.retain(|dir_entry_result| {
                        dir_entry_result
                            .as_ref()
                            .map(|dir_entry| {
                                dir_entry.file_type.is_file()
                                    && dir_entry.file_name == "package.json"
                            })
                            .unwrap_or(false)
                    });
                } else {
                    children.retain(|dir_entry_result| {
                        dir_entry_result
                            .as_ref()
                            .map(|dir_entry| dir_entry.file_type.is_dir())
                            .unwrap_or(false)
                    });
                }
            },
        );

        PackageIterator {
            it: walk_dir.into_iter().flatten(),
        }
    }
}

impl Iterator for PackageIterator {
    type Item = Package;

    fn next(&mut self) -> Option<Self::Item> {
        for entry in self.it.by_ref() {
            if entry.file_type.is_file() {
                let reader = open_reader(entry.path()).unwrap();
                let result: serde_json::Result<Package> = serde_json::from_reader(reader);
                if let Ok(package) = result {
                    return Some(package);
                }
            }
        }
        None
    }
}

pub fn find_packages<P: AsRef<Path>>(packages_path: P) -> PackageIterator {
    PackageIterator::new(packages_path)
}

pub fn get_packages<P: AsRef<Path>>(packages_path: P) -> HashMap<String, Version> {
    find_packages(packages_path)
        .map(|package| (package.name, package.version))
        .collect::<HashMap<String, Version>>()
}

pub fn get_sorted_package_list(packages: &HashMap<String, Version>) -> Vec<Package> {
    let mut package_list = packages
        .iter()
        .map(|(k, v)| Package {
            name: k.clone(),
            version: v.clone(),
        })
        .collect::<Vec<Package>>();
    package_list.sort_unstable();
    package_list
}
