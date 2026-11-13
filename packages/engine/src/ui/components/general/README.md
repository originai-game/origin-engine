# general: built-in components

The pieces most games share whatever their genre: a shop and its purchase
confirmation, achievements, daily quests and a login reward calendar, chat
and friends, as documents over core controls (docs/ui.md §4). They draw what
the game tells them and return intents with stable row keys. They hold no
money or reward logic: the game passes every price, balance, verdict
(sold out, owned, affordable), progress figure, date and countdown, already
formatted, and decides what an intent does.

They are built into the engine: a game's page uses one by name
(`<Shop .../>`) and loading the page loads it, with the `ProgressBar` and
`Spinner` they use. `og ui eject general/shop` copies one into
`assets/ui/general/` to change it; the game's copy of the same name wins.

They compose the generic components rather than repeat them: progress is
`ProgressBar`, a wait is `Spinner`. A leaderboard, notifications and a
results screen are the built-in screens of those names; a menu is `Menu`.

## Structure and appearance

Each member's `<style>` holds structure only: layout, token sizes and
spacing. It sets no colour, font or art, and lands in the `og-components`
layer. A style pack (`og-pack`, above it) supplies every appearance through
the members' named parts (docs/ui.md §3.2), qualified by their states and
`data-*` attributes, for example `Shop::offer[data-sold-out=true]` or
`button::part(DailyRewards\.day)` for a part that is a button. Each member
lists its parts in its `<meta name="parts">`. The plain action buttons
(Claim, Buy, Cancel, Send, Accept, Decline, Invite, Add friend, Try again)
name no part: they are core buttons (`variant="primary"`, `secondary`,
`ghost`) and take the pack's plates. The fantasy pack's
`templates/styles/fantasy/general.css` is the reference dress.

State never rests on colour alone: a refused offer says Sold out, Owned or
Not enough beside a mark, and in its accessible name; an achievement says
Locked or Unlocked with its date, and its tier as a word; a day says the
word of the state the game gave it (Claimed, Missed, Upcoming or Today), or
Claim on the button while today's reward waits; a friend's presence is a
word beside its seal.

