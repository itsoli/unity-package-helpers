use clap::{self, Parser};

mod manifest_util;

use manifest_util::update_manifest_packages;
use package_lib::{find_packages, Package, Result};

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub(crate) struct Options {
    #[clap(short, long, default_value = ".")]
    pub packages_path: String,
    #[clap(short, long, default_value = "manifest.json")]
    pub manifest_path: String,
    #[clap(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let options = Options::parse();

    let mut packages = find_packages(options.packages_path.as_str()).collect::<Vec<Package>>();

    if options.verbose {
        println!("{} package(s) found", packages.len());

        packages.sort_unstable();
        for package in packages.iter() {
            println!("{} {}", package.name, package.version);
        }
    }

    update_manifest_packages(&options.manifest_path, &packages)?;

    Ok(())
}
