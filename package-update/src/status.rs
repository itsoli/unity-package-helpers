use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str;

use git2::Repository;

use package_lib::{find_packages, Result, Version, PACKAGE_MANIFEST_FILENAME};

/// Information about a package along with git changes.
pub(crate) struct Package {
    pub name: String,
    pub version: Version,
    pub path: PathBuf,
    pub changes: Vec<(String, git2::Status)>,
}

impl Package {
    pub fn new(package: package_lib::Package) -> Self {
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

    /// Returns the path of the package relative to the repository workdir.
    pub fn path_in_repo(&self, repo: &Repository) -> PathBuf {
        let repo_workdir_path = get_repo_workdir_path(repo);
        self.path
            .strip_prefix(repo_workdir_path)
            .map(|p| p.to_path_buf())
            .unwrap_or(self.path.clone())
    }
}

fn get_repo_workdir_path(repo: &Repository) -> PathBuf {
    let repo_workdir_path = repo.workdir().unwrap_or(repo.path());
    fs::canonicalize(repo_workdir_path).unwrap_or(repo_workdir_path.to_path_buf())
}

/// Returns a mutable reference to the package in the `packages` hash map at `path_str` or `None`
/// if no package exists.
fn get_package_mut<'a>(
    path_str: &str,
    packages: &'a mut HashMap<String, Package>,
) -> Option<&'a mut Package> {
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

fn set_statuses(repo: &Repository, packages: &mut HashMap<String, Package>) -> Result<()> {
    let mut status_options = git2::StatusOptions::new();
    status_options
        .show(git2::StatusShow::Index)
        .include_ignored(false)
        .include_untracked(false)
        .recurse_untracked_dirs(false)
        .exclude_submodules(true);

    let statuses = repo.statuses(Some(&mut status_options))?;

    for entry in statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
    {
        let Some(path) = entry.path() else {
            continue;
        };
        let Some(package) = get_package_mut(path, packages) else {
            continue;
        };
        package.changes.push((path.to_owned(), entry.status()));
        package.changes.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
    }

    Ok(())
}

pub(crate) fn get_changed_packages(
    repo: &Repository,
    repository_path: &Path,
    packages_path: &Path,
) -> Result<Vec<Package>> {
    let packages_path = repository_path.join(packages_path);
    let mut packages = find_packages(packages_path.as_path())
        .map(|package| (package.name.clone(), Package::new(package)))
        .collect::<HashMap<String, Package>>();

    set_statuses(repo, &mut packages)?;

    let mut changed_packages = packages
        .into_values()
        .filter(|package| !package.is_deleted() && package.is_changed())
        .collect::<Vec<Package>>();
    changed_packages.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    Ok(changed_packages)
}

pub(crate) fn status_to_str(status: &git2::Status) -> &str {
    if status.is_index_new() || status.is_wt_new() {
        "new"
    } else if status.is_index_modified() || status.is_wt_modified() {
        "modified"
    } else if status.is_index_deleted() || status.is_wt_deleted() {
        "deleted"
    } else if status.is_index_renamed() || status.is_wt_renamed() {
        "renamed"
    } else if status.is_index_typechange() || status.is_wt_typechange() {
        "typechange"
    } else {
        "unknown"
    }
}
