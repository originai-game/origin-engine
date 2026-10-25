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
| mkcert | Not on PATH. No root CA found at either default path checked: `~/Library/Application Support/mkcert` or `~/.local/share/mkcert` |
| LAN address | `192.168.1.40`, `en0`. The only non-loopback IPv4 address, so `device.ts` selects it |

Raw commands and output: `phone-preflight.txt`, `safari-preferences.txt`.
No phone performance measurements have run. A Pixel 10 is not the design
budget's 2022 mid-range Android; results on this phone cannot close that
hardware-specific budget without noting the mismatch.

## Owner actions still needed

- On the Mac, enable Safari automation with `safaridriver --enable` and pair
  the iPhone in Xcode. This run did not change either setting.
- On the iPhone, enable Web Inspector and Remote Automation in Safari's
  advanced settings.
- Install mkcert, install its root CA on the Mac, and issue a certificate
  for `192.168.1.40`. Install and explicitly trust the root CA on the iPhone.
  The iPhone must reach that address on the LAN.
- Close the running EchoPlay simulator game and its simulator, or authorize
  stopping them. EchoPlay and SimMetalHost were using CPU and GPU during
  preparation. No owner process was stopped.

The Android USB-debugging authorization was resolved between the two checks.
Real-device runs and desktop GPU measurements remain held pending the
coordinator's confirmation.
