
use std::path::PathBuf;

pub use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub(crate) struct Options {
    /// Path to the git repository
    #[clap(short, long, default_value = ".")]
    pub repository_path: PathBuf,
    /// Path to the packages directory
    #[clap(short, long, default_value = "Packages")]
    pub packages_path: PathBuf,
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
    pub verbose: bool,
}
