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

`chrome-webgl2/report.md` and `chrome-webgl2/device.json`, re-run on the
p3-culling branch after main's overlay glass fix (cf6aeaf) was merged in:
webgl2 (ANGLE Metal, Apple M4 Pro), first frame 1125 ms, display interval
p50 8.34 / p95 37.27 / p99 50 ms over 338 frames, engine work p95 1.41 ms,
cross-origin isolated, COOP/COEP/CORP on `/` and every wasm. The one finding
(p95 frame time) is the machine, not the page: the run shared it at a load
average of 74, and the engine work stayed under 1.7 ms every frame. Before
that fix this run failed, exit 1: the `overlay glass` pipeline panicked on
WebGL2 because `fs_glass` called `textureNumLevels`, which GLSL ES 3.0 lacks.

The test, outside the bun-test gate like every tests/web script (it needs
Chrome; a missing Chrome is an error, never a skip):

    cd packages/engine && bun tests/web/device.ts [--out <dir>] [--chrome <path>]

`device-result.json` is its last run, on p3-culling after the merge: 27 of 27
checks, the webgl2 case included (it was 15 of 18 before the glass fix, the
three failures all the webgl2 case).
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

For the iPhone og serves on 0.0.0.0 and picks the LAN address the phone
opens from this machine's IPv4 addresses: link-local 169.254.x.x dropped,
tunnel, bridge, VM and container interfaces (utun, tun, tap, ppp, ipsec, gif,
stf, bridge, vmnet, vboxnet, docker, veth, awdl, llw, anpi, zt, tailscale,
wg and the like) last, RFC 1918 addresses before public ones, and Ethernet or
Wi-Fi names (en, eth, wlan, wl) before others; the run prints the URL and the
interface it chose. When a VPN or several interfaces make that pick wrong, or
the phone can reach only one particular interface, `--host <ip>` names the
address instead, and the mkcert certificate has to name that same address.
`--host`, like `--cert`/`--key`, is the iPhone's only: `--device chrome` and
`--device android` open the page at localhost and refuse it.

Owed row 40 (the measurement on a real phone) remains the owner's.
