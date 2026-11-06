# BLAKE3

The native cooker uses the published `blake3` crate, version 1.8.7, for action
keys and content verification. Source: https://github.com/BLAKE3-team/BLAKE3.
The crate offers CC0-1.0, Apache-2.0, or Apache-2.0 with LLVM exceptions.
This repository uses Apache-2.0; `LICENSE` is the crate's unmodified `LICENSE_A2`.
No BLAKE3 source is vendored here, and the wasm runtime does not depend on it.