Hooks for a pack: `data-featured`, `data-sold-out`, `data-owned` and
`data-affordable` on shop offers (a refused one also carries
`aria-disabled`); `data-short` on prices (the currency the
player lacks); `data-kind` (soldOut, owned, short) on shop statuses;
`data-state` (locked, progress, unlocked), `data-hidden`, `data-tier`
(common, rare, epic, legendary), `data-claimable` and `data-claimed` on
achievement rows, `data-tier` again on the tier and rarity; `data-state`
(active, done, claimed) on quest rows; `data-state` (claimed, today, upcoming, missed) and
`data-milestone` on days; `data-kind` (player, own, system) on chat messages,
senders and text; `data-status` (online, ingame, away, offline) on friend
rows, avatars, statuses and the presence seal, `data-selected` on the
chosen friend's row and `data-invited` on its Invite button.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `Shop` (`shop.svelte`) | `title`, `wallet` keyed `{ id, icon, label, amount }`, `tabs` keyed `{ id, label }`, `tab`, `offers` keyed `{ id, name, icon, detail, badge, featured, soldOut, owned, affordable }`, `prices` keyed `{ id, offer, icon, label, amount, was, short }` (one row per currency of an offer), `selected`, `loading`, `error` | `buy` (an offer's key; a sold out, owned or unaffordable offer keeps the focus, is marked `aria-disabled`, names its verdict and has no buy handler), `choosetab`, `tabNext`/`tabPrev` (E/Q, bumpers), `retry` | `frame`, `band`, `title`, `wallet`, `balance`, `tab`, `offer`, `name`, `badge`, `price`, `status` |
| `ShopConfirm` (`shop-confirm.svelte`) | `title`, `name`, `icon`, `detail`, `prices` keyed `{ id, icon, label, amount, was, after, short }`, `canBuy`, `busy`, `error` | `confirm` (Buy; Enter and pad A press the focused button, Cancel first; silent while busy or refused), `cancel` (Cancel, Escape, pad B) | `frame`, `band`, `title`, `name`, `price`, `error` |
| `Achievements` (`achievements.svelte`) | `title`, `summary`, `done` 0–1, `achievements` keyed `{ id, name, about, icon, state, progress, progressText, date, hidden, rarity, tier, claimable, claimed }` | `claim` (an achievement's key; only while claimable and not claimed) | `frame`, `band`, `title`, `summary`, `row`, `medal`, `name`, `state`, `tier`, `rarity` |
| `DailyQuests` (`daily-quests.svelte`) | `title`, `refresh` (the game's countdown text), `quests` keyed `{ id, name, icon, progress, progressText, reward, rewardIcon, state }`, `loading` | `claim` (a done quest's key) | `frame`, `band`, `title`, `refresh`, `row`, `medal`, `name`, `reward` |
| `DailyRewards` (`daily-rewards.svelte`) | `title`, `refresh`, `days` keyed `{ id, label, icon, amount, state, milestone }`, `canClaim` | `claim` (today's key; the day whose state is `today` is a button only while `canClaim`, every other day is a labelled tile with no handler) | `frame`, `band`, `title`, `refresh`, `day`, `label`, `amount`, `state` |
| `Chat` (`chat.svelte`) | `channels` keyed `{ id, label, unread }`, `channel`, `messages` keyed `{ id, kind, sender, time, text }` oldest first, `draft` (two-way: `bind:draft={$draft}`), `maxLength`, `canSend`, `notice` | `choosetab`, `send` (the Send button; silent while `canSend` is false); the field writes the draft back | `frame`, `tab`, `unread`, `log`, `message`, `sender`, `time`, `text`, `notice`, `compose` |
| `Friends` (`friends.svelte`) | `title`, `requests` keyed `{ id, name, avatar, note }`, `friends` keyed `{ id, name, avatar, status, activity, canInvite, invited }`, `selected` | `accept`, `decline` (a request's key), `choose`, `invite` (a friend's key; a pending invite is disabled), `add` | `frame`, `band`, `title`, `heading`, `request`, `row`, `avatar`, `presence`, `name`, `status` |

Parts are named `<Component>.<part>` (`Shop.offer`). Every member carries a
`fixture:default` and the fixtures of its states (empty, loading, error,
disabled, overflow; Friends also `requests`) that the sheet and the tests mount. Phone layouts come
from the members' container queries: the shop's grid drops to two columns,
the calendar to four, the achievement rows wrap their Claim under the text,
the quest rows drop their medals, and Chat and Friends drop their button
words to the marks.

The purchase confirmation is content, not a dialog: it carries no dialog
role and traps no focus. Show it in a host that owns modality and focus,
such as the built-in confirm screen or the game's own modal. It sizes to
its content up to the height it is given; below that its band, its error and its Cancel and Buy stay put and
the offer and prices scroll between them.

The chat binds no tab keys: a letter typed into its field would otherwise
also reach the page's E/Q actions. Enter in the field sends as Send does
(the form's implicit submission), and not while Send is disabled. Its log scrolls with the newest
message at the foot.

## Review sheet and tests

`og ui sheet general` renders the four review pages from
`packages/engine/sheets/general/`, dressed by the fantasy pack: `store`
(shop, purchase confirmation, login calendar), `progress` (achievements,
daily quests, calendar), `social` (chat, friends) and `states` (the shop
failed, a purchase that failed, no achievements, quests
loading and empty, today's reward already taken, a muted chat, no friends).
Icons come from the shared ink icon sheet; there is no painted art and no
locale table yet.

`bun test tests/general.test.ts` (from `packages/engine`) mounts every
member by name from a game page, from every fixture it declares, at desktop
and phone; checks every intent with its row key and that refused offers
(by pointer, Enter and pad A), days, purchases, claims and sends are silent;
that a shop's selection follows its key through a reorder; that Invite is
absent where the game refuses it; that a hidden achievement leaks its name
neither in its text nor in its accessible name; that typing writes the draft back and
does not switch channels; that the chat log shows its newest line and breaks
a long word; and that each member's markup and its parts list agree.
