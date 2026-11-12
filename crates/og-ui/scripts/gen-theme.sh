#!/usr/bin/env bash
# Regenerates crates/og-ui-core/{src/theme.rs,default.css} from the
# design language's tokens (packages/design/tokens.json):
#
#   crates/og-ui/scripts/gen-theme.sh
#
# og-ui's tests/theme.rs fails when either output differs from this.
set -eu -o pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
cd "$root"
cargo run -q -p og-ui --example gen-theme -- packages/design/tokens.json crates/og-ui-core/src/theme.rs crates/og-ui-core/default.css
