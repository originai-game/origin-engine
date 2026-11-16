# HUD instruments

Default-style review pages for HUD instruments, abilities, buttons, touch
controls and maps. The meters page covers shield, healing,
segmented health, low and critical health, inside values, full stamina and
empty experience. Reserves cover partial life and mana, extended stamina
and exhaustion. Objectives cover partial, full and indeterminate progress,
and ordinary, warning and critical countdowns.

The actions page covers cooldown, charges, unaffordable and locked slots.
Buttons include enabled, disabled, optional holding text and long labels.
Touch includes charged and disabled actions; navigation includes a compass,
a plain map and a rotating player-up map.

Render pages with their commands; mounting without a page command is
not a review of the sheet:

```sh
og ui preview packages/engine/sheets/hud \
  --screen 'meters-default=style:default;page:meters' \
  --screen 'objectives-default=style:default;page:objectives' \
  --screen 'actions-default=style:default;page:actions' \
  --screen 'buttons-default=style:default;page:buttons' \
  --screen 'touch-default=style:default;page:touch' \
  --screen 'navigation-default=style:default;page:navigation' \
  --breakpoints desktop,phonePortrait --locales game --scales 100
```

The pages use the default style, respect the phone safe area and keep the
objective states on a separate page so they remain visible on a phone.
