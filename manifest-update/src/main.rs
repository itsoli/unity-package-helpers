use clap::{self, Parser};

mod manifest_util;
mod package_util;
mod shared;

use manifest_util::update_manifest_packages;
use package_util::find_packages;
use shared::Result;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub(crate) struct Options {
    #[clap(short, long, default_value = ".")]
    pub packages_path: String,
    #[clap(short, long, default_value = "manifest.json")]
    pub manifest_path: String,
}

fn main() -> Result<()> {
    let options = Options::parse();

    let packages = find_packages(&options.packages_path)?;
    println!("{} packages found", packages.len());
    for package in package_util::get_sorted_package_list(&packages).iter() {
        println!("{} {}", package.name, package.version);
    }

    update_manifest_packages(&options.manifest_path, &packages)?;

    Ok(())
}
