use clap::{self, Parser};

mod manifest_util;

use manifest_util::update_manifest_packages;
use package_lib::{get_packages, Result};

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

    let packages = get_packages(&options.packages_path);
    println!("{} packages found", packages.len());
    for package in package_lib::get_sorted_package_list(&packages).iter() {
        println!("{} {}", package.name, package.version);
    }

    update_manifest_packages(&options.manifest_path, &packages)?;

    Ok(())
}
