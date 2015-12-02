# Usage

TODO

# Building on a Mac

The `git2` crate needs `cmake` (it's on brew), and I had to supply the builder with `OPENSSL_ROOT_DIR` to make it work:

````sh
OPENSSL_ROOT_DIR=/usr/local/opt/openssl cargo build
````
