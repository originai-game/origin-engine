#!/usr/bin/env bash
# Regenerates crates/og-ui/{src/theme.rs,default.css} and
# packages/design/tokens.json from the design language's tokens
# (packages/design/tokens/*.css):
#
#   crates/og-ui/scripts/gen-theme.sh
#
# og-ui's tests/theme.rs fails when an output differs from this.
set -eu -o pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
cd "$root"
cargo run -q -p og-ui --example gen-theme -- packages/design/tokens crates/og-ui/src/theme.rs crates/og-ui/default.css packages/design/tokens.json
