use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str;

use clap::Parser;
use git2::Repository;
use owo_colors::OwoColorize;

use package_lib::{
    find_packages, read_json, trim_string, write_json, Package, Result, Version,
    PACKAGE_MANIFEST_FILENAME,
};
use serde_json::{json, Value};

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub(crate) struct Options {
    /// Path to the git repository
    #[clap(short, long, default_value = ".")]
    pub repository_path: String,
    /// Path to the packages directory
    #[clap(short, long, default_value = "Packages")]
    pub packages_path: String,
    /// Name of the changelog file
    #[clap(long, default_value = "release_notes.md")]
    pub changelog_filename: String,
    /// Changelog entry template
    #[clap(long, default_value = "\n## Version {version}")]
    pub changelog_version_entry_template: String,
    /// Changelog change item template
    #[clap(long, default_value = " - {message}")]
    pub changelog_change_item_template: String,
    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum CommandKind {
    UpdateMajor,
    UpdateMinor,
    UpdatePatch,
    Skip,
    Diff,
    Quit,
    Help,
}

struct CommandMeta {
    pub kind: CommandKind,
    pub key: char,
    pub help: &'static str,
}

const COMMAND_LIST: [CommandMeta; 7] = [
    CommandMeta {
        kind: CommandKind::UpdateMajor,
        key: '1',
        help: "update package major version",
    },
    CommandMeta {
        kind: CommandKind::UpdateMinor,
        key: '2',
        help: "update package minor version",
    },
    CommandMeta {
        kind: CommandKind::UpdatePatch,
        key: '3',
        help: "update package patch version",
    },
    CommandMeta {
        kind: CommandKind::Skip,
        key: 's',
        help: "skip current package",
    },
    CommandMeta {
        kind: CommandKind::Diff,
        key: 'd',
        help: "show diff for current package",
    },
    CommandMeta {
        kind: CommandKind::Quit,
        key: 'q',
        help: "quit; do not update package or any of the remaining ones",
    },
    CommandMeta {
        kind: CommandKind::Help,
        key: '?',
        help: "print help",
    },
];

fn get_command_meta(kind: CommandKind) -> &'static CommandMeta {
    COMMAND_LIST
        .iter()
        .find(|&command| command.kind == kind)
        .unwrap()
}

fn get_command_meta_from_input(input: &str) -> Option<&'static CommandMeta> {
    if input.len() != 1 {
        return None;
    }
    let key = input.chars().next().unwrap();
    COMMAND_LIST.iter().find(|&command| command.key == key)
}

fn get_command_kind_from_input(input: &str) -> Option<CommandKind> {
    if input.len() != 1 {
        return None;
    }
    let key = input.chars().next().unwrap();
    COMMAND_LIST
        .iter()
        .find(|&command| command.key == key)
        .map(|command| command.kind)
}

fn get_command_help() -> String {
    COMMAND_LIST
        .iter()
        .map(|command| format!("{} - {}", command.key, command.help))
        .collect::<Vec<String>>()
        .join("\n")
}

fn read_line_from_stdin() -> String {
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input).map_err(|_| input.clear());
    trim_string(&mut input);
    input
}

fn write_package_manifest(package: &PackageInfo, new_version: Version) -> Result<()> {
    let package_manifest_path = package.path.join(PACKAGE_MANIFEST_FILENAME);
    let mut package_json: Value = read_json(package_manifest_path.as_path())?;
    // TODO: error handling
    package_json["version"] = json!(new_version.to_string());

    write_json(package_manifest_path, &package_json)
}

fn write_package_changelog(
    package: &PackageInfo,
    new_version: Version,
    change_lines: &[String],
    options: &Options,
) -> Result<()> {
    let changelog_path = package.path.join(options.changelog_filename.as_str());

    let mut text = fs::read_to_string(changelog_path.as_path())?;
    trim_string(&mut text);
    if !text.is_empty() {
        text.push('\n');
    }

    let version_entry = options
        .changelog_version_entry_template
        .replace("{version}", new_version.to_string().as_str());
    text.push_str(version_entry.as_str());
    text.push('\n');

    for change_line in change_lines.iter() {
        let change_item = options
            .changelog_change_item_template
            .replace("{message}", change_line);
        text.push_str(change_item.as_str());
        text.push('\n');
    }

    fs::write(changelog_path.as_path(), text)?;

    Ok(())
}

