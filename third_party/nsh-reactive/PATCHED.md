# nsh-reactive

From NSH (https://github.com/longbridge/nsh), commit
`a09da7c4235a0b55717bc365ce79a24302481d12`, Apache-2.0 (`LICENSE-APACHE`):
`src/lib.rs` is `crates/nsh-core/src/reactive.rs` and `src/tests.rs` is
`crates/nsh-core/src/reactive/tests.rs`, both unmodified. The file has no
dependency on the rest of nsh-core, so it is a crate of its own here; og-ui's
new core runs its components on it.

`Cargo.toml` allows `dead_code`: some helpers are called only from nsh-core
modules not kept here.
