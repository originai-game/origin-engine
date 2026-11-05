cd /c/Users/win/src/origin-engine-android
export PATH="$HOME/.cargo/bin:/c/Users/win/.bun/bin:$PATH" SCCACHE_DIR='E:\sccache'
while ! grep -q done-build .amp/in/build/window.log; do sleep 10; done
echo native2-start $(date -Iseconds)
OG_TEST_ASSETS=$PWD/crates/og-tests/assets cargo run -q -p og-tests-cooked --bin og-gfx -- --all > .amp/in/build/og-gfx-dx12.log 2>&1; echo og-gfx rc=$? $(date -Iseconds)
cargo test -p og-ui --test tests -- gfx:: > .amp/in/build/og-ui-gfx.log 2>&1; echo og-ui-gfx rc=$? $(date -Iseconds)
echo done-native2