fn update_package(package: &PackageInfo, new_version: Version, options: &Options) -> Result<()> {
    let mut change_lines = Vec::<String>::new();

    loop {
        print!("{}", "enter change message: ".blue());
        io::stdout().flush().unwrap();

        let line = read_line_from_stdin();
        if line.is_empty() {
            if !change_lines.is_empty() {
                break;
            }
            println!("{}", "change message required".red());
        } else {
            change_lines.push(line);
        }
    }

    write_package_changelog(package, new_version, &change_lines, options)?;
    write_package_manifest(package, new_version)?;

    Ok(())
}

fn process_packages(
    changed_packages: &[&PackageInfo],
    repo: &Repository,
    options: &Options,
) -> Result<()> {
    let command_prompt = format!(
        "[{}] > ",
        COMMAND_LIST
            .iter()
            .map(|command| command.key.to_string())
            .collect::<Vec<String>>()
            .join(","),
    );

    let mut index = 0;
    while index < changed_packages.len() {
        let package = changed_packages[index];

        println!("package name: {}", package.name.yellow());
        println!("package version: {}", package.version.yellow());
        println!("changed files:");
        for (path, _status) in package.changes.iter() {
            println!("  * {}", path.purple());
        }

        let command_prompt_text = format!(
            "({}/{}) {}",
            index + 1,
            changed_packages.len(),
            command_prompt
        );
        print!("{}", command_prompt_text.blue());
        io::stdout().flush().unwrap();

        match get_command_kind_from_input(read_line_from_stdin().as_str()) {
            Some(CommandKind::UpdateMajor) => {
                let new_version = package.version.increment_major();
                println!("update major {} -> {}", package.version, new_version);
                update_package(package, new_version, options)?;
            }
            Some(CommandKind::UpdateMinor) => {
                let new_version = package.version.increment_minor();
                println!("update minor {} -> {}", package.version, new_version);
                update_package(package, new_version, options)?;
            }
            Some(CommandKind::UpdatePatch) => {
                let new_version = package.version.increment_patch();
                println!("update patch {} -> {}", package.version, new_version);
                update_package(package, new_version, options)?;
            }
            Some(CommandKind::Skip) => {}
            Some(CommandKind::Diff) => {
                print_diff(repo, package)?;
                continue;
            }
            Some(CommandKind::Quit) => break,
            Some(CommandKind::Help) | None => {
                println!("{}", get_command_help().red());
                continue;
            }
        }

        index += 1;
    }

    Ok(())
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

fn set_statuses(repo: &Repository, packages: &mut HashMap<String, PackageInfo>) -> Result<()> {
    let mut status_options = git2::StatusOptions::new();
    status_options
        .show(git2::StatusShow::Index)
        .include_ignored(false)
        // .include_untracked(true)
        // .recurse_untracked_dirs(true)
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

// fn print_stats(diff: &Diff) -> Result<()> {
//     let stats = diff.stats()?;
//     let mut format = git2::DiffStatsFormat::NONE;
//     format |= git2::DiffStatsFormat::FULL;
//     // format |= git2::DiffStatsFormat::SHORT;
//     // format |= git2::DiffStatsFormat::NUMBER;
//     format |= git2::DiffStatsFormat::INCLUDE_SUMMARY;
//     let buf = stats.to_buf(format, 80)?;
//     print!("{}", str::from_utf8(&buf).unwrap());
//     Ok(())
// }

// fn print_diff_line(
//     _delta: git2::DiffDelta,
//     _hunk: Option<git2::DiffHunk>,
//     line: git2::DiffLine,
// ) -> bool {
//     // if args.color() {
//     //     print!("{}", RESET);
//     //     if let Some(color) = line_color(&line) {
//     //         print!("{}", color);
//     //     }
//     // }
//     match line.origin() {
//         '+' | '-' | ' ' => print!("{}", line.origin()),
//         _ => {}
//     }
//     print!("{}", str::from_utf8(line.content()).unwrap());
//     true
// }

// fn get_repo_path<P>(repo: &Repository, path: P) -> Result<&Path>
// where
//     P: AsRef<Path>,
// {
//     let repo_workdir_path = repo.workdir().unwrap();
//     let package_relative_path = Path::strip_prefix(path.as_ref(), repo_workdir_path)?;
//     Ok(package_relative_path)
// }

fn get_repo_path<'a>(repo: &Repository, path: &'a Path) -> Result<&'a Path> {
    let repo_workdir_path = repo.workdir().unwrap();
    let package_relative_path = Path::strip_prefix(path, repo_workdir_path)?;
    Ok(package_relative_path)
}

