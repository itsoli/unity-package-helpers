use std::cmp::{Ordering, self};
use std::{error, collections::HashMap};
use std::path::{Path, PathBuf};
// use std::fs::File;
// use std::io::{stdin, stdout};
use std::{result, fs};

use clap::{self, Parser};
use git2::{Repository, StatusOptions};
use walkdir::{DirEntry, WalkDir};

type Result<T> = result::Result<T, Box<dyn error::Error>>;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub(crate) struct Options {
    #[clap(short, long, default_value = ".")]
    pub repository_path: String,
    #[clap(short, long, default_value = "Packages")]
    pub packages_path: String
}

struct Package {
    pub name: String,
    pub path: PathBuf,

    // pub new: Vec<String>,
    // pub modified: Vec<String>,
    // pub added: Vec<String>,
    // pub deleted: Vec<String>,
    // pub renamed: Vec<String>,
    pub changes: Vec<(String, git2::Status)>
}

impl Package {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            changes: Vec::new(),
        }
    }

    pub fn is_changed(&self) -> bool {
        !self.changes.is_empty()
    }

    pub fn is_deleted(&self) -> bool {
        // self.deleted.iter().any(|name| name.ends_with("package.json"))
        self.changes.iter().any(|(name, status)|
            (status.is_wt_deleted() || status.is_index_deleted())
            && name.ends_with("package.json")
        )
    }
}

fn get_package_name(path: Option<&str>, packages: &HashMap::<String, Package>) -> Option<String> {
    // path.and_then(|p| Path::new(p).to_owned());//.and(|x|)
    let p = Path::new(path.unwrap());
    for x in p {
        let k = x.to_string_lossy().into_owned();
        if packages.contains_key(&k) {
            return Some(k);
        }
    }
    None
}

// fn get_package<'a>(path: Option<&str>, packages: &'a HashMap::<String, Package>) -> Option<&'a mut Package> {
//     // path.and_then(|p| Path::new(p).to_owned());//.and(|x|)
//     let p = Path::new(path.unwrap());
//     for x in p {
//         let k = x.to_string_lossy().into_owned();
//         let y = packages.get_mut(&k);
//         if y.is_some() {
//             return y;
//         }
//         // if packages.contains_key(&k) {
//         //     // return Some(k);
//         //     return
//         // }
//     }
//     None
// }

// fn status_value(status: &git2::Status) -> u32 {
//     if status.is_index_new() || status.is_wt_new() {
//         0
//     } else if status.is_index_modified() || status.is_wt_modified() {
//         1
//     } else if status.is_index_deleted() || status.is_wt_deleted() {
//         2
//     }
// }

// fn cmp(a: &git2::Status, b: &git2::Status) -> Ordering {
//     if a == b {
//         return Ordering::Equal;
//     }
//     let diff = status_value(a) - status_value(b);
//     if diff < 0 { Ordering::Less } else { Ordering::Greater }
// }

fn main() -> Result<()> {
    let options = Options::parse();

    let mut packages = HashMap::<String, Package>::new();
    let repository_path_abs = fs::canonicalize(&options.repository_path)?;
    let packages_path = Path::new(options.repository_path.as_str())
        .join(options.packages_path.as_str());

    for entry in WalkDir::new(packages_path).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "package.json" {
            continue;
        }
        let package_path = {
            let mut path = entry.path().to_path_buf();
            path.pop();
            path.strip_prefix(&repository_path_abs).unwrap().to_owned()
        };
        let package_name = package_path.iter().next_back().unwrap().to_owned().into_string().unwrap();
        packages.insert(package_name.clone(), Package::new(package_name, package_path));
    }

    let repo = Repository::open(options.repository_path)?;

    let mut status_options = StatusOptions::new();
    status_options
        .include_ignored(false)
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .exclude_submodules(true);

    let statuses = repo.statuses(Some(&mut status_options))?;

    for entry in statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
    {
        // let package = get_package(entry.path(), &packages);
        // println!("{:?} {:?} {:?}", package.and_then(|x| Some(&x.name)), entry.path(), entry.status());
        let package_name = get_package_name(entry.path(), &packages);
        if package_name.is_some() {
            let pn = package_name.unwrap();
            let xx = packages.get_mut(&pn).unwrap();
            xx.changes.push((pn, entry.status()));
            // xx.changes.sort_by(|(x, y), (x2, y2)| {
            //     let status = cmp(y, y2);
            //     if status == Ordering::Equal { x.cmp(x2) } else { status }
            // });
            xx.changes.sort_unstable_by(|(x, _), (x2, _)| x.cmp(x2));
        }
        // println!("{:?} {:?} {:?}", package_name, entry.path(), entry.status());
    }

    // let changed_packages: Vec::<Package> = packages.iter().filter(|&(_, package)| package.is_changed()).map(|(_, package)| *package).collect();
    let changed_packages: Vec::<String> = packages.iter().filter(|&(_, package)| package.is_changed()).map(|(name, _)| name.clone()).collect();

    Ok(())
}
