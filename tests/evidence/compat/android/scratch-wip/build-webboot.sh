cd /c/Users/win/src/origin-engine-webboot
export PATH="$HOME/.cargo/bin:/c/Users/win/.bun/bin:$PATH" SCCACHE_DIR='E:\sccache'
echo start $(git rev-parse --short HEAD) $(date -Iseconds)
(cd packages/engine && bun install --frozen-lockfile > /c/Users/win/src/origin-engine-android/.amp/in/build/wb-install.log 2>&1); echo install rc=$?
sh scripts/check.sh --only wasm-build wasm-opt > /c/Users/win/src/origin-engine-android/.amp/in/build/wb-wasm.log 2>&1; echo wasm rc=$? $(date -Iseconds)
OG_MOTION_MODEL=/e/src/og-verify/compat/model.safetensors OG_GATE=1 sh packages/engine/scripts/build.sh > /c/Users/win/src/origin-engine-android/.amp/in/build/wb-pkg.log 2>&1; echo pkg rc=$? $(date -Iseconds)
echo done-webboot