// pub type FileCb<'a> = dyn FnMut(DiffDelta<'_>, f32) -> bool + 'a;
// pub type BinaryCb<'a> = dyn FnMut(DiffDelta<'_>, DiffBinary<'_>) -> bool + 'a;
// pub type HunkCb<'a> = dyn FnMut(DiffDelta<'_>, DiffHunk<'_>) -> bool + 'a;
// pub type LineCb<'a> = dyn FnMut(DiffDelta<'_>, Option<DiffHunk<'_>>, DiffLine<'_>) -> bool + 'a;

// fn diff_delta_file(delta: git2::DiffDelta) -> String {
//     if delta.new_file().path() == delta.old_file().path() {
//         return delta
//             .new_file()
//             .path()
//             .unwrap()
//             .to_str()
//             .unwrap()
//             .to_owned();
//     }

//     format!(
//         "{} -> {}",
//         delta.new_file().path().map_or("", |x| x.to_str().unwrap()),
//         delta.old_file().path().map_or("", |x| x.to_str().unwrap())
//     )
// }

fn diff_file_cb(delta: git2::DiffDelta, _progress: f32) -> bool {
    // println!(
    //     "|diff_file_cb| file:{} progress:{}",
    //     diff_delta_file(delta),
    //     progress
    // );
    let old_file = delta.old_file().path().map_or("", |x| x.to_str().unwrap());
    let new_file = delta.new_file().path().map_or("", |x| x.to_str().unwrap());
    let text = format!("--- a/{}\n+++ b/{}\n", old_file, new_file);
    print!("{}", text.bold());
    true
}

fn diff_binary_cb(_delta: git2::DiffDelta, _binary: git2::DiffBinary) -> bool {
    // println!(
    //     "|diff_binary_cb| new_file:{} old_file:{} contains_data:{}",
    //     delta.new_file().path().map_or("", |x| x.to_str().unwrap()),
    //     delta.old_file().path().map_or("", |x| x.to_str().unwrap()),
    //     binary.contains_data()
    // );
    println!("Binary files differ");
    true
}

fn split_header(header: &str) -> (&str, &str) {
    let mut lines_end = 0;
    let mut count = 0;
    for (index, ch) in header.chars().enumerate() {
        if ch != '@' {
            continue;
        }
        count += 1;
        if count == 4 {
            lines_end = index + 1;
            break;
        }
    }
    (header[0..lines_end].trim(), header[lines_end..].trim())
}

fn git_diff_hunk_cb(_delta: git2::DiffDelta, hunk: git2::DiffHunk) -> bool {
    // println!(
    //     "|git_diff_hunk_cb| file:{}\n{} {} {} {} {}",
    //     diff_delta_file(delta),
    //     str::from_utf8(hunk.header()).unwrap().purple(),
    //     hunk.old_start(),
    //     hunk.old_lines(),
    //     hunk.new_start(),
    //     hunk.new_lines(),
    // );
    let header = str::from_utf8(hunk.header()).unwrap();
    let (lines, context) = split_header(header);
    println!("{} {}", lines.bright_cyan(), context);
    true
}

