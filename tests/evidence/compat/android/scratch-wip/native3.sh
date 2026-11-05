cd /c/Users/win/src/origin-engine-android
export PATH="$HOME/.cargo/bin:$PATH" SCCACHE_DIR='E:\sccache'
echo start $(date -Iseconds)
cargo test -p og-tests --test render --test tests > .amp/in/build/og-tests-dx12.log 2>&1; echo og-tests rc=$? $(date -Iseconds)
cargo test -p og-ui --test tests -- gfx:: > .amp/in/build/og-ui-gfx.log 2>&1; echo og-ui-gfx rc=$? $(date -Iseconds)
echo done-native3