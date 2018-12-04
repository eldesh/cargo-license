
extern crate cargo_license;
extern crate ansi_term;
extern crate getopts;

use std::env;
use getopts::Options;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::btree_map::Entry::*;
use ansi_term::Colour::Green;

fn group_by_license_type(dependencies: Vec<cargo_license::Dependency>, display_authors: bool, without_color: bool) {

    let mut table: BTreeMap<String, Vec<cargo_license::Dependency>> = BTreeMap::new();

    for dependency in dependencies {
        let license = dependency.get_license().unwrap_or("N/A".to_owned());
        match table.entry(license) {
            Vacant(e) => {e.insert(vec![dependency]);},
            Occupied(mut e) => {e.get_mut().push(dependency);},
        };
    }

    for (license, crates) in table {
        let crate_names = crates.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
        if display_authors {
            let crate_authors = crates
                    .iter()
                    .flat_map(|c| c.get_authors().unwrap_or(vec![]))
                    .collect::<BTreeSet<_>>();
            println!("{} ({})\n{}\n{} {}",
                     if without_color {
                         license.into()
                     } else {
                         Green.bold().paint(license)
                     },
                     crates.len(),
                     crate_names.join(", "),
                     if without_color {
                         "by".into()
                     } else {
                         Green.paint("by")
                     },
                     crate_authors.into_iter().collect::<Vec<_>>().join(", "));
        } else {
            println!("{} ({}): {}",
                     if without_color {
                         license.into()
                     } else {
                         Green.bold().paint(license)
                     },
                     crates.len(),
                     crate_names.join(", "));
        }
    }
}

fn one_license_per_line(dependencies: Vec<cargo_license::Dependency>, display_authors: bool, without_color: bool) {

    for dependency in dependencies {
        let name = dependency.name.clone();
        let version = dependency.version.clone();
        let license = dependency.get_license().unwrap_or("N/A".to_owned());
        let source = dependency.source.clone();
        if display_authors {
            let authors = dependency.get_authors().unwrap_or(vec![]);
            println!("{}: {}, \"{}\", {}, {} \"{}\"",
                     if without_color {
                         name.into()
                     } else {
                         Green.bold().paint(name)
                     },
                     version,
                     license,
                     source,
                     if without_color {
                         "by".into()
                     } else {
                         Green.paint("by")
                     },
                     authors.into_iter().collect::<Vec<_>>().join(", "));
        } else {
            println!("{}: {}, \"{}\", {}",
                     if without_color {
                         name.into()
                     } else {
                         Green.bold().paint(name)
                     },
                     version,
                     license,
                     source);
        }
    };

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
        Err(f) => { print_usage(&program, opts); 
            panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let display_authors = matches.opt_present("authors");
    let do_not_bundle = matches.opt_present("do-not-bundle");
    let without_color = matches.opt_present("without-color");

    let dependencies = match cargo_license::get_dependencies_from_cargo_lock() {
        Ok(m) => m,
        Err(err) => {
            println!("Cargo.lock file not found. Try building the project first.\n{}", err);
            std::process::exit(1);
        }
    };

    if do_not_bundle {
        one_license_per_line(dependencies, display_authors, without_color);
    } else {
        group_by_license_type(dependencies, display_authors, without_color);
    }

}
