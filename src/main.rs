extern crate semver;
extern crate git2;
extern crate regex;
extern crate getopts;

use std::process;
use semver::Version;
use git2::Repository;
use git2::SORT_TOPOLOGICAL;
use git2::SORT_TIME;
use git2::SORT_REVERSE;
use regex::Regex;
use std::io::{self, Write};
use std::env;
use getopts::Options;

/// Extracts build number metadata from a version if available and increments it
fn get_metadata(version: &Version) -> u64 {
    let mut build: u64;

    if version.build.len() == 1 {
        let build_id = &version.build[0];
        build = build_id.to_string().parse::<u64>().unwrap();
        build = build + 1;
    } else {
        build = 1;
    }

    return build;
}

#[derive(Debug)]
struct Bumps {
    objects: u64,
    major: bool,
    minor: bool,
    patch: bool
}

/// Walk the repository from the given version up to HEAD and find all commit messages with bumps
fn detect_bumps(repo: &Repository, start_version: &Version, major_pattern: &str, minor_pattern: &str, patch_pattern: &str) -> Bumps {
    // Extract the version's object
    let object = repo.revparse_single(&start_version.to_string()).unwrap();
    let sha = object.id();

    // Set up the revwalker
    let mut revwalker = repo.revwalk().unwrap();
    // Sort chronically backwards
    revwalker.set_sorting(SORT_TOPOLOGICAL | SORT_TIME | SORT_REVERSE);
    // Start at HEAD
    revwalker.push_head().unwrap();

    // Hide the version's object and everything older
    revwalker.hide(sha).unwrap();

    // Walk through all of the objects and detect the bumps
    let mut bumps = Bumps {
        objects: 0,
        major: false,
        minor: false,
        patch: false
    };

    // Set up reg-expes
    let major_re = Regex::new(major_pattern).unwrap();
    let minor_re = Regex::new(minor_pattern).unwrap();
    let patch_re = Regex::new(patch_pattern).unwrap();

    for sha in revwalker {
        // Count objects
        bumps.objects += 1;
        // Find object
        let sha_str = sha.to_string();
        let object = repo.revparse_single(&sha_str).unwrap();
        // Read as commit if possible
        match object.as_commit() {
            Some(commit) => {
                // Read commit message
                let message = commit.message().unwrap();

                // Match against our regular expressions
                if major_re.is_match(message) {
                    bumps.major = true;
                } else if minor_re.is_match(message) {
                    bumps.minor = true;
                } else if patch_re.is_match(message) {
                    bumps.patch = true;
                }
            }
            None => {}
        }
    }

    return bumps;
}

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

/// Help printout using getopts
fn print_usage(executable_path: &str, opts: Options) {
    let brief = format!("Usage: {} [REPOSITORY] [options]", executable_path);
    print!("{}", opts.usage(&brief));
}

fn main() {
    // Handle arguments
    let args: Vec<String> = env::args().collect();
    // Extract the executable's path
    let executable_path = args[0].clone();

    // Set up the options
    let mut opts = Options::new();
    opts.optopt("m", "major-regexp", "set major regexp pattern", "PATTERN");
    opts.optopt("f", "minor-regexp", "set minor regexp pattern", "PATTERN");
    opts.optopt("p", "patch-regexp", "set patch regexp pattern", "PATTERN");
    // Help flag
    opts.optflag("h", "help", "print this help menu");

    // Parse the arguments, crash on illegal arguments
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    // Print help
    if matches.opt_present("h") {
        print_usage(&executable_path, opts);
        return;
    }

    // Major regular expressions
    let major_regexp = match matches.opt_str("m") {
        Some(r) => r,
        None => r":major:".to_string()
    };
    // Minor regular expressions
    let minor_regexp = match matches.opt_str("m") {
        Some(r) => r,
        None => r":minor:".to_string()
    };
    // Patch regular expressions
    let patch_regexp = match matches.opt_str("m") {
        Some(r) => r,
        None => r":patch:".to_string()
    };

    // Repository directory
    let repo_dir = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        ".".to_string()
    };

    // Open the repo
    let repo = Repository::open(repo_dir).unwrap();
    // Retrieve the highest version
    let highest_version = match get_highest_version(&repo) {
        // Version found
        Some(version) => version,
        // Fallback to 0.0.1
        None => Version {
            major: 0,
            minor: 0,
            patch: 1,
            pre: vec!(),
            build: vec!()
        }
    };

    // Detect commit message bumps
    let bumps = detect_bumps(&repo, &highest_version, &major_regexp, &minor_regexp, &patch_regexp);
    // If no objects were walked, we don't want to tag the release at all
    if bumps.objects == 0 {
        process::exit(1);
    }

    // Clone to a new version
    let mut new_version = highest_version.clone();

    // Perform bumps
    if bumps.patch {
        new_version.increment_patch();
    }
    if bumps.minor {
        new_version.increment_minor();
    }
    if bumps.major {
        new_version.increment_major();
    }

    // Handle build metadata
    let build;
    if !bumps.patch && !bumps.minor && !bumps.major {
        build = get_metadata(&new_version);
    } else {
        build = 0;
    }

    // Construct a version name
    let output_version;
    if build == 0 {
        // If build metadata was 0, do not include it in the version name
        output_version = format!("{}.{}.{}", new_version.major, new_version.minor, new_version.patch);
    } else {
        output_version = format!("{}.{}.{}+{}", new_version.major, new_version.minor, new_version.patch, build);
    }

    // Send to stdout
    io::stdout().write(output_version.as_bytes()).unwrap();
}
