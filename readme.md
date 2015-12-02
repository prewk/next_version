# What is it?

A really crappy [GitVersion](https://github.com/GitTools/GitVersion) made in Rust. If you use this in production, you're nuts.

# What's it for?

Tagging up releases automatically on CI servers.

# Building on a Mac

The `git2` crate needs `cmake` (it's on brew), and I had to supply the builder with `OPENSSL_ROOT_DIR` or it wouldn't compile [libgit2](https://github.com/libgit2/libgit2) Correctly:

````sh
OPENSSL_ROOT_DIR=/usr/local/opt/openssl cargo build
````

# Usage

````sh
next_version # In a git repo, defaults to current path
next_version /path/to/repo # Repo path
next_version --major-regexp "_major_" # Match "_major_" against commit messages for major bumps
next_version --minor-regexp "_feature_" # Match "_feature_" against commit messages for minor bumps
next_version --patch-regexp "_fix_" # Match "_fix_" against commit messages for patch bumps
````

# Rules

1. Finds highest current semver version tag.
2. Checks all commit messages from that point to look for bump messages.
3. If none were found, metadata increments a build number.
4. Prints out the suggested name for the next version's tag.

* If the repo lacks semver tags completely, the next suggested version will be `0.0.1`.
* If the current version is `1.2.3`, and a commit message matching a minor bump is detected, the next suggested version will be `1.3.0`.
* If the current version is `1.3.0`, and no bumps are detected, the next suggested version will be `1.3.0+1`.
* If the current version is `1.3.0+1`, and no bumps are detected, the next suggested version will be `1.3.0+2`.
* If the current version is `1.3.0+2`, and a patch bump is detected, the next suggested version will be `1.3.1`.
* If the current version is `1.3.1` and _no commit messages are found after that version_, the command will exit with a non-zero status.

## Default bump match patterns

* `:major:` for major bumps
* `:minor:` for minor bumps
* `:patch:` for patch bumps
