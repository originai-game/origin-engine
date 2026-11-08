# Component documents

Generic component documents built from og-ui's public elements: the HUD,
feedback, world and pattern pieces that used to be TypeScript builders on
`og.ui`. Copy one with `og ui eject <component>` and edit it in
`assets/ui/<component>.svelte`; a screen then uses it by its declared name
(`<AbilityBar slots={$slots} />`).

They follow the screen conventions (`../screens/README.md`): each declares
`<meta name="component">`, its props and actions, and a
`<meta name="fixture:default">` JSON fixture; structure is scriptless markup,
look is scoped CSS and tokens, rings and bars are inline SVG, world tags are
`<anchor>`, ground marks `<ground>`, and overhead bars and damage numbers are
`<anchor mass>` rows drawn on the GPU. Visible words come from the companion
string table (`<meta name="strings" content="./<component>.en.ogt">`), which
eject copies too.

Expressions compute no geometry, so the game hands in what a builder used to
derive: a chart's line and a path line's route as SVG path data, a radial
menu's slice positions (`at`, `span`), a split view's ratio from its
divider's pointer events, a ground mark's world centre.

`og ui eject --list` prints these with the screens. The component document
test cooks every one from its fixture at desktop and phone sizes.

| Purpose | Documents |
|---|---|
| Ability bars, cooldowns and buffs | `ability-bar`, `ability-slot`, `cooldown`, `status-grid` |
| Meters and resource bars | `stat-bar`, `orb-meter`, `radial-meter` |
| Progress and loading placeholders | `progress-bar`, `progress-ring`, `skeleton-group`, `spinner` |
| Timers and scores | `hud-timer`, `star-bar` |
| Navigation (minimap, compass) | `minimap`, `minimap-view`, `compass` |
| Menus and layout (radial menu, split view, carousel, breadcrumbs) | `radial-menu`, `split-view`, `carousel`, `breadcrumbs` |
| Charts and map overlays | `chart`, `overlay-bar` |
| Data views (inventories, level selects) | `data-grid` |
| Buttons and badges (hero button, touch action, hold to confirm, action bar, ribbon) | `hero-button`, `action-button`, `hold-button`, `action-hints`, `ribbon` |
| World tags over units and items | `nameplate`, `waypoint`, `loot-label`, `speech-bubble`, `interaction-prompt` |
| Ground marks (selection, telegraphs, paths, placement) | `selection-ring`, `telegraph`, `path-line`, `placement-ghost` |
| Mass world elements (overhead bars, damage numbers) | `overhead-bars`, `damage-numbers` |
