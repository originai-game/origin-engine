# Phone preflight, 2026-10-25

Mac runner: macOS 27.2 arm64. Measurement source revision: `1bb341e5`.
The initial tool checks used the existing native executable before the rebuild.
The CLI source at `1bb341e5` confirms that `--preflight` is an internal
`device.ts` option, not an `og run` option.

| Check | Result |
|---|---|
| `og run templates/game --device android --preflight` | Fail, exit 2: unknown option `--preflight` |
| `og run templates/game --device iphone --preflight` | Fail, exit 2: unknown option `--preflight` |
| `bun packages/engine/src/node/device.ts --preflight --device android` | Pass, exit 0. Checks that adb exists, not that a phone is authorized |
| `bun packages/engine/src/node/device.ts --preflight --device iphone` | Pass, exit 0. Checks macOS and safaridriver existence, not Remote Automation or phone pairing |
| adb | `/opt/homebrew/bin/adb`, 1.0.41, platform tools 37.0.1-15733141 |
| Android at 01:28 UTC | Serial `58051FDCR007QL`, unauthorized |
| Android at 01:44 UTC | Same serial authorized (`device`), Pixel 10, Android 17, Chrome 155.0.8059.87 |
| safaridriver | `/System/Cryptexes/App/usr/bin/safaridriver`, Safari 27.2 (22625.2.5.11.1) |
| Safari Remote Automation enabled | Not measurable from these checks. User and system Safari/WebDriver preference domains were absent. No enable command or WebDriver session was run |
| mkcert at 01:28 UTC | Not on PATH. No root CA found at either default path checked: `~/Library/Application Support/mkcert` or `~/.local/share/mkcert` |
| mkcert after owner setup | `/opt/homebrew/bin/mkcert`; `rootCA.pem` exists in `~/Library/Application Support/mkcert`. Owner reports the iPhone trusts this CA |
| Physical iPhone | Xcode `devicectl` lists iPhone Air `00008150-000A70511144401C` as `unavailable`. `xctrace` lists iPhone 27.0.1 under Devices Offline. The connected iPhone 17 is a simulator |
| LAN address | `192.168.1.40`, `en0`. The only non-loopback IPv4 address, so `device.ts` selects it |

Raw commands and output: `phone-preflight.txt`, `safari-preferences.txt`,
`phone-followup.txt`. The requested CLI `--preflight` returning exit 2 is a
CLI defect, not a missing tool. No engine code was changed to fix it here.
A Pixel 10 is not the design
budget's 2022 mid-range Android; results on this phone cannot close that
hardware-specific budget without noting the mismatch.

## Owner actions still needed

- Plug in and unlock the iPhone, accept the trust prompt, and pair it in Xcode.
  Enable Safari automation with `safaridriver --enable` if it is not enabled.
  This run did not change either setting.
- On the iPhone, enable Web Inspector and Remote Automation in Safari's
  advanced settings.
- A certificate for `192.168.1.40` is needed for the HTTPS run. The iPhone
  must reach that address on the LAN. The owner has installed mkcert and
  reports the root CA trusted on both devices.
- Close the running EchoPlay simulator game and its simulator, or authorize
  stopping them. EchoPlay and SimMetalHost were using CPU and GPU during
  preparation. No owner process was stopped.

The Android USB-debugging authorization was resolved between the two checks.
The coordinator authorized Android first-frame and CPU measurements, Chrome
physics hashes and stats, and terrain settle measurements under recorded load.
Desktop GPU budgets and iPhone frame times remain held pending confirmation
that the simulator is stopped.
