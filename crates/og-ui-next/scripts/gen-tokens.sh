#!/usr/bin/env bash
# Regenerates packages/design/tokens.json, the design language's tokens as
# one JSON tree (@origingame/design's main export), from its token files
# (packages/design/tokens/*.css):
#
#   crates/og-ui-next/scripts/gen-tokens.sh
#
# og-ui-next's tests/design_tokens.rs fails when the file differs from this.
set -eu -o pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
cd "$root"
cargo run -q -p og-ui-next --example gen-tokens -- packages/design/tokens packages/design/tokens.json
