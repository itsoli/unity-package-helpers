use std::collections::HashMap;
use std::path::PathBuf;

// use jwalk::WalkDir;
// use rayon::prelude::*;
use serde::Deserialize;
// use walkdir::WalkDir;

use package_lib::semver::Version;
use crate::shared::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
}

// struct FindPackages {
//     it: <walkdir::WalkDir as std::iter::IntoIterator>::IntoIter,
//     // it: u32,
// }

// impl FindPackages {
//     pub fn new<P: AsRef<Path>>(packages_path: P) -> Self {
//         Self {
//             it: WalkDir::new(packages_path).into_iter(),
//         }
//     }
// }

// impl Iterator for FindPackages {
//     type Item = PathBuf;

//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             let entry = match self.it.next() {
//                 None|Some(Err(_)) => return None,
//                 Some(Ok(entry)) => entry,
//             };

//             if !entry.file_type().is_file() {
//                 continue;
//             }
//             if entry.file_name() != "package.json" {
//                 continue;
//             }

//             self.it.skip_current_dir();

//             return Some(entry.into_path());
//         }
//     }
// }

// fn find_packages<P: AsRef<Path>>(packages_path: P) -> impl Iterator<Item=&Path> {
    // let mut it = WalkDir::new(options.packages_path).into_iter();
    // loop {
    //     let entry = match it.next() {
    //         None => break,
    //         Some(Err(err)) => panic!("ERROR: {}", err),
    //         Some(Ok(entry)) => entry,
    //     };

    //     if !entry.file_type().is_file() {
    //         continue;
    //     }
    //     if entry.file_name() != "package.json" {
    //         continue;
    //     }

    //     yield entry.path();

    //     it.skip_current_dir();
    // }
// }

fn impl_sync(packages_path: &str) -> Result<HashMap::<String, Version>> {
    use walkdir::WalkDir;

    // let mut packages = Vec::<Package>::new();
    let mut packages = HashMap::<String, Version>::new();

    // for entry in WalkDir::new(packages_path).into_iter().filter_map(|e| e.ok()) {
    let mut it = WalkDir::new(packages_path).into_iter();
    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(err)) => panic!("ERROR: {}", err),
            Some(Ok(entry)) => entry,
        };

        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "package.json" {
            continue;
        }

        let reader = open_reader(entry.path())?;
        let package: Package = serde_json::from_reader(reader)?;
        // packages.push(package);
        packages.insert(package.name, package.version);

        it.skip_current_dir();
    }

    Ok(packages)
}

fn impl_rayon(packages_path: &str) -> HashMap::<String, Version> {
    use walkdir::WalkDir;

    let mut package_files = Vec::<PathBuf>::new();

    let mut it = WalkDir::new(packages_path).into_iter();
    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(err)) => panic!("ERROR: {}", err),
            Some(Ok(entry)) => entry,
        };

        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "package.json" {
            continue;
        }

        package_files.push(entry.path().to_owned());

        it.skip_current_dir();
    }

    // let x = FindPackages::new(options.packages_path).into_iter();

    package_files
    // let packages: HashMap::<String, Version> = FindPackages::new(options.packages_path)
        .iter()
        .map(|path| {
            let reader = open_reader(path).unwrap();
            let package: Package = serde_json::from_reader(reader).unwrap();
            (package.name, package.version)
        })
        .collect::<HashMap::<String, Version>>()
}

fn impl_jwalk(packages_path: &str) -> HashMap::<String, Version> {
    use jwalk::WalkDir;

    // let x = WalkDir::new(options.packages_path).into_iter().process;

    let walk_dir = WalkDir::new(packages_path).process_read_dir(|depth, path, read_dir_state, children| {
        let is_package = children.iter().any(|dir_entry_result| {
            dir_entry_result
                .as_ref()
                .map(|dir_entry| dir_entry.file_type.is_file() && dir_entry.file_name == "package.json")
                .unwrap_or(false)
        });
        if is_package {
            children.retain(|dir_entry_result| {
                dir_entry_result
                    .as_ref()
                    .map(|dir_entry| dir_entry.file_type.is_file() && dir_entry.file_name == "package.json")
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

        // let package = children.iter().find_map(|dir_entry_result| {
        //     dir_entry_result.as_ref().map_or(None, |dir_entry| {
        //         if dir_entry.file_type.is_file() && dir_entry.file_name == "package.json" {
        //             Some(dir_entry)
        //         } else {
        //             None
        //         }
        //     })
        // });

        // children.retain(|dir_entry_result| {
        //     dir_entry_result.as_ref().map(|dir_entry| {
        //         dir_entry.file_type.is_dir() || dir_entry.file_name == "package.json"
        //         // dir_entry.file_name
        //         //     .to_str()
        //         //     .map(|s| s == "package.json")
        //         //     .unwrap_or(false)
        //     }).unwrap_or(false)
        // });
    });

    // for dir_entry in walk_dir.into_iter().flatten() {
    //     if dir_entry.file_type.is_file() {
    //         // println!("xx {}", dir_entry.path().display());
    //     }
    //     // if entry?.file_type.is_file() {
    //     //     println!("xx {}", entry?.path().display());
    //     // }
    // }

    walk_dir
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type.is_file())
        .map(|entry| {
            let reader = open_reader(entry.path()).unwrap();
            let package: Package = serde_json::from_reader(reader).unwrap();
            (package.name, package.version)
        })
        .collect::<HashMap::<String, Version>>()
}

pub fn find_packages(packages_path: &str) -> Result<HashMap::<String, Version>> {
    // impl_sync(packages_path) // 0.060
    // Ok(impl_rayon(packages_path)) // 0.058
    Ok(impl_jwalk(packages_path)) // 0.040
}

pub fn get_sorted_package_list(packages: &HashMap::<String, Version>) -> Vec<Package> {
    let mut package_list = packages
        .iter()
        .map(|(k, v)| Package { name: k.clone(), version: v.clone() })
        .collect::<Vec<Package>>();
    package_list.sort_unstable();
    package_list
}
