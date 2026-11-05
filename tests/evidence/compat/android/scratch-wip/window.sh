cd /c/Users/win/src/origin-engine-android
export PATH="$HOME/.cargo/bin:/c/Users/win/.bun/bin:$PATH" SCCACHE_DIR='E:\sccache'
echo native-start $(date -Iseconds)
cargo test -p og-render > .amp/in/build/render-test.log 2>&1; echo render rc=$? $(date -Iseconds)
cargo test -p og-tests --test gfx > .amp/in/build/gfx-dx12.log 2>&1; echo gfx-dx12 rc=$? $(date -Iseconds)
sh .amp/in/build/build.sh