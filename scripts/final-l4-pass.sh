#!/bin/bash
# The final L4 pass: one Colab L4 session over one main SHA that judges every
# look deferred to it (docs/ci.md §The final L4 pass).
#
#   scripts/final-l4-pass.sh bundle [DIR]      on the orb: build og, the addon, og-gfx, the GPU
#                                              test binaries and the web build of this checkout,
#                                              and gather what the run needs (default
#                                              ../final-l4-bundle, beside the checkout)
#   scripts/final-l4-pass.sh run BUNDLE OUT    on the GPU host, from the synced checkout: every
#                                              section, into OUT
#   scripts/final-l4-pass.sh run BUNDLE OUT --dry
#                                              the same on a CPU host, each section cut to a few
#                                              cases: proves the wiring, judges nothing
#   ... run BUNDLE OUT --only gallery,fused    some sections
#
# Sections: gallery (4 packs x dark/light with the lane: contrast, missing glyphs, clip, settle),
# text (8956's measuring scripts, scripts/text-l4/run.sh), fused (the frames either side of every
# cut of packages/engine/tests/fixtures/fused, won and lost routes), gpu-tests (og-render's ui_paint,
# the float-output blend included), web (the web-reference groups in Chrome), web-compat (the
# browser's compatibility mode read back, the engine's Compat tier, the adapter retry), filters (the
# UI filter scenes per pack and scheme, and their 0.5 ms budget at 1080p), gfx (og-gfx's core
# and compat scenes against main's references, movements kept for review, not a gate).
#
# OUT holds, per section, <section>/sheet-*.jpg contact sheets and its logs; summary.txt (the SHA,
# the adapters, each section's wall time and result), lane.txt (the gallery lane's lines) and
# failures.txt (one line per finding or section that could not run). The exit code is 0 only when
# failures.txt is empty.
set -uo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
SECTIONS="gallery text fused gpu-tests web web-compat filters gfx"

die() { echo "final-l4-pass: $*" >&2; exit 2; }

