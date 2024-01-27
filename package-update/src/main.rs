use std::fs;
use std::io::{self, Write};

use git2::Repository;
use owo_colors::OwoColorize;
use serde_json::{json, Value};

use package_lib::{read_json, write_json, NormalizeLineEndings, Result, Trim, Version};

mod command;
mod diff;
mod options;
mod package;

use crate::command::*;
use crate::diff::*;
use crate::options::*;
use crate::package::*;

fn read_line_from_stdin() -> String {
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input).map_err(|_| input.clear());
    input.trim();
    input
}

fn write_package_manifest(package: &Package, new_version: Version) -> Result<()> {
    let package_manifest_path = package
        .path_abs
        .join(package_lib::PACKAGE_MANIFEST_FILENAME);

    let mut package_json: Value = read_json(package_manifest_path.as_path())?;
    if !package_json.is_object() {
        return Err("package.json is not an object".into());
    }

    package_json["version"] = json!(new_version.to_string());

    write_json(package_manifest_path, &package_json)
}

fn write_package_changelog(
    package: &Package,
    new_version: Version,
    change_lines: &[String],
    options: &Options,
) -> Result<()> {
    let changelog_path = package.path_abs.join(options.changelog_filename.as_str());

    let mut text = fs::read_to_string(changelog_path.as_path())?;
    text.trim();
    text.normalize_line_endings();
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

fn update_package(package: &Package, new_version: Version, options: &Options) -> Result<()> {
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
    changed_packages: &[Package],
    repo: &Repository,
    options: &Options,
) -> Result<()> {
    let command_prompt = format!("[{}] > ", get_command_key_text());

    let mut index = 0;
    while index < changed_packages.len() {
        let package = &changed_packages[index];

        println!("package name: {}", package.name.yellow());
        println!("package version: {}", package.version.yellow());
        println!("changed files:");
        for (path, status) in package.changes.iter() {
            println!("  [{}] {}", status_to_str(status), path.purple());
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

fn main() -> Result<()> {
    let options = Options::parse();

    let repository_path = options.repository_path.as_path();
    let repo = Repository::open(repository_path).map_err(|err| err.message().to_owned())?;
    if repo.is_bare() {
        return Err("repository is bare".into());
    }
    if repo.is_shallow() {
        return Err("repository is shallow".into());
    }

    let packages_path = options.packages_path.as_path();
    let changed_packages = get_changed_packages(&repo, repository_path, packages_path)?;

    if options.verbose {
        println!("{} package(s) changed", changed_packages.len());
        for package in changed_packages.iter() {
            println!("{} {}", package.name.bold(), package.version);
            for (path, status) in package.changes.iter() {
                println!("  [{}] {}", status_to_str(status), path);
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
