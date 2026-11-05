cd /c/Users/win/src/origin-engine-android
export PATH="$HOME/.cargo/bin:$PATH" SCCACHE_DIR='E:\sccache'
for i in 1 2 3; do cargo test -p og-tests --test tests -- terrain::grass::the_wind_bends > .amp/in/build/grass-$i.log 2>&1; echo run$i rc=$?; done