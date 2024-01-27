use std::fs;
use std::path::{Path, PathBuf};
use std::str;

use git2::Repository;

use package_lib::{find_packages, Result, Version, PACKAGE_MANIFEST_FILENAME};

/// Information about a package along with git changes.
pub(crate) struct Package {
    /// Name of the package.
    pub name: String,
    /// Version of the package.
    pub version: Version,
    /// Path of the package relative to the repository workdir.
    pub path: PathBuf,
    /// Absolute path of the package.
    pub path_abs: PathBuf,
    /// Git changes of the package.
    pub changes: Vec<(String, git2::Status)>,
}

impl Package {
    pub fn new(package: package_lib::Package, repo: &Repository) -> Result<Self> {
        let path = get_path_in_repo(package.path.as_path(), repo);
        let changes = get_changes(repo, path.as_path())?;

        Ok(Self {
            name: package.name,
            version: package.version,
            path,
            path_abs: package.path,
            changes,
        })
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

fn get_repo_workdir_path(repo: &Repository) -> PathBuf {
    let repo_workdir_path = repo.workdir().unwrap_or(repo.path());
    fs::canonicalize(repo_workdir_path).unwrap_or(repo_workdir_path.to_path_buf())
}

fn get_path_in_repo(path: &Path, repo: &Repository) -> PathBuf {
    let repo_workdir_path = get_repo_workdir_path(repo);
    path.strip_prefix(repo_workdir_path)
        .map(|p| p.to_path_buf())
        .unwrap_or(path.to_path_buf())
}

fn get_changes(repo: &Repository, path: &Path) -> Result<Vec<(String, git2::Status)>> {
    let mut status_options = git2::StatusOptions::new();
    status_options
        .pathspec(path)
        .show(git2::StatusShow::Index)
        .include_ignored(false)
        .include_untracked(false)
        .recurse_untracked_dirs(false)
        .exclude_submodules(true);

    let statuses = repo.statuses(Some(&mut status_options))?;

    let mut changes = statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
        .map(|e| {
            let path = e.path().unwrap();
            let status = e.status();
            (path.to_owned(), status)
        })
        .collect::<Vec<(String, git2::Status)>>();

    changes.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    Ok(changes)
}

pub(crate) fn get_changed_packages(
    repo: &Repository,
    repository_path: &Path,
    packages_path: &Path,
) -> Result<Vec<Package>> {
    let packages_path = repository_path.join(packages_path);

    let mut changed_packages = find_packages(packages_path.as_path())
        .map(|package| Package::new(package, repo))
        .filter_map(|package| package.ok())
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
