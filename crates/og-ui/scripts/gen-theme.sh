#!/usr/bin/env bash
# Regenerates crates/og-ui/src/theme.rs, the engine's default style, from the
# design language's tokens (packages/design/tokens.json):
#
#   crates/og-ui/scripts/gen-theme.sh
#
# og-ui's tests/theme.rs fails when theme.rs is not exactly what this writes.
set -eu -o pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
cd "$root"
cargo run -q -p og-ui --example gen-theme -- packages/design/tokens.json crates/og-ui/src/theme.rs
