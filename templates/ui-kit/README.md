# UI kit demo

Every `og.ui` screen template and the game widget set on one gallery
(`crates/og-ui`, `packages/engine/src/ui-kit`, the IDL module `ui`). Pick a
screen from the list, or `show:<screen>` on the command channel; Back
(Escape, gamepad B, the Gallery button) returns.

```sh
og run templates/ui-kit                       # the window, desktop size
og dev templates/ui-kit                       # the browser, reloading on save
og test templates/ui-kit                      # the playtests below
OG_UI_CAPTURE=1 og test templates/ui-kit      # re-record this tier's reference frames
python3 templates/ui-kit/tools/make-assets.py templates/ui-kit/assets   # regenerate icons and backdrop
```

Commands: `show:<screen>`, `viewport:<desktop|tablet|phone|landscape>`
(lays the UI out for that surface through `og.ui.previewViewport`; render at
`--size 1170x2532` for the 390×844@3 phone), `pad:<up|down|left|right|next|prev|activate|back>`
(a gamepad, through `og.ui.navigate`), `toast:<text>`, `damage:<hp>`, `heal:<hp>`.
`dump_entity:demo` reads the demo's state (the `Demo` component).

Screens: gallery, widgets, mainMenu, pause, settings, platformer, shooter,
puzzle, idle, racing, cutscene, shop, inventory, levels, results,
leaderboard, tutorial, loading, confirm, alert, toasts. `screens/<screen>.png`
shows each at desktop (left) and on a phone in portrait (right).

## The reference source

The demo is where `crates/og-ui/references/<tier>/kit-desktop-<screen>.png`
and `kit-phone-<screen>.png` come from: `tests/frames-desktop.og.ts` and
`tests/frames-phone.og.ts` show each screen, wait for its entrance, take its
frame and compare it with the reference under og-tests' rule (a channel off
by more than 32/255 is an outlier; fail above 0.5 % outliers or RMSE 0.01). A
tier without its own references compares with `metal` and says so.

| Playtest | Proves |
|---|---|
| `frames-desktop`, `frames-phone` | every screen matches its reference frame at 1280×720 and at 390×844@3 with a notch and a home indicator |
| `touch` | `move_to_text:=shop` opens the shop from the scrolled gallery, `Detail/Buy` spends the price and toasts, Escape closes; Share calls the share hook; Tap counts; a generator buys; the dialogue box finishes, then advances a line |
| `keys` | Tab, Tab, Enter opens the second row; in Settings, Down reaches the Master slider, Left twice steps it to 0.9, Escape closes and the save hook runs once |
| `pad` | gamepad intents: down ×4 and A open Settings, B closes; Tab order reaches the select, A opens its menu, down and A choose Hard |
| `drag` | a press on slot 0 dragged to slot 7 moves the potion; Use runs for it |
| `phone` | on a phone the HUD corners sit in the safe area, every touch control is at least 44 CSS px, a tap on Jump answers once |
| `tutorial` | the spotlight's hole contains each step's target, a press on the dimmed part changes nothing, Next walks to Done |
