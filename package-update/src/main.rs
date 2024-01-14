use std::collections::HashMap;
use std::path::{Path, PathBuf};

use clap::{self, Parser};
use git2::{Repository, StatusOptions};

use package_lib::{find_packages, Package, Result, Version, PACKAGE_MANIFEST_FILENAME};

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub(crate) struct Options {
    /// Path to the git repository
    #[clap(short, long, default_value = ".")]
    pub repository_path: String,
    /// Path to the packages directory
    #[clap(short, long, default_value = "Packages")]
    pub packages_path: String,
    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
}

/// Information about a package along with git changes.
struct PackageInfo {
    pub name: String,
    pub version: Version,
    pub path: PathBuf,
    pub changes: Vec<(String, git2::Status)>,
}

impl PackageInfo {
    pub fn new(package: Package) -> Self {
        Self {
            name: package.name,
            version: package.version,
            path: package.path,
            changes: Vec::new(),
        }
    }

    pub fn is_changed(&self) -> bool {
        !self.changes.is_empty()
    }

    pub fn is_deleted(&self) -> bool {
        self.changes.iter().any(|(name, status)| {
            (status.is_wt_deleted() || status.is_index_deleted())
                && name.ends_with(PACKAGE_MANIFEST_FILENAME)
        })
    }
}

/// Returns a mutable reference to the package in the `packages` hash map at `path_str` or `None`
/// if no package exists.
fn get_package_mut<'a>(
    path_str: &str,
    packages: &'a mut HashMap<String, PackageInfo>,
) -> Option<&'a mut PackageInfo> {
    let path = Path::new(path_str);
    for path_component in path {
        let path_component_str = path_component.to_str().unwrap();
        // FIXME: This is likely a limitation of the borrow checker. Would be nice to avoid the second lookup here.
        // if let Some(package) = packages.get_mut(path_component_str) {
        //     return Some(package);
        // }
        if packages.contains_key(path_component_str) {
            return Some(packages.get_mut(path_component_str).unwrap());
        }
    }
    None
}

fn main() -> Result<()> {
    let options = Options::parse();

    let packages_path =
        Path::new(options.repository_path.as_str()).join(options.packages_path.as_str());
    let mut packages = find_packages(packages_path.as_path())
        .map(|package| (package.name.clone(), PackageInfo::new(package)))
        .collect::<HashMap<String, PackageInfo>>();

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
        let Some(path) = entry.path() else {
            continue;
        };
        let Some(package) = get_package_mut(path, &mut packages) else {
            continue;
        };
        package.changes.push((path.to_owned(), entry.status()));
        package.changes.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
    }

    let changed_packages = packages
        .iter()
        .filter(|&(_, package)| !package.is_deleted() && package.is_changed())
        .map(|(_, package)| package)
        .collect::<Vec<&PackageInfo>>();

    if options.verbose {
        println!("{} package(s) changed", changed_packages.len());
        for package in changed_packages.iter() {
            println!(
                "{} {} ({})",
                package.name,
                package.version,
                package.path.display()
            )
        }
    }

    Ok(())
}
