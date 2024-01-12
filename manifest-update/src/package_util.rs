use std::collections::HashMap;
use std::iter::{Filter, Flatten};
use std::path::{Path, PathBuf};

use jwalk::WalkDir;
// use rayon::iter::{Filter, Flatten};
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

// type IteratorType = Filter<Flatten<jwalk::DirEntryIter<((), ())>>, dyn FnMut(&jwalk::DirEntry<((), ())>) -> bool>;
// type TheType = Filter<Flatten<jwalk::DirEntryIter<((), ())>>, impl FnMut(&jwalk::DirEntry<((), ())>) -> bool>;

// struct FindPackages<F: FnMut(&jwalk::DirEntry<((), ())>) -> bool> {
// struct FindPackages<F: FnMut(&jwalk::DirEntry<((), ())>) -> bool + Sized> {
// struct FindPackages {
// struct FindPackages<T> {
//     // it: <walkdir::WalkDir as std::iter::IntoIterator>::IntoIter,
//     // it: u32,
//     // walk_dir: jwalk::WalkDirGeneric<((), ())>,
//     // it: Filter<>,
//     // it: Filter<Flatten<jwalk::DirEntryIter<((), ())>>, dyn Fn(&jwalk::DirEntry<((), ())>) -> bool>,
//     // it: Filter<I, P>,
//     // it: Filter<Flatten<jwalk::DirEntryIter<((), ())>>, F>,
//     // it: IteratorType,
//     // it: Filter<Flatten<jwalk::DirEntryIter<((), ())>>, dyn FnMut(&jwalk::DirEntry<((), ())>) -> bool + Sized>,
//     it: T,
// }

// struct FindPackages<I: Iterator<Item = jwalk::DirEntry<((), ())>>> {
//     it: I,
// }

// impl<I: Iterator, P> Iterator for Filter<I, P>
// where
//     P: FnMut(&I::Item) -> bool,

// struct FindPackages<P: FnMut(&jwalk::DirEntryIter<((), ())>) -> bool> {
//     it: Filter<Flatten<jwalk::DirEntryIter<((), ())>>, P>,
// }

struct FindPackages {
    it: Flatten<jwalk::DirEntryIter<((), ())>>,
}

// impl<F: FnMut(&jwalk::DirEntry<((), ())>) -> bool> FindPackages<F> {
// impl<F: FnMut(&jwalk::DirEntry<((), ())>) -> bool + Sized> FindPackages<F> {
impl FindPackages {
// impl<T> FindPackages<T> {
// impl<I: Iterator<Item = jwalk::DirEntry<((), ())>>> FindPackages<I> {
// impl<I: Iterator, P: FnMut(&I::Item) -> bool> FindPackages<I, P> {
// impl<P: FnMut(&jwalk::DirEntryIter<((), ())>) -> bool> FindPackages<P> {
    // pub fn new<PP: AsRef<Path>>(packages_path: PP) -> FindPackages<Filter<Flatten<jwalk::DirEntryIter<((), ())>>, impl FnMut(&jwalk::DirEntry<((), ())>) -> bool>> {
    // pub fn new<PP: AsRef<Path>>(packages_path: PP) -> FindPackages<impl FnMut(&'a jwalk::DirEntryIter<((), ())>) -> bool + '_> {
    pub fn new<PP: AsRef<Path>>(packages_path: PP) -> FindPackages {
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

        use jwalk::*;

        // Filter<Flatten<jwalk::DirEntryIter<((), ())>>, FnMut(&Self::Item) -> bool>
        // let it = walk_dir
        //     .into_iter()
        //     .flatten()
        //     .filter(|entry| entry.file_type.is_file());
        let it = walk_dir.into_iter().flatten();

        // while let Some(entry) = it.next() {
        //     let reader = open_reader(entry.path()).unwrap();
        //     let package: Package = serde_json::from_reader(reader).unwrap();
        //     // (package.name, package.version)
        // }

            // it.map(|entry| {
            //     let reader = open_reader(entry.path()).unwrap();
            //     let package: Package = serde_json::from_reader(reader).unwrap();
            //     (package.name, package.version)
            // })
            // .collect::<HashMap::<String, Version>>();

        FindPackages {
            // it: WalkDir::new(packages_path).into_iter(),
            // walk_dir,
            it,
        }
    }
}

