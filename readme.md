# Building on a Mac

The `git2` crate needs `cmake` (it's on brew), and I had to supply the builder with `OPENSSL_ROOT_DIR` or it wouldn't compile [libgit2](https://github.com/libgit2/libgit2) Correctly:

````sh
OPENSSL_ROOT_DIR=/usr/local/opt/openssl cargo build
````

# Usage

````sh
next_version # Shows current latest semver version or 0.0.1
next_version --major # Extracts latest and bumps major
next_version --minor # Extracts latest and bumps minor
next_version --patch # Extracts latest and bumps patch
next_version --patch ../foo # Specify git repo in another directory
````
