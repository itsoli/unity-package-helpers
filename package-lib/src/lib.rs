use std::error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
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

pub static PACKAGE_MANIFEST_FILENAME: &str = "package.json";

#[derive(Deserialize, Debug)]
struct PackageManifest {
    pub name: String,
    pub version: Version,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub path: PathBuf,
}

pub struct PackageIterator {
    it: DirEntryIter<((), ())>,
}

impl PackageIterator {
    pub fn new<P: AsRef<Path>>(packages_path: P) -> PackageIterator {
        let walk_dir = WalkDir::new(packages_path.as_ref()).process_read_dir(
            |_depth, _path, _read_dir_state, children| {
                let is_package = children.iter().any(|dir_entry_result| {
                    dir_entry_result
                        .as_ref()
                        .map(|dir_entry| {
                            dir_entry.file_type.is_file()
                                && dir_entry.file_name == PACKAGE_MANIFEST_FILENAME
                        })
                        .unwrap_or(false)
                });
                if is_package {
                    children.retain(|dir_entry_result| {
                        dir_entry_result
                            .as_ref()
                            .map(|dir_entry| {
                                dir_entry.file_type.is_file()
                                    && dir_entry.file_name == PACKAGE_MANIFEST_FILENAME
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
            it: walk_dir.into_iter(),
        }
    }
}

impl Iterator for PackageIterator {
    type Item = Package;

    fn next(&mut self) -> Option<Self::Item> {
        for entry in self.it.by_ref().flatten() {
            if entry.file_type.is_file() {
                if let Ok(reader) = open_reader(entry.path()) {
                    if let Ok(package) = serde_json::from_reader::<_, PackageManifest>(reader) {
                        let mut package_path = entry.path().to_path_buf();
                        package_path.pop();

                        return Some(Package {
                            name: package.name,
                            version: package.version,
                            path: package_path,
                        });
                    }
                }
            }
        }
        None
    }
}

pub fn find_packages<P: AsRef<Path>>(packages_path: P) -> PackageIterator {
    PackageIterator::new(packages_path)
}
