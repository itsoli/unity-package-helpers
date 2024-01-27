use std::str;

use git2::Repository;
use owo_colors::OwoColorize;

use package_lib::Result;

use crate::Package;

fn diff_file_cb(delta: git2::DiffDelta, _progress: f32) -> bool {
    let old_file = delta.old_file().path().map_or("", |x| x.to_str().unwrap());
    let new_file = delta.new_file().path().map_or("", |x| x.to_str().unwrap());
    let text = format!("--- a/{}\n+++ b/{}\n", old_file, new_file);
    print!("{}", text.bold());
    true
}

fn diff_binary_cb(_delta: git2::DiffDelta, _binary: git2::DiffBinary) -> bool {
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
    let header = str::from_utf8(hunk.header()).unwrap();
    let (lines, context) = split_header(header);
    println!("{} {}", lines.bright_cyan(), context);
    true
}

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

pub(crate) fn print_diff(repo: &Repository, package: &Package) -> Result<()> {
    let mut diff_options = git2::DiffOptions::new();
    diff_options
        .pathspec(package.path.as_path())
        .ignore_whitespace(true);

    let head = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head), None, Some(&mut diff_options))?;

    diff.foreach(
        &mut diff_file_cb,
        Some(&mut diff_binary_cb),
        Some(&mut git_diff_hunk_cb),
        Some(&mut diff_line_cb),
    )?;

    Ok(())
}