impl Iterator for FindPackages {
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

// impl Iterator for FindPackages<Filter<Flatten<jwalk::DirEntryIter<((), ())>>, impl FnMut(&jwalk::DirEntry<((), ())>) -> bool>> {
// impl<T: Iterator<Item = PathBuf>> Iterator for FindPackages<T> {
// impl<I: Iterator<Item = jwalk::DirEntry<((), ())>>> Iterator for FindPackages<I> {
// impl<I: Iterator, P: FnMut(&I::Item) -> bool> Iterator for FindPackages<I, P> {
// impl<T: Filter<Flatten<jwalk::DirEntryIter<((), ())>>, impl FnMut(&jwalk::DirEntry<((), ())>) -> bool>> Iterator for FindPackages<T> {
// impl<P: FnMut(&jwalk::DirEntryIter<((), ())>) -> bool> Iterator for FindPackages<P> {
//     // type Item = PathBuf;
//     type Item = (String, Version);

//     fn next(&mut self) -> Option<Self::Item> {
//         // if let Some(entry) = self.it.next() {
//         //     // let reader = open_reader(entry.path()).unwrap();
//         //     let reader = open_reader(entry.as_path()).unwrap();
//         //     let package: Package = serde_json::from_reader(reader).unwrap();
//         //     return Some((package.name, package.version));
//         // }
//         // None

//         match self.it.next() {
//             Some(entry) => {
//                 // let reader = open_reader(entry.path()).unwrap();
//                 let reader = open_reader(entry.path()).unwrap();
//                 let package: Package = serde_json::from_reader(reader).unwrap();
//                 Some((package.name, package.version))
//             },
//             None => None,
//         }

//         // loop {
//         //     let entry = match self.it.next() {
//         //         None|Some(Err(_)) => return None,
//         //         Some(Ok(entry)) => entry,
//         //     };

//         //     if !entry.file_type().is_file() {
//         //         continue;
//         //     }
//         //     if entry.file_name() != "package.json" {
//         //         continue;
//         //     }

//         //     self.it.skip_current_dir();

//         //     return Some(entry.into_path());
//         // }
//     }
// }

// impl<T: Iterator<Item = PathBuf>> IntoIterator for FindPackages<T> {
//     type Item = (String, Version);
//     type IntoIter = FindPackages<T>;

//     fn into_iter(self) -> FindPackages<T> {
//         self
//     }
// }

fn find_packages_x(packages_path: &str) -> FindPackages {
    // let x = FindPackages::new(packages_path);
    // for i in x {
    // }
    FindPackages::new(packages_path)
}


// fn find_packages<P: AsRef<Path>>(packages_path: P) -> impl Iterator<Item = (String, Version)> {
//     let fp = FindPackages::new(packages_path);
//     fp.n
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

fn impl_jwalk_it(packages_path: &str) -> HashMap::<String, Version> {
    find_packages_x(packages_path)
        .map(|package| (package.name, package.version))
        .collect::<HashMap::<String, Version>>()
}

pub fn find_packages(packages_path: &str) -> Result<HashMap::<String, Version>> {
    // impl_sync(packages_path) // 0.060
    // Ok(impl_rayon(packages_path)) // 0.058
    // Ok(impl_jwalk(packages_path)) // 0.040
    Ok(impl_jwalk_it(packages_path))
}

pub fn get_sorted_package_list(packages: &HashMap::<String, Version>) -> Vec<Package> {
    let mut package_list = packages
        .iter()
        .map(|(k, v)| Package { name: k.clone(), version: v.clone() })
        .collect::<Vec<Package>>();
    package_list.sort_unstable();
    package_list
}
