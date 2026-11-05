cd /c/Users/win/src/origin-engine-android
export PATH="$HOME/.cargo/bin:/c/Users/win/.bun/bin:$PATH" SCCACHE_DIR='E:\sccache'
echo start $(git rev-parse --short HEAD) $(date -Iseconds)
sh scripts/check.sh --only wasm-build wasm-opt > .amp/in/build/wasm.log 2>&1; echo wasm rc=$? $(date -Iseconds)
OG_MOTION_MODEL=/e/src/og-verify/compat/model.safetensors OG_GATE=1 sh packages/engine/scripts/build.sh > .amp/in/build/pkg.log 2>&1; echo pkg rc=$? $(date -Iseconds)
cargo build --profile wasm --target wasm32-unknown-unknown --target-dir target/wasm-gfx -p og-tests-web --lib > .amp/in/build/gfx.log 2>&1; echo gfx rc=$? $(date -Iseconds)
echo done-build

sh /c/Users/win/src/origin-engine-android/.amp/in/build/build-webboot.sh
echo done-all
