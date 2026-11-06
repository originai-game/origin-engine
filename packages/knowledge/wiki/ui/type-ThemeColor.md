<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ThemeColor"></a>

### `ThemeColor` (enum)

A colour role of the UI style in effect (the game's style, or the default style before it picks one).

A string: one of the variants below. The zero value is `"backdrop"`.

| Variant | Doc |
|---|---|
| `"backdrop"` | The screen behind the UI where nothing else is. |
| `"well"` | A recessed ground inside a panel: wells, map frames, empty slots. |
| `"panel"` | Panels and cards. |
| `"panelHover"` | A panel under the pointer. |
| `"panelRaised"` | Dialogs, menus and tooltips. |
| `"line"` | Hairlines and frames. |
| `"lineStrong"` | Stronger frames: high contrast, map borders, scrollbars. |
| `"ink"` | The main ink. |
| `"inkMuted"` | Secondary ink: captions, keys, inactive tabs. |
| `"inkFaint"` | The faintest ink that still reads: placeholders, hints. |
| `"inkInverse"` | Ink on a ground of ink (a selected chip). |
| `"control"` | Neutral control fills: secondary buttons, tracks, fields. |
| `"controlStrong"` | A neutral control hovered or pressed; selection. |
| `"primary"` | The one directed action: primary buttons, a slider's fill, a switch that is on, the caret. |
| `"primaryHover"` | primary hovered. |
| `"primaryActive"` | primary pressed. |
| `"onPrimary"` | Ink on primary. |
| `"primarySoft"` | A translucent wash of primary. |
| `"highlight"` | Emphasised ink: a speaker's name, a price, a link, the player's map dot. |
| `"selection"` | The selected row, tab or card, independent of primary actions. |
| `"onSelection"` | Ink on the selection ground. |
| `"accent"` | Decorative emphasis, independent of primary and selection. |
| `"sceneInk"` | Scheme-aware text drawn over the scene. |
| `"scenePlate"` | Scheme-aware plate behind HUD content. |
| `"sceneShadow"` | Separation shadow under text over the scene. |
| `"focus"` | The focus ring of key and pad navigation. |
| `"knob"` | Slider and switch knobs. |
| `"hud"` | HUD capsules over the game (scores, timers, toasts). |
| `"scrim"` | Behind a dialog. |
| `"shadow"` | Drop shadows. |
| `"sheen"` | A skeleton's passing sheen. |
| `"success"` | Success. |
| `"warn"` | Warnings. |
| `"danger"` | Danger and errors. |
| `"info"` | Information. |
| `"health"` | A health meter. |
| `"mana"` | A mana meter. |
| `"xp"` | An experience meter. |
| `"gold"` | Currency: coins, gold, prices that can be paid. |

```ogt
enum ThemeColor {
  doc "A colour role of the UI style in effect (the game's style, or the default style before it picks one)."
  variant backdrop "The screen behind the UI where nothing else is."
  variant well "A recessed ground inside a panel: wells, map frames, empty slots."
  variant panel "Panels and cards."
  variant panelHover "A panel under the pointer."
  variant panelRaised "Dialogs, menus and tooltips."
  variant line "Hairlines and frames."
  variant lineStrong "Stronger frames: high contrast, map borders, scrollbars."
  variant ink "The main ink."
  variant inkMuted "Secondary ink: captions, keys, inactive tabs."
  variant inkFaint "The faintest ink that still reads: placeholders, hints."
  variant inkInverse "Ink on a ground of ink (a selected chip)."
  variant control "Neutral control fills: secondary buttons, tracks, fields."
  variant controlStrong "A neutral control hovered or pressed; selection."
  variant primary "The one directed action: primary buttons, a slider's fill, a switch that is on, the caret."
  variant primaryHover "primary hovered."
  variant primaryActive "primary pressed."
  variant onPrimary "Ink on primary."
  variant primarySoft "A translucent wash of primary."
  variant highlight "Emphasised ink: a speaker's name, a price, a link, the player's map dot."
  variant selection "The selected row, tab or card, independent of primary actions."
  variant onSelection "Ink on the selection ground."
  variant accent "Decorative emphasis, independent of primary and selection."
  variant sceneInk "Scheme-aware text drawn over the scene."
  variant scenePlate "Scheme-aware plate behind HUD content."
  variant sceneShadow "Separation shadow under text over the scene."
  variant focus "The focus ring of key and pad navigation."
  variant knob "Slider and switch knobs."
  variant hud "HUD capsules over the game (scores, timers, toasts)."
  variant scrim "Behind a dialog."
  variant shadow "Drop shadows."
  variant sheen "A skeleton's passing sheen."
  variant success "Success."
  variant warn "Warnings."
  variant danger "Danger and errors."
  variant info "Information."
  variant health "A health meter."
  variant mana "A mana meter."
  variant xp "An experience meter."
  variant gold "Currency: coins, gold, prices that can be paid."
}
```

<sub>`api.ogt` line 15714</sub>
