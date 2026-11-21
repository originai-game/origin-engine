#!/usr/bin/env bash
# Regenerates crates/og-ui-next/default/pack.json, the engine's default
# style pack, from the design language's tokens (packages/design/tokens/*.css):
#
#   crates/og-ui-next/scripts/gen-pack.sh
#
# og-ui-next's pack::design test fails when the file differs from this.
set -eu -o pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
cd "$root"
cargo run -q -p og-ui-next --example gen-pack -- crates/og-ui-next/default/pack.json
