# Safari Remote Automation verification, 2026-10-25

Read-only verification on the Mac; no settings changed.

`/usr/bin/safaridriver --version` reports Safari 27.2 (22625.2.5.11.1).
`safaridriver --help` says `--enable` applies configuration changes, so
that option was not executed under the instruction to fix nothing.
`defaults read com.apple.Safari AllowRemoteAutomation` and
`defaults read com.apple.WebDriver` both report absent domains; those
reads alone do not establish the automation setting.

Started a temporary `/usr/bin/safaridriver --port <free-local-port>`.
An HTTP `GET /status` returned 200:

```json
{"value":{"message":"","ready":true}}
```

An HTTP `POST /session` with the following request returned 500:

```json
{"capabilities":{"alwaysMatch":{"browserName":"safari","platformName":"mac"}}}
```

```json
{"value":{"error":"session not created","message":"Could not create a session: You must enable 'Allow remote automation' in the Developer section of Safari Settings to control Safari via WebDriver.","stacktrace":""}}
```

Result: **Fail, Mac Safari Remote Automation is not enabled for a WebDriver
session.** Driver readiness is not browser automation readiness. The owner
must enable Allow remote automation in Safari Settings > Developer (called
Develop > Allow Remote Automation on other Safari versions). No checkbox
or configuration was changed. Whether additional authentication setup with
`safaridriver --enable` is needed remains unverified until that blocker clears.

The temporary driver was terminated after the failed session request.
No owner process was stopped. EchoPlay PID 81366 and SimMetalHost PID 95801
were still present. Load averages before this check were 4.56, 5.10, 5.93.
No build, gate, or GPU measurement was started.
