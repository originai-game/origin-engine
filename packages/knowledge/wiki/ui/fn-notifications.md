<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-notifications"></a>

### `og.ui.notifications`

A notification stack (toasts, achievements, rewards, invites): cards anchored in the safe area, newest nearest the anchor's edge, that enter on the style's enter spring from that edge, reflow on its panel spring as others leave (layout animation), and leave sliding out. A card's lifetime drains along its bottom edge; the pointer resting on it holds the time; a press dismisses it, and a swipe toward the edge throws it off. More than max wait in a queue whose time does not run.

Parts: Notification.card[<variant>][<tone>] (the card, partClass hud), Notification.accent (the start edge in the tone), Notification.well[rarity=<tier>] (the icon or art slot), Notification.title, Notification.body, Notification.count (the ×3 chip), Notification.progress and Notification.progressFill, Notification.life (the draining time line), Notification.action. Slots: each item's image (art kind item, achievement or avatar, 1:1).

States: on each card, group life (entering, shown, leaving) and group tone (the item's tone). Motion: enter and exit springs, layout motion for the reflow, an achievement's well pops (emphasis spring). Sounds: notification, reward (achievement, reward) or warning (alert) when a card first shows. Accessibility: each card is a status (an alert: an alert) named by its eyebrow, title and body; its action a button. Probe ids: <id>/<key>, its Title, Body, Count, Action, Life.

Also known as: notification stack, achievement popup, toast stack, in-game alerts

```ts
og.ui.notifications(id: string, desc?: NotificationsDesc): NotificationsResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `NotificationsDesc` | yes | The items and where they stack. |

**Returns** `NotificationsResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn notifications {
  doc "A notification stack (toasts, achievements, rewards, invites): cards anchored in the safe area, newest nearest the anchor's edge, that enter on the style's enter spring from that edge, reflow on its panel spring as others leave (layout animation), and leave sliding out. A card's lifetime drains along its bottom edge; the pointer resting on it holds the time; a press dismisses it, and a swipe toward the edge throws it off. More than max wait in a queue whose time does not run."
  doc "Parts: Notification.card[<variant>][<tone>] (the card, partClass hud), Notification.accent (the start edge in the tone), Notification.well[rarity=<tier>] (the icon or art slot), Notification.title, Notification.body, Notification.count (the ×3 chip), Notification.progress and Notification.progressFill, Notification.life (the draining time line), Notification.action. Slots: each item's image (art kind item, achievement or avatar, 1:1)."
  doc "States: on each card, group life (entering, shown, leaving) and group tone (the item's tone). Motion: enter and exit springs, layout motion for the reflow, an achievement's well pops (emphasis spring). Sounds: notification, reward (achievement, reward) or warning (alert) when a card first shows. Accessibility: each card is a status (an alert: an alert) named by its eyebrow, title and body; its action a button. Probe ids: <id>/<key>, its Title, Body, Count, Action, Life."
  aliases "notification stack, achievement popup, toast stack, in-game alerts"
  host false
  param id String "Its id."
  param desc NotificationsDesc "The items and where they stack."
  returns NotificationsResult
}
```

<sub>Module `ui` line 4592</sub>
