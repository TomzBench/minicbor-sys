use cddl_gen::{Language, Options};
use clap::{arg, value_parser, ArgAction, Command};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::result::Result;
use std::str::FromStr;
use tracing::{info, Level};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("atx-codegen")
        .version("0.0.1")
        .author("Thomas Chiantia <thomas@altronix.com")
        .about("Altronix CDDL codegen tool")
        .args(vec![
            arg!(-D --debug [LEVEL] "Debug level [TRACE ERROR WARN DEBUG INFO NONE]"),
            arg!(-n --name <NAME> "Name of project"),
            arg!(-b --bindings "Generate as \"C\" bindings").action(ArgAction::SetTrue),
            arg!(-p --prefix <PREFIX> "Prefix generated types with PREFIX"),
            arg!(-c --cddl <CDDL> "Path to CDDL descriptor").value_parser(value_parser!(PathBuf)),
            arg!(-V --version <VERSION> "Generate crate with version VERSION"),
            arg!(<path> ... "Path to create project in")
                .trailing_var_arg(true)
                .value_parser(value_parser!(PathBuf)),
        ])
        .get_matches();

    // Parse the log level
    let level = matches
        .get_one::<String>("debug")
        .map(String::as_ref)
        .unwrap_or("info");
    tracing_subscriber::fmt()
        .with_max_level(Level::from_str(level)?)
        .init();

    let path: &PathBuf = matches
        .get_one("path")
        .expect("Must provide path for project!");
    let name = matches
        .get_one::<String>("name")
        .map(String::as_ref)
        .expect("Must provide name for project!");
    // We create one big concattenated CDDL file from multiple cddl paths
    let cddl = matches
        .get_many::<PathBuf>("cddl")
        .expect("Must provide path for cddl!")
        .map(fs::read)
        .collect::<Result<Vec<Vec<u8>>, _>>()?
        .iter()
        .map(|cddl| std::str::from_utf8(&cddl))
        .collect::<Result<String, _>>()?;

    let prefix = matches.get_one::<String>("prefix").map(String::from);
    let version = matches.get_one::<String>("version").map(String::from);
    let bindings = matches.get_flag("bindings");
    let options = Options {
        prefix,
        version,
        language: if bindings {
            Language::C
        } else {
            Language::Rust
        },
    };
    info!("Creating CDDL project [{}]", path.display());
    let cargo = path.clone().join("Cargo.toml");
    let lib = path.clone().join("src/lib.rs");
    fs::create_dir_all(path)?;
    fs::create_dir_all(path.join("src"))?;
    fs::write(lib, cddl_gen::render_lib(&cddl, &options)?)?;
    fs::write(cargo, cddl_gen::render_cargo(&name, &options)?)?;
    if matches.get_flag("bindings") {
    } else {
    }

    Ok(())
}