// fn color_line<C: owo_colors::Color>(origin: char, content: &str) -> owo_colors::FgColorDisplay<'_, C, &str> {
//     if origin == '+' {
//         content.green()
//     } else if origin == '-' {
//         content.red()
//     } else {
//         content.
//     }
// }

fn diff_line_cb(
    _delta: git2::DiffDelta,
    _hunk: Option<git2::DiffHunk>,
    line: git2::DiffLine,
) -> bool {
    let origin = line.origin();
    let content = str::from_utf8(line.content()).unwrap();
    if origin == '+' {
        print!("{} {}", origin.green(), content.green());
    } else if origin == '-' {
        print!("{} {}", origin.red(), content.red());
    } else {
        print!("{} {}", origin, content);
    }
    true
}

fn print_diff(repo: &Repository, package: &PackageInfo) -> Result<()> {
    // let repo_workdir_path = repo.workdir().unwrap();
    // let package_relative_path = Path::strip_prefix(package.path.as_path(), repo_workdir_path)?;
    let package_relative_path = get_repo_path(repo, package.path.as_path())?;
    // println!("DIFF [{}]", package_relative_path.display());

    let mut diff_options = git2::DiffOptions::new();
    diff_options
        .ignore_whitespace(true)
        // diff_options.pathspec(package.path.as_path());
        // diff_options.pathspec("stratkit/Packages/com.stratkit.ui-unit-production");
        .pathspec(package_relative_path);

    // let head = repo.head()?.resolve()?.peel_to_tree()?;
    let head = repo.head()?.peel_to_tree()?;
    // let head = repo.revparse_single("HEAD")?.peel(ObjectType::Tree)?.as_tree();
    let diff = repo.diff_tree_to_index(Some(&head), None, Some(&mut diff_options))?;
    // let head = repo.revparse_single("HEAD")?.peel(ObjectType::Tree)?;
    // let diff = repo.diff_tree_to_index(head.as_tree(), None, Some(&mut diff_options))?;

    // print_stats(&diff)?;

    // diff.print(git2::DiffFormat::Patch, |d, h, l| print_diff_line(d, h, l))?;
    // diff.print(git2::DiffFormat::Raw, print_diff_line)?;
    // diff.print(git2::DiffFormat::PatchId, print_diff_line)?;

    diff.foreach(
        &mut diff_file_cb,
        Some(&mut diff_binary_cb),
        Some(&mut git_diff_hunk_cb),
        Some(&mut diff_line_cb),
    )?;

    Ok(())
}

fn main() -> Result<()> {
    let options = Options::parse();

    let packages_path =
        Path::new(options.repository_path.as_str()).join(options.packages_path.as_str());
    let mut packages = find_packages(packages_path.as_path())
        .map(|package| (package.name.clone(), PackageInfo::new(package)))
        .collect::<HashMap<String, PackageInfo>>();

    let repo = Repository::open(options.repository_path.as_str())?;

    set_statuses(&repo, &mut packages)?;

    let mut changed_packages = packages
        .iter()
        .filter(|&(_, package)| !package.is_deleted() && package.is_changed())
        .map(|(_, package)| package)
        .collect::<Vec<&PackageInfo>>();
    changed_packages.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let mut pkgs = packages.values().collect::<Vec<&PackageInfo>>();
    pkgs.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    if options.verbose {
        println!("{} package(s) changed", changed_packages.len());
        for package in changed_packages.iter() {
            println!(
                "* {} {} ({})",
                package.name,
                package.version,
                package.path.display()
            );
            for (path, status) in package.changes.iter() {
                println!("  - {} {:?}", path, status);
            }
        }
    }

    if changed_packages.is_empty() {
        println!("no packages changed");
    } else {
        process_packages(&changed_packages, &repo, &options)?;
    }

    Ok(())
}
