cd /c/Users/win/src/origin-engine-android
export PATH="$HOME/.cargo/bin:$PATH" SCCACHE_DIR='E:\sccache'
cargo test -p og-tests --test tests -- hooks::skinned_hooks terrain::grass::the_wind_bends > .amp/in/build/baseline.log 2>&1; echo rc=$? >> .amp/in/build/baseline.log