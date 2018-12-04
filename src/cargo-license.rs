extern crate ansi_term;
extern crate cargo_license;
extern crate getopts;

use ansi_term::{ANSIGenericString, Colour::Green};
use getopts::Options;
use std::collections::btree_map::Entry::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;

#[derive(Debug)]
struct DisplayConfig {
    /// Display authors
    author: bool,
    /// Display colored strings
    color: bool,
}

impl DisplayConfig {
    pub fn new(author: bool, color: bool) -> Self {
        Self { author, color }
    }

    pub fn display_authors(&self) -> bool {
        self.author
    }

    pub fn with_colors<'a>(&self, str: &'a str) -> ANSIGenericString<'a, str> {
        if self.color {
            Green.bold().paint(str)
        } else {
            str.into()
        }
    }
}

fn group_by_license_type(dependencies: Vec<cargo_license::Dependency>, config: &DisplayConfig) {
    let mut table: BTreeMap<String, Vec<cargo_license::Dependency>> = BTreeMap::new();

    for dependency in dependencies {
        let license = dependency.get_license().unwrap_or("N/A".to_owned());
        match table.entry(license) {
            Vacant(e) => {
                e.insert(vec![dependency]);
            }
            Occupied(mut e) => {
                e.get_mut().push(dependency);
            }
        };
    }

    for (license, crates) in table {
        let crate_names = crates.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
        if config.display_authors() {
            let crate_authors = crates
                .iter()
                .flat_map(|c| c.get_authors().unwrap_or(vec![]))
                .collect::<BTreeSet<_>>();
            println!(
                "{} ({})\n{}\n{} {}",
                config.with_colors(&license),
                crates.len(),
                crate_names.join(", "),
                config.with_colors("by"),
                crate_authors.into_iter().collect::<Vec<_>>().join(", ")
            );
        } else {
            println!(
                "{} ({}): {}",
                config.with_colors(&license),
                crates.len(),
                crate_names.join(", ")
            );
        }
    }
}

fn one_license_per_line(dependencies: Vec<cargo_license::Dependency>, config: &DisplayConfig) {
    for dependency in dependencies {
        let name = dependency.name.clone();
        let version = dependency.version.clone();
        let license = dependency.get_license().unwrap_or("N/A".to_owned());
        let source = dependency.source.clone();
        if config.display_authors() {
            let authors = dependency.get_authors().unwrap_or(vec![]);
            println!(
                "{}: {}, \"{}\", {}, {} \"{}\"",
                config.with_colors(&name),
                version,
                license,
                source,
                config.with_colors("by"),
                authors.into_iter().collect::<Vec<_>>().join(", ")
            );
        } else {
            println!(
                "{}: {}, \"{}\", {}",
                config.with_colors(&name),
                version,
                license,
                source
            );
        }
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    let program = args[0].clone();
    opts.optflag("a", "authors", "Display crate authors");
    opts.optflag("d", "do-not-bundle", "Output one license per line.");
    opts.optflag("m", "without-color", "Output without color.");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            print_usage(&program, opts);
            panic!(f.to_string())
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let do_not_bundle = matches.opt_present("do-not-bundle");
    let config = DisplayConfig::new(
        matches.opt_present("authors"),
        !matches.opt_present("without-color"),
    );

    let dependencies = match cargo_license::get_dependencies_from_cargo_lock() {
        Ok(m) => m,
        Err(err) => {
            println!(
                "Cargo.lock file not found. Try building the project first.\n{}",
                err
            );
            std::process::exit(1);
        }
    };

    if do_not_bundle {
        one_license_per_line(dependencies, &config);
    } else {
        group_by_license_type(dependencies, &config);
    }
}
