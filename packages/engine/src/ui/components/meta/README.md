# meta: built-in components

The pieces most games share whatever their genre: a shop and its purchase
confirmation, achievements, daily quests and a login reward calendar, a
season pass, reward reveals, inbox and news, profiles, chat and friends.
They draw what the game tells them and report intents with stable row
keys. They hold no money, reward or entitlement logic: the game passes
every price, balance, verdict (sold out, owned, affordable, claimed,
expired), progress figure, date and countdown, already formatted, and
decides what an intent does. They never roll rewards, mark messages read
or delete data themselves.

They compose the generic components rather than repeat them: progress is
`ProgressBar`, a wait is `Spinner`. A leaderboard, notifications and a
results screen are the built-in screens of those names; a menu is `Menu`.

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `Achievements` | `meta/achievements` | The achievement list. |
| `Chat` | `meta/chat` | A chat panel. |
| `DailyQuests` | `meta/daily-quests` | Today's quests. |
| `DailyRewards` | `meta/daily-rewards` | A login reward calendar. |
| `Friends` | `meta/friends` | The friends panel. |
| `Inbox` | `meta/inbox` | Messages and their attachments: a list beside the selected message; reports choose, claim, delete and retry. |
| `News` | `meta/news` | News and patch notes in game order, the selected one expanded; reports choose, open and retry. |
| `PassTrack` | `meta/pass-track` | A season pass. |
| `Profile` | `meta/profile` | A player's card: portrait, name, rank, status and figures, with the game's actions under it. |
| `Reveal` | `meta/reveal` | A reward reveal. |
| `ShopConfirm` | `meta/shop-confirm` | The purchase confirmation the game shows after a shop's buy. |
| `Shop` | `meta/shop` | A storefront. |

## Gallery

`bun packages/engine/src/node/gallery.ts --only meta/` draws every member's fixtures under every pack.
