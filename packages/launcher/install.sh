#!/bin/sh
# Installs og, Origin Engine's launcher (docs/release.md §1), into
# ~/.origin/bin/og ($ORIGIN_HOME/bin/og):
#
#   curl -fsSL https://origingame.ai/downloads/engine/install.sh | sh
#
# The launcher is one TypeScript file run by bun; it downloads each engine
# version a game pins, verified with the engine's signing key, into
# ~/.cache/origin-engine. Running this again replaces the launcher and
# leaves the installed engines alone.
set -eu

dir="${ORIGIN_HOME:-$HOME/.origin}"
say() { printf 'origin: %s\n' "$*"; }

mkdir -p "$dir/bin" "$dir/lib"
cat > "$dir/lib/og.ts.new" <<'__OG_LAUNCHER_END__'
__OG_LAUNCHER__
__OG_LAUNCHER_END__
mv "$dir/lib/og.ts.new" "$dir/lib/og.ts"

cat > "$dir/bin/og.new" <<__OG_SHIM_END__
#!/bin/sh
# og, Origin Engine's launcher: $dir/lib/og.ts on bun.
bun=\$(command -v bun 2>/dev/null || true)
[ -n "\$bun" ] || bun="\$HOME/.bun/bin/bun"
[ -x "\$bun" ] || { echo "og: bun is not installed; the engine runs on it: curl -fsSL https://bun.sh/install | bash" >&2; exit 1; }
exec "\$bun" "$dir/lib/og.ts" "\$@"
__OG_SHIM_END__
chmod 755 "$dir/bin/og.new"
mv "$dir/bin/og.new" "$dir/bin/og"
say "installed og in $dir/bin/og"

if ! command -v bun >/dev/null 2>&1 && [ ! -x "$HOME/.bun/bin/bun" ]; then
  say "og runs on bun, which is not installed: curl -fsSL https://bun.sh/install | bash"
fi
command -v tar >/dev/null 2>&1 || say "og unpacks engines with tar, which is not installed"
case ":${PATH:-}:" in
  *":$dir/bin:"*) ;;
  *) say "add $dir/bin to PATH, for instance in ~/.zshrc or ~/.bashrc: export PATH=\"$dir/bin:\$PATH\"" ;;
esac
say "then: og new my-game && cd my-game && og run ."
