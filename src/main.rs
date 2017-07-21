extern crate semver;
extern crate git2;
extern crate clap;

use semver::Version;
use git2::Repository;
use std::io::{self, Write};
use clap::{Arg, App};

/// Get the highest version from the repo tags
fn get_highest_version(repo: &Repository) -> Option<Version> {
    // Get the tags
    let tags = repo.tag_names(None).unwrap();
    // Prepare a versions vector
    let mut versions = Vec::new();

    // Iterate through the tags
    for tag in tags.iter() {
        let name = tag.unwrap();
        // If a version tag is detected, push it to the versions vector
        match Version::parse(name) {
            Ok(parsed) => versions.push(parsed),
            _ => {}
        }
    }

    // Were there any versions at all?
    if versions.len() > 0 {
        // Yep, sort them
        versions.sort_by(|a, b| a.cmp(b));
        // Get the highest
        let highest_version = versions.last().unwrap();
        // Return a clone of it
        Some(highest_version.clone())
    } else {
        // Nope
        None
    }
}

fn main() {
    let matches = App::new("next_version")
        .version("2.0")
        .about("Helps guess the next git semver tag")
        .author("oskar.thornblad@gmail.com")
        .arg(
            Arg::with_name("repo_dir")
                .index(1)
                .short("r")
                .long("repo")
                .value_name("DIR")
                .help("Git repository")
                .takes_value(true)
        )
        .arg(Arg::with_name("major").short("M").long("major").help("Bump major version"))
        .arg(Arg::with_name("minor").short("m").long("minor").help("Bump minor version"))
        .arg(Arg::with_name("patch").short("p").long("patch").help("Bump patch version"))
        .get_matches();

    let repo_dir = matches.value_of("repo_dir").unwrap_or(".");

    let repo = Repository::open(repo_dir).expect("Couldn't open git repository");

    // Extract repo's highest version
    let highest_version = match get_highest_version(&repo) {
        Some(version) => version,
        None => Version {
            major: 0,
            minor: 0,
            patch: 1,
            pre: vec!(),
            build: vec!(),
        }
    };

    // Construct the new version
    let mut new_version = highest_version.clone();

    if matches.is_present("major") {
        new_version.increment_major();
    } else if matches.is_present("minor") {
        new_version.increment_minor();
    } else if matches.is_present("patch") {
        new_version.increment_patch();
    }

    let output_version = format!("{}.{}.{}", new_version.major, new_version.minor, new_version.patch);

    io::stdout().write(output_version.as_bytes()).unwrap();
}