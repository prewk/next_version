extern crate semver;
extern crate git2;
extern crate regex;

use std::process;
use semver::Version;
use git2::Repository;
use git2::SORT_TOPOLOGICAL;
use git2::SORT_TIME;
use git2::SORT_REVERSE;
use regex::Regex;
use std::io::{self, Write};

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

struct Bumps {
    objects: u64,
    major: bool,
    minor: bool,
    patch: bool
}

fn detect_bumps(repo: &Repository, start_version: &Version) -> Bumps {
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
    let major_re = Regex::new(r"_MAJOR_").unwrap();
    let minor_re = Regex::new(r"_MINOR_").unwrap();
    let patch_re = Regex::new(r"_PATCH_").unwrap();

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

fn main() {

    let repo = Repository::open("./../test_repo").unwrap();

    let tags = repo.tag_names(None).unwrap();
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

    // Sort tags
    versions.sort_by(|a, b| a.cmp(b));

    // Extract highest version
    let highest_version = versions.last().unwrap();
    let results = repo.revparse_single(&highest_version.to_string()).unwrap();
    let highest_sha = results.id();

    let mut rw = repo.revwalk().unwrap();
    rw.set_sorting(SORT_TOPOLOGICAL | SORT_TIME | SORT_REVERSE);
    rw.push_head().unwrap();

    // Hide the highest versions commit & everything older
    rw.hide(highest_sha).unwrap();

    let mut major_bumps = false;
    let mut minor_bumps = false;
    let mut patch_bumps = false;

    let major_re = Regex::new(r"_MAJOR_").unwrap();
    let minor_re = Regex::new(r"_MINOR_").unwrap();
    let patch_re = Regex::new(r"_PATCH_").unwrap();

    // Step through the history
    let mut i = 0;

    for sha in rw {
        i += 1;
        let sha_str = sha.to_string();
        let object = repo.revparse_single(&sha_str).unwrap();
        match object.as_commit() {
            Some(commit) => {
                let message = commit.message().unwrap();

                if major_re.is_match(message) {
                    major_bumps = true;
                } else if minor_re.is_match(message) {
                    minor_bumps = true;
                } else if patch_re.is_match(message) {
                    patch_bumps = true;
                }
            }
            None => {}
        }
    }

    if i == 0 {
        process::exit(0);
    }

    let mut new_version = highest_version.clone();

    if patch_bumps {
        new_version.increment_patch();
    }
    if minor_bumps {
        new_version.increment_minor();
    }
    if major_bumps {
        new_version.increment_major();
    }

    let mut build = 0;
    if !patch_bumps && !minor_bumps && !major_bumps {
        if new_version.build.len() == 1 {
            let build_id = &new_version.build[0];
            build = build_id.to_string().parse::<u64>().unwrap();
            build = build + 1;
        } else {
            build = 1;
        }
    }

    let output_version;
    if build == 0 {
        output_version = format!("{}.{}.{}", new_version.major, new_version.minor, new_version.patch);
    } else {
        output_version = format!("{}.{}.{}+{}", new_version.major, new_version.minor, new_version.patch, build);
    }

    let output_version_u8 = output_version.as_bytes();

    io::stdout().write(output_version_u8).unwrap();
}