# ---------------------------------------------------------------- bundle (orb)
bundle() {
  local b="${1:-$root/../final-l4-bundle}" target="${CARGO_TARGET_DIR:-$root/target}"
  export PATH="$HOME/.cargo/bin:$PATH"
  mkdir -p "$b/bin/t/deps" "$b/web"
  b="$(cd "$b" && pwd)"
  cd "$root"
  git rev-parse HEAD > "$b/sha"
  git status --porcelain --untracked-files=no | grep -q . && echo "final-l4-pass: bundle: the checkout has uncommitted changes; the pass names $(cut -c1-9 "$b/sha") anyway" >&2
  cargo build -q -p og-cli -p og-host-node -p og-tests-cooked --bin og --bin og-gfx --lib || die "bundle: cargo build failed"
  cp "$target/debug/og" "$target/debug/og-gfx" "$b/bin/"
  cp "$target/debug/libog_host_node.so" "$b/bin/og-addon.node"
  # The GPU test binaries; the run finds them by name.
  local pkg exe
  for pkg in og-render og-runtime og-tests og-tests-cooked; do
    exe="$(cargo test -q -p "$pkg" --test tests --no-run --message-format=json 2>/dev/null \
      | python3 -c 'import json,sys
for l in sys.stdin:
  m=json.loads(l)
  if m.get("reason")=="compiler-artifact" and m.get("executable") and m["target"]["kind"]==["test"]: print(m["executable"])' | tail -1)"
    [ -n "$exe" ] || die "bundle: no test binary for $pkg"
    cp "$exe" "$b/bin/t/deps/$pkg-tests"
  done
  strip -g "$b/bin/og" "$b/bin/og-gfx" "$b/bin/og-addon.node" "$b"/bin/t/deps/* 2>/dev/null
  # The web build the web groups serve, with the 3D publish pair lane-panic loads (the gate's
  # reference variants: OG_GATE_SKIP=web-reference scripts/check.sh full --only web-reference).
  [ -f packages/engine/dist/web-publish/origin-engine.js ] || die "bundle: no packages/engine/dist/web-publish; run OG_GATE_SKIP=web-reference scripts/check.sh full --only web-reference first"
  rm -rf "$b/web" && mkdir -p "$b/web" && cp -r packages/engine/dist/web packages/engine/dist/web-publish packages/engine/native "$b/web/"
  # Ignored assets the checkout's sync leaves behind (the template fonts the text section's Chrome
  # sheets use, the fused fixture's footage, vehicle-terrain's assets): fetched here, shipped in the
  # bundle, put back by run.
  local where="templates crates/og-tests packages/engine/tests/fixtures/fused tests/fixtures/vehicle-terrain packages/engine/vendor/basis packages/engine/motion packages/engine/shell/brand"
  # shellcheck disable=SC2086
  bun scripts/assets.ts ensure $where >/dev/null || die "bundle: assets ensure failed"
  rm -rf "$b/assets" && mkdir -p "$b/assets"
  # shellcheck disable=SC2086
  git ls-files -z -o -i --exclude-standard -- $where | grep -zvE '(^|/)(cooked|\.og|node_modules|og-out|dist)/' \
    | tar --null -T - -cf - | tar -xf - -C "$b/assets"
  du -sh "$b" | sed 's/^/final-l4-pass: bundle /'
  # Colab's upload refuses a large archive (a 290 MB one failed, 206 MB went through): the bundle
  # also goes up as parts of at most 80 MB each, hard links into it, which assemble puts together.
  local parts="$b.parts"
  rm -rf "$parts"
  python3 - "$b" "$parts" <<'PY'
import os, sys
b, parts = sys.argv[1], sys.argv[2]
files = []
for r, _, fs in os.walk(b):
    for f in fs:
        p = os.path.join(r, f); files.append((os.path.relpath(p, b), os.path.getsize(p)))
groups, cur, size = [], [], 0
for rel, n in sorted(files, key=lambda x: -x[1]):
    if cur and size + n > 80 << 20: groups.append(cur); cur, size = [], 0
    cur.append(rel); size += n
groups.append(cur)
for i, g in enumerate(groups):
    for rel in g:
        d = os.path.join(parts, f"p{i:02d}", os.path.dirname(rel)); os.makedirs(d, exist_ok=True)
        os.link(os.path.join(b, rel), os.path.join(parts, f"p{i:02d}", rel))
print(f"final-l4-pass: {len(groups)} parts in {parts}")
PY
  echo "final-l4-pass: next: colab_gpu_sync the checkout, then each $parts/pNN (repo final-l4-pNN), then on the host: scripts/final-l4-pass.sh assemble /content/work /content/work/final-l4-bundle, then run"
}

# ---------------------------------------------------------------- assemble (GPU host)
# assemble WORK BUNDLE: the synced parts (WORK/final-l4-pNN) put together as BUNDLE, by hard links.
assemble() {
  local work="$1" b="$2" p
  rm -rf "$b"; mkdir -p "$b"
  for p in "$work"/final-l4-p[0-9][0-9]; do cp -al "$p/." "$b/"; done
  chmod +x "$b/bin/og" "$b/bin/og-gfx" "$b"/bin/t/deps/* "$b"/web/native/og.* 2>/dev/null
  [ -f "$b/sha" ] || die "assemble: no parts under $work"
  echo "final-l4-pass: assembled $(cut -c1-9 "$b/sha") in $b ($(du -sh "$b" | cut -f1))"
}

# ---------------------------------------------------------------- run (GPU host)
OUT="" BUNDLE="" DRY=0 MONTAGE=montage IM=magick
fail() { echo "$1: $2" >> "$OUT/failures.txt"; echo "FAIL $1: $2" >&2; }
note() { echo "$*" >> "$OUT/summary.txt"; echo "$*"; }

# sheet DIR NAME PNG...: a contact sheet of PNGs (labelled by file name), 40 to a page.
sheet() {
  local dir="$1" name="$2"; shift 2
  [ "$#" -gt 0 ] || return 0
  local page=0
  while [ "$#" -gt 0 ]; do
    local chunk=("${@:1:40}"); shift $(( $# < 40 ? $# : 40 ))
    $MONTAGE -label '%t' "${chunk[@]}" -tile 5x -geometry 384x216+4+4 -pointsize 11 -background '#333' -fill '#ddd' -quality 85 "$dir/sheet-$name-$(printf %02d $page).jpg" 2>/dev/null
    page=$((page + 1))
  done
}

# blank PNG: why the frame is black or blank (one flat colour), else nothing.
blank() {
  local s; s="$($IM "$1" -colorspace gray -format '%[fx:mean] %[fx:standard_deviation]' info: 2>/dev/null)" || { echo "unreadable"; return; }
  awk -v s="$s" 'BEGIN { split(s, a, " "); if (a[1] < 0.02) print "black (mean " a[1] ")"; else if (a[2] < 0.004) print "blank (one colour, sd " a[2] ")" }'
}

timed() { # timed SECTION: runs sec_SECTION, records its wall time
  local s="$1" t0 rc
  t0="$(date +%s)"
  mkdir -p "$OUT/$s"
  "sec_${s//-/_}" > "$OUT/$s/run.log" 2>&1; rc=$?
  note "$s: $([ "$rc" = 0 ] && echo done || echo "exit $rc") in $(( $(date +%s) - t0 ))s"
  [ "$rc" = 0 ] || fail "$s" "the section exited $rc (log $s/run.log)"
}

sec_gallery() {
  local only=()
  [ "$DRY" = 1 ] && only=(--only "hud/crosshair · default,shooter/radar · default,racing/garage · default,screens/subtitles · captions")
  rm -rf "$root/target/ui-gallery/project"
  (cd "$root/packages/engine" && OG_NATIVE_ADDON="$BUNDLE/bin/og-addon.node" bun src/node/gallery.ts --og "$BUNDLE/bin/og" \
    --packs default,cartoon,fantasy,scifi --schemes dark,light --out "$OUT/gallery/frames" "${only[@]}")
  local rc=$?
  cp "$OUT/gallery/frames/defects.txt" "$OUT/lane.txt" 2>/dev/null || : > "$OUT/lane.txt"
  local c
  for c in "$OUT"/gallery/frames/*/; do
    c="$(basename "$c")"; [ "$c" = rest ] && continue
    # shellcheck disable=SC2046
    sheet "$OUT/gallery" "$c" $(ls "$OUT/gallery/frames/$c"/*.png 2>/dev/null)
  done
  note "gallery: $(wc -l < "$OUT/lane.txt") lane lines; $(grep -c 'draws no glyph\|tofu' "$OUT/lane.txt") missing-glyph, $(grep -c 'draws nothing' "$OUT/lane.txt") empty"
  # The lane exits 1 on its own lines: findings for review, not a broken run.
  [ "$rc" -le 1 ]
}

sec_text() {
  # 8956's measuring scripts, called as they are (scripts/text-l4/run.sh): the sheets against
  # Chrome at 12-48 px per pack, the mixed-script frames, the captions and → lines, and the cold
  # time to the first CJK glyph natively and in Chrome on the NVIDIA adapter.
  if [ ! -x "$root/scripts/text-l4/run.sh" ]; then
    fail text "not wired: scripts/text-l4/run.sh (8956's text-l4-scripts) is not in this checkout"
    return 0
  fi
  [ -n "${CHROME:-}" ] || { fail text "no Chrome"; return 0; }
  local rc
  ( export OG_ENGINE_PACKAGE="$root/packages/engine" RT_TESTS="$BUNDLE/bin/t/deps/og-runtime-tests"
    [ "$DRY" = 1 ] && export SWIFTSHADER=1
    cd "$root" && scripts/text-l4/run.sh "$BUNDLE/bin/og" "$CHROME" "$OUT/text" ) ; rc=$?
  [ -s "$OUT/text/failures.txt" ] && sed 's/^/text: /' "$OUT/text/failures.txt" >> "$OUT/failures.txt"
  # shellcheck disable=SC2046
  sheet "$OUT/text" frames $(find "$OUT/text" -name 'cmp-*.png' -o -name 'og-*.png' | sort)
  [ "$rc" -le 1 ]
}

# The fused fixture: one run dumps the flow every tick to find the cuts, a second shoots the
# frames either side of each. KeyD at 3 s wins the held level (cutscene, then the live-action
# ending); KeyQ loses it.
sec_fused() {
  local src="$root/packages/engine/tests/fixtures/fused" route keys end size=1280x720
  [ "$DRY" = 1 ] && size=320x180
  for route in won lost; do
    case $route in
      won) keys="3:key_down:KeyD"; end=11 ;;
      lost) keys="3:key_down:KeyQ;3.1:key_up:KeyQ"; end=5 ;;
    esac
    local w="$OUT/fused/$route" p; p="$(mktemp -d)"
    mkdir -p "$w"; cp -r "$src/." "$p/"; rm -rf "$p/cooked" "$p/.og"
    local cmds
    cmds="$(python3 -c "
import sys
end=float(sys.argv[2]); s=[f'{t/60:.4f}:dump_flow:f{t:04d}.json' for t in range(1,int(end*60))]
s+=sys.argv[1].split(';'); s.sort(key=lambda x: float(x.split(':')[0])); print(';'.join(s+[f'{end+0.1}:quit']))" "$keys" "$end")"
    (cd "$p" && "$BUNDLE/bin/og" run . --headless --no-check --size "$size" --out "$w/dumps" --commands "$cmds") > "$w/dumps.log" 2>&1 \
      || { fail fused "$route: the flow run failed (fused/$route/dumps.log)"; continue; }
    # Cut ticks: the first tick whose dump names a new node.
    local cuts
    cuts="$(python3 -c "
import json,os,sys
d=sys.argv[1]; prev=None; out=[]
for f in sorted(x for x in os.listdir(d) if x.startswith('f') and x.endswith('.json')):
  t=int(f[1:5]); n=(json.load(open(os.path.join(d,f))).get('flows') or [{}])[0].get('node')
  if n!=prev: out.append(f'{t}:{n}'); prev=n
print(' '.join(out))" "$w/dumps")"
    echo "$cuts" > "$w/cuts.txt"
    note "fused $route: cuts $cuts"
    [ -n "$cuts" ] || { fail fused "$route: no cut found in the flow dumps"; continue; }
    cmds="$(python3 -c "
import sys
ts=set([30])
for c in sys.argv[2].split():
  t=int(c.split(':')[0]); ts.update(range(max(1,t-2),t+4))
s=[f'{t/60:.4f}:screenshot:t{t:04d}.png' for t in sorted(ts)]+sys.argv[1].split(';')
s.sort(key=lambda x: float(x.split(':')[0])); print(';'.join(s+[f'{max(ts)/60+0.2:.4f}:quit']))" "$keys" "$cuts")"
    rm -rf "$p/cooked" "$p/.og"
    (cd "$p" && "$BUNDLE/bin/og" run . --headless --no-check --size "$size" --out "$w/shots" --commands "$cmds") > "$w/shots.log" 2>&1 \
      || { fail fused "$route: the capture run failed (fused/$route/shots.log)"; continue; }
    local f why
    for f in "$w"/shots/*.png; do
      why="$(blank "$f")"
      # Boot (to tick 30) may stand behind the loading screen; every later frame must draw.
      local t; t="$(basename "$f" .png)"; t=$((10#${t#t}))
      [ -n "$why" ] && [ "$t" -gt 30 ] && fail fused "$route $(basename "$f" .png): $why"
    done
    # shellcheck disable=SC2046
    sheet "$OUT/fused" "$route" $(ls "$w"/shots/*.png)
    rm -rf "$p"
  done
}

# og-render's UI paint tests (bb11's float-output blend among them): unchecked on lavapipe, run here.
sec_gpu_tests() {
  local log="$OUT/gpu-tests/ui_paint.txt"
  "$BUNDLE/bin/t/deps/og-render-tests" ui_paint --test-threads=1 > "$log" 2>&1; local rc=$?
  grep -q "missing GPU" "$log" && fail gpu-tests "ui_paint: no GPU adapter, so the tests were skipped, never passed"
  [ "$rc" = 0 ] || fail gpu-tests "ui_paint: exit $rc ($(grep -c '^test .* FAILED' "$log") failed; gpu-tests/ui_paint.txt)"
  note "gpu-tests: $(grep -E '^test result' "$log" | tail -1)"
  grep -E "on (.*NVIDIA|.*llvmpipe|.*SwiftShader)" "$log" | head -1 | sed 's/^/gpu-tests adapter: /' >> "$OUT/summary.txt"
  return 0
}

# The web-reference groups (scripts/gate/commands.sh WEB_GROUPS) in Chrome on the NVIDIA adapter.
sec_web() {
  local e="$root/packages/engine" g rc
  export OG_WEB_GPU=nvidia
  [ "$DRY" = 1 ] && unset OG_WEB_GPU
  declare -A cmd=(
    [ui]="bun tests/web/a11y.ts && bun tests/web/autoplay.ts"
    [gfx]="bun tests/web/reference-gfx.ts --out $OUT/web/gfx"
    [vehicle-terrain]="bun tests/web/reference.ts --project tests/fixtures/vehicle-terrain --playtest tests/drive.og.ts --out $OUT/web/vehicle-terrain"
    [lane-panic]="bun tests/web/lane-panic.ts"
    [video]="bun tests/web/video-alpha.ts --out $OUT/web/video-alpha"
  )
  local groups="ui gfx vehicle-terrain lane-panic video"
  [ "$DRY" = 1 ] && groups="lane-panic video"
  for g in $groups; do
    local t0; t0="$(date +%s)"
    (cd "$e" && eval "${cmd[$g]}") > "$OUT/web/$g.log" 2>&1; rc=$?
    note "web $g: $([ "$rc" = 0 ] && echo PASS || echo "FAIL exit $rc") in $(( $(date +%s) - t0 ))s"
    [ "$rc" = 0 ] || fail web "$g: exit $rc (web/$g.log)"
  done
  # The adapter each timing and frame was made on; one marked (software) is not counted.
  grep -rhoE '"adapter": *"[^"]*"' "$OUT/web" 2>/dev/null | sort -u | sed 's/^/web adapter: /' >> "$OUT/summary.txt"
  grep -rqE '"adapter": *"[^"]*software' "$OUT/web" 2>/dev/null && [ "$DRY" = 0 ] && fail web "a web run named a software adapter: its frames and times are not counted"
  # shellcheck disable=SC2046
  sheet "$OUT/web" frames $(find "$OUT/web" -name '*.png' ! -name '*.diff*' | sort)
  return 0
}

# Web compat, in L4 Chrome through scripts/l4-chrome.sh (the wgpu fork thread's harness, called as
# it is). Linux Chrome's opengles adapter crashes its GPU process at a canvas's first present, so
# the browser's compatibility mode is checked without one, and the engine's Compat tier presents
# on the core adapter:
#   readback   compat-readback.ts on the opengles adapter: a compatibility device draws into a
#              texture and reads it back (a validation error, a lost device, a core adapter or
#              wrong pixels fails)
#   tier       reference.ts --tier compat on the core adapter, 2D and 3D, against the references:
#              its stations' RMSE, and 0 "[og] WebGPU error:" lines in Chrome's log
#   retry      adapter-probe.ts, cold launches on both adapters: what the first requestAdapter
#              answers, the null the engine's adapter retry covers (facts, not a verdict)
sec_web_compat() {
  local e="$root/packages/engine" w="$OUT/web-compat" rc
  if [ ! -f "$e/tests/web/compat-readback.ts" ]; then
    fail web-compat "not wired: tests/web/compat-readback.ts and reference.ts --tier compat (the wgpu fork's async-pipelines head) are not in this checkout"
    return 0
  fi
  [ -n "${CHROME:-}" ] || { fail web-compat "no Chrome"; return 0; }
  # The harness's own GPU switches are the L4 set, which the wrapper replaces with its own.
  export OG_L4_CHROME_BIN="$CHROME" OG_WEB_GPU=nvidia
  local chrome="$root/scripts/l4-chrome.sh" runs=3 probes=4 errs
  [ "$DRY" = 1 ] && runs=1 probes=1

  (cd "$e" && OG_L4_COMPAT=1 OG_L4_ANGLE=gl-egl OG_L4_CHROME_LOG="$w/readback-chrome.log" \
    timeout 300 bun tests/web/compat-readback.ts --chrome "$chrome" --runs "$runs") > "$w/readback.jsonl" 2> "$w/readback.log"; rc=$?
  note "web-compat readback: $([ "$rc" = 0 ] && echo PASS || echo "FAIL exit $rc"), $(grep -c '"ok":true' "$w/readback.jsonl")/$runs runs ok"
  grep -m1 -oE '"adapter":\{[^}]*\}' "$w/readback.jsonl" | sed 's/^/web-compat readback adapter: /' >> "$OUT/summary.txt"
  [ "$rc" = 0 ] || fail web-compat "readback: exit $rc: $(grep -ohE '"findings":\[[^]]*\]' "$w/readback.jsonl" | sort -u | tr '\n' ' ')(web-compat/readback.jsonl)"

  local case p t
  local cases="side-scroller:slash vehicle-terrain:drive"
  [ "$DRY" = 1 ] && cases="side-scroller:slash"
  for case in $cases; do
    p="${case%%:*}" t="${case#*:}"
    (cd "$e" && OG_L4_COMPAT=0 OG_L4_CHROME_LOG="$w/tier-$p-chrome.log" timeout 600 bun tests/web/reference.ts \
      --project "tests/fixtures/$p" --playtest "tests/$t.og.ts" --tier compat --chrome "$chrome" --out "$w/tier-$p") > "$w/tier-$p.log" 2>&1; rc=$?
    errs="$(cat "$w/tier-$p-chrome.log" 2>/dev/null | grep -c '\[og\] WebGPU error:')"
    note "web-compat tier $p: $([ "$rc" = 0 ] && echo PASS || echo "FAIL exit $rc"), $errs WebGPU errors, rmse $(python3 -c '
import json,sys
try: r=json.load(open(sys.argv[1]))
except Exception: print("none"); sys.exit()
print(" ".join(k+"="+str(v.get("rmse")) for k,v in (r.get("stations") or {}).items()) or "none", r.get("renderer") or "")' "$w/tier-$p/reference-webgpu.json")"
    [ "$rc" = 0 ] || fail web-compat "tier $p: exit $rc$([ "$rc" = 124 ] && echo ' (hung)') (web-compat/tier-$p.log)"
    [ "$errs" = 0 ] || fail web-compat "tier $p: $errs WebGPU validation errors (web-compat/tier-$p-chrome.log)"
  done

  local m
  for m in 0 1; do
    (cd "$e" && OG_L4_COMPAT=$m OG_L4_ANGLE="$([ "$m" = 1 ] && echo gl-egl || echo vulkan)" OG_L4_CHROME_LOG="$w/retry-$m-chrome.log" \
      timeout 300 bun tests/web/adapter-probe.ts "$chrome" "$probes") > "$w/retry-$m.txt" 2>&1
    sed "s/^/web-compat retry: /" "$w/retry-$m.txt" >> "$OUT/summary.txt"
    grep -q 'early \[{"ms":[0-9]*,"ok":false[^]]*"ok":false' "$w/retry-$m.txt" && fail web-compat "retry compat=$m: a cold launch got no adapter on either request (web-compat/retry-$m.txt)"
    grep -q '^run [0-9]* error' "$w/retry-$m.txt" && fail web-compat "retry compat=$m: the probe did not answer (web-compat/retry-$m.txt)"
    [ "$DRY" = 0 ] && grep -q '"arch":"swiftshader"' "$w/retry-$m.txt" && fail web-compat "retry compat=$m: Chrome gave a software adapter, so this section's results are not counted (web-compat/retry-$m.txt)"
  done
  grep -h 'GPU process exited unexpectedly' "$w"/*-chrome.log 2>/dev/null | wc -l | sed 's/^/web-compat GPU-process crashes: /' >> "$OUT/summary.txt"
  # shellcheck disable=SC2046
  sheet "$w" frames $(find "$w" -name '*.png' | sort)
  return 0
}

# The UI filter scenes (scripts/final-l4/filters: glass, shadows, spotlight, minimap), shot per
# pack and scheme for review, and the filters' cost at 1080p: og cost of the four together, with
# and without their filter classes, held to the 0.5 ms GPU budget (offscreen filter plan §6).
FILTER_BUDGET_MS=0.5
sec_filters() {
  local src="$root/scripts/final-l4/filters" w="$OUT/filters" p
  p="$(mktemp -d)"; cp -r "$src/." "$p/"; rm -rf "$p/cooked" "$p/.og" "$p/node_modules"
  mkdir -p "$p/node_modules/@origingame" && ln -sfn "$root/packages/engine" "$p/node_modules/@origingame/engine"
  local packs="default cartoon fantasy scifi" schemes="dark light" scenes="glass shadows spotlight minimap all hud" size=1920x1080 pk sc s
  # The dry run proves the wiring only: one scene, small, and no timing (a CPU judges no budget).
  [ "$DRY" = 1 ] && packs="default" schemes="dark" scenes="all" size=320x180
  # One run a scene: a mount is never taken down mid-run.
  for pk in $packs; do for sc in $schemes; do for s in $scenes; do
    (cd "$p" && OG_NATIVE_ADDON="$BUNDLE/bin/og-addon.node" "$BUNDLE/bin/og" run . --headless --no-check --size "$size" --out "$w/shots" \
      --commands "0.02:pack:$pk;0.03:scheme:$sc;0.05:scene:$s;1.5:screenshot:$pk-$sc-$s.png;1.6:quit") > "$w/shots-$pk-$sc-$s.log" 2>&1 \
      || fail filters "$pk $sc $s: the capture run failed (filters/shots-$pk-$sc-$s.log)"
  done; done; done
  local f why
  for f in "$w"/shots/*.png; do why="$(blank "$f")"; [ -n "$why" ] && fail filters "$(basename "$f" .png): $why"; done
  # shellcheck disable=SC2046
  sheet "$w" scenes $(ls "$w"/shots/*.png 2>/dev/null)
  # The budget: the filter plan's §6 HUD (scene hud: 4 blurred cards, 8 drop-shadowed icons, 2
  # backdrops), with its filters and without. Not on a dry run.
  if [ "$DRY" = 1 ]; then note "filters: cost not measured on a dry run"; rm -rf "$p"; return 0; fi
  local end=240
  (cd "$p" && OG_NATIVE_ADDON="$BUNDLE/bin/og-addon.node" "$BUNDLE/bin/og" cost . --json --runs 8 --ticks "60..$end" \
    --on "--commands 0.02:scene:hud" --off "--commands 0.02:scene:hud:plain" -- --headless --no-check --size 1920x1080 --ticks $((end + 20))) > "$w/cost.json" 2> "$w/cost.log"
  local rc=$? ms
  ms="$(python3 -c 'import json,sys
try: r=json.load(open(sys.argv[1]))
except Exception: sys.exit()
v=(r.get("cost") or {}).get("median")
print("" if v is None else round(v, 3))' "$w/cost.json")"
  note "filters: cost at 1920x1080 $([ -n "$ms" ] && echo "$ms ms, median GPU frame with filters less without (budget $FILTER_BUDGET_MS; range and load in filters/cost.json)" || echo "unmeasured (exit $rc, filters/cost.log)")"
  if [ -z "$ms" ]; then fail filters "cost: og cost gave no difference (exit $rc, filters/cost.log)"
  elif awk -v m="$ms" -v b="$FILTER_BUDGET_MS" 'BEGIN { exit !(m > b) }'; then fail filters "cost: the filters take $ms ms at 1080p, over the $FILTER_BUDGET_MS ms budget"
  fi
  rm -rf "$p"
  return 0
}

# og-gfx's core (og-tests and og-tests-cooked `gfx`) and compat scenes against main's references.
# A moved frame is reported with its actual and diff PNGs, for review; only a crash, a black or
# blank frame or an unchecked run is a failure.
sec_gfx() {
  local rep="$OUT/gfx/movements" pkg filter=gfx rc
  mkdir -p "$rep"
  [ "$DRY" = 1 ] && filter="gfx::ui_shapes"
  export OG_REFERENCE_REPORT_DIR="$rep" OG_TEST_REFERENCES="$root/crates/og-tests/references" OG_TEST_ASSETS="$root/crates/og-tests/assets"
  # One process a scene, as nextest runs them: in one process the GPU memory of scene after scene
  # piles up (21 GB by the tenth on the L4) and every later scene fails out of memory.
  local t bin passed failed
  for pkg in og-tests og-tests-cooked; do
    bin="$BUNDLE/bin/t/deps/$pkg-tests" passed=0 failed=0
    : > "$OUT/gfx/$pkg.txt"
    for t in $("$bin" "$filter" --list 2>/dev/null | sed -n 's/: test$//p'); do
      "$bin" "$t" --exact --test-threads=1 >> "$OUT/gfx/$pkg.txt" 2>&1; rc=$?
      if [ "$rc" = 0 ]; then passed=$((passed + 1)); else failed=$((failed + 1)); fail gfx "$pkg $t: exit $rc (a crash, a broken frame or a missing reference; gfx/$pkg.txt)"; fi
    done
    grep -q "missing GPU" "$OUT/gfx/$pkg.txt" && fail gfx "$pkg: no GPU adapter, so the scenes were skipped, never passed"
    note "gfx $pkg: $passed passed, $failed failed, one process a scene"
  done
  grep -h -m1 -oE '\[[a-z0-9-]+ on [^,]*' "$OUT"/gfx/*.txt | head -1 | sed 's/^/gfx adapter: /' >> "$OUT/summary.txt"
  # The movements: reference | actual | diff, side by side, one row each.
  local out_dir="$BUNDLE/bin/tmp/og-gfx" a s n=0
  mkdir -p "$OUT/gfx/frames"
  [ -d "$out_dir" ] && cp -r "$out_dir/." "$OUT/gfx/frames/"
  for a in $(find "$OUT/gfx/frames" -name '*.actual.png' | sort); do
    s="${a%.actual.png}"
    $MONTAGE -label '%t' "$a" "$s.diff.png" -tile 2x -geometry 640x360+4+4 -background '#333' -fill '#ddd' "$OUT/gfx/sheet-$(basename "$s").jpg" 2>/dev/null
    n=$((n + 1))
  done
  note "gfx: $n moved frames (gfx/sheet-*.jpg), for quality review"
  return 0
}

run() {
  BUNDLE="$(cd "$1" && pwd)" || die "run: no bundle at $1"; OUT="$2"; shift 2
  local only="$SECTIONS" a
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --dry) DRY=1 ;;
      --only) only="${2//,/ }"; shift ;;
      *) die "run: unknown argument $1" ;;
    esac
    shift
  done
  for a in "$BUNDLE/bin/og" "$BUNDLE/bin/og-addon.node" "$BUNDLE/sha"; do [ -e "$a" ] || die "run: $a missing (scripts/final-l4-pass.sh bundle)"; done
  rm -rf "$OUT"; mkdir -p "$OUT"; OUT="$(cd "$OUT" && pwd)"; : > "$OUT/failures.txt"
  # The test binaries read the checkout at the path they were built in (CARGO_MANIFEST_DIR).
  local built="/home/user/workspace/repo"
  if [ "$root" != "$built" ] && [ ! -e "$built" ]; then mkdir -p "$(dirname "$built")" && ln -s "$root" "$built"; fi
  command -v montage >/dev/null || command -v magick >/dev/null || { apt-get install -y -qq imagemagick >/dev/null 2>&1 || fail setup "no ImageMagick: no contact sheets"; }
  # ImageMagick 7's montage labels each frame with its name; 6's needs the label before reading.
  MONTAGE="montage" IM="convert"; command -v magick >/dev/null && MONTAGE="magick montage" IM="magick"
  if [ -z "${CHROME:-}" ] && [[ " $only " == *" web "* || " $only " == *" web-compat "* || " $only " == *" text "* ]]; then
    CHROME="$(ls -d "$HOME"/.agent-browser/browsers/chrome-*/chrome 2>/dev/null | tail -1)"
    if [ -z "$CHROME" ]; then
      (cd /tmp && bunx @puppeteer/browsers install chrome@stable --path /content/chrome >/dev/null 2>&1)
      CHROME="$(ls -d /content/chrome/chrome/linux-*/chrome-linux64/chrome 2>/dev/null | tail -1)"
      # Chrome for Testing ships no libraries: installing Google Chrome's package pulls them all in,
      # under whatever names the host's release gives them.
      (cd /tmp && curl -fsSLo chrome.deb https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb \
        && apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq ./chrome.deb) >/dev/null 2>&1
      ldd "$CHROME" 2>/dev/null | grep -q "not found" && fail setup "Chrome's libraries are missing: $(ldd "$CHROME" | grep "not found" | awk '{print $1}' | tr '\n' ' ')"
    fi
    [ -n "$CHROME" ] && export CHROME || fail setup "no Chrome for Testing: web and text cannot run"
  fi
  # og outside a checkout's project (the fused copies, 8956's projects) runs this engine package.
  export OG_ENGINE_PACKAGE="$root/packages/engine"
  # The ignored assets the bundle carries (bundle).
  cp -rn "$BUNDLE/assets/." "$root/"
  # The engine package's native og and addon and its web build, which web and text serve.
  mkdir -p "$root/packages/engine/dist" "$root/packages/engine/native"
  cp -r "$BUNDLE/web/web" "$BUNDLE/web/web-publish" "$root/packages/engine/dist/"
  # By rename, never over a file a running og or addon has mapped.
  for a in "$BUNDLE"/web/native/*; do
    cmp -s "$a" "$root/packages/engine/native/${a##*/}" && continue
    cp "$a" "$root/packages/engine/native/.${a##*/}.new" && mv -f "$root/packages/engine/native/.${a##*/}.new" "$root/packages/engine/native/${a##*/}"
  done
  note "final L4 pass on $(cut -c1-9 "$BUNDLE/sha") ($( [ "$DRY" = 1 ] && echo "DRY RUN on the CPU: wiring only, nothing judged" || echo "GPU host" )), $(date -u +%FT%TZ)"
  note "host: $(nvidia-smi --query-gpu=name,driver_version --format=csv,noheader 2>/dev/null || echo 'no nvidia-smi')"
  note "vulkan: $(vulkaninfo --summary 2>/dev/null | grep -m1 deviceName | sed 's/.*= //' || echo unknown)"
  note "chrome: ${CHROME:-none} $("${CHROME:-false}" --version 2>/dev/null)"
  for a in $only; do
    [[ " $SECTIONS " == *" $a "* ]] || die "run: unknown section $a ($SECTIONS)"
    timed "$a"
  done
  note "failures: $(wc -l < "$OUT/failures.txt") (failures.txt)"
  [ ! -s "$OUT/failures.txt" ]
}

case "${1:-}" in
  bundle) shift; bundle "$@" ;;
  assemble) shift; [ "$#" -eq 2 ] || die "assemble WORK BUNDLE"; assemble "$@" ;;
  run) shift; [ "$#" -ge 2 ] || die "run BUNDLE OUT [--dry] [--only a,b]"; run "$@" ;;
  *) sed -n '2,25p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
esac
