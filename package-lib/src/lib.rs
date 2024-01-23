use std::error;
use std::path::{Path, PathBuf};
use std::result;

use jwalk::{DirEntryIter, WalkDir};
use serde::Deserialize;

mod io;
mod semver;

pub use io::*;
pub use semver::Version;

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

/// Filename of the package manifest file "package.json".
pub static PACKAGE_MANIFEST_FILENAME: &str = "package.json";

#[derive(Deserialize, Debug)]
struct PackageManifest {
    pub name: String,
    pub version: Version,
}

/// Unity package metadata.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub path: PathBuf,
}

/// Iterator over UPM packages.
pub struct PackageIterator {
    it: DirEntryIter<((), ())>,
}

impl PackageIterator {
    pub fn new<P: AsRef<Path>>(packages_path: P) -> PackageIterator {
        let walk_dir = WalkDir::new(packages_path.as_ref()).process_read_dir(
            |_depth, _path, _read_dir_state, children| {
                // Find index of package.json file in children.
                let mut package_index = None;
                for (index, dir_entry_result) in children.iter().enumerate() {
                    if dir_entry_result
                        .as_ref()
                        .map(|dir_entry| {
                            dir_entry.file_type.is_file()
                                && dir_entry.file_name == PACKAGE_MANIFEST_FILENAME
                        })
                        .unwrap_or(false)
                    {
                        package_index = Some(index);
                        break;
                    }
                }

                if let Some(package_index) = package_index {
                    let mut index = 0;
                    children.retain(|_| {
                        let retain = index == package_index;
                        index += 1;
                        retain
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
            it: walk_dir.into_iter(),
        }
    }
}

impl Iterator for PackageIterator {
    type Item = Package;

    fn next(&mut self) -> Option<Self::Item> {
        for entry in self.it.by_ref().flatten() {
            if !entry.file_type.is_file() {
                continue;
            }
            let Ok(package): Result<PackageManifest> = read_json(entry.path()) else {
                continue;
            };

            let mut package_path = entry.path().to_path_buf();
            package_path.pop();

            return Some(Package {
                name: package.name,
                version: package.version,
                path: package_path,
            });
        }
        None
    }
}

/// Recursively scans the provided `packages_path` for UPM packages and returns iterator over found
/// packages.
pub fn find_packages<P: AsRef<Path>>(packages_path: P) -> PackageIterator {
    PackageIterator::new(packages_path)
}
