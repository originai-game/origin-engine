# og run --device: evidence

Recorded 2026-10-19 on an Apple M4 Pro (macOS, a 120 Hz display), Google
Chrome for Testing 150, headless, at 1f94224 plus the uncommitted
`og run --device` change.

## Chrome (exercised here)

    cargo build -p og-cli
    OG_ENGINE_PACKAGE=$PWD/packages/engine target/debug/og run --device chrome templates/game --out tests/evidence/P3/device/chrome

`chrome/report.md` and `chrome/device.json`: webgpu (apple, metal-3), first
frame 270 ms against a 2000 ms budget, display interval p50 8.33 / p95 9.82 /
p99 10.26 ms over 601 frames (the display's 120 Hz), engine work p95 0.73 ms,
cross-origin isolated, COOP/COEP/CORP on `/` and every wasm, no finding.

    OG_ENGINE_PACKAGE=$PWD/packages/engine target/debug/og run --device chrome templates/game --webgl --out tests/evidence/P3/device/chrome-webgl2

fails, exit 1, `chrome-webgl2-failure.txt`: on the WebGL2 fallback the engine
panics creating the `overlay glass` pipeline, because `fs_glass` in
crates/og-render/shaders/overlay.wgsl calls `textureNumLevels`, which GLSL ES
3.0 lacks (`Features(TEXTURE_LEVELS)`), since 42d7294. Every WebGL2 page fails
to start until that shader stops asking for the level count (for example a
uniform carrying it). The test's webgl2 case stays red until then.

The test, outside the bun-test gate like every tests/web script (it needs
Chrome; a missing Chrome is an error, never a skip):

    cd packages/engine && bun tests/web/device.ts [--out <dir>] [--chrome <path>]

`device-result.json` is its last run: 15 of 18 checks, the three failures all
the webgl2 case above; the webgpu case and the missing-Chrome refusal pass.
Broken once: with the play-gate tap removed from src/node/device.ts the webgpu
case failed ("a tap on the play gate did not start play"), restored, passed.

## Android (not exercised here)

    og run --device android[:serial] <project>

Needs adb on PATH (Android platform-tools: `brew install
android-platform-tools`), USB debugging on the phone (Settings > About phone >
tap Build number seven times, then Developer options > USB debugging), the
phone authorised for this computer (`adb devices` lists it as `device`), and
Chrome installed and opened once. og forwards the phone's
`localabstract:chrome_devtools_remote` and reverses the serving port, so the
page is `http://localhost:<port>/`, a secure context. With several phones,
name one: `android:<serial>` from `adb devices`.

## iPhone (not exercised here)

    og run --device iphone[:udid] <project> --cert lan.pem --key lan-key.pem

Needs macOS; on the iPhone Settings > Apps > Safari > Advanced > Web Inspector
and Remote Automation on; on the Mac `safaridriver --enable` once and the
phone paired in Xcode (Window > Devices and Simulators). The phone opens the
LAN URL, which is a secure context (needed for cross-origin isolation and
WebGPU) only over HTTPS: `mkcert -install && mkcert <lan-ip>` makes the pair
for `--cert`/`--key`, and the phone must trust mkcert's root CA (AirDrop
`mkcert -CAROOT`/rootCA.pem, install the profile, then Settings > General >
About > Certificate Trust Settings). Without `--cert` the run warns and
measures an unisolated page.

Owed row 40 (the measurement on a real phone) remains the owner's.
