# Meta components

The pieces most games share whatever their genre: a shop and its purchase
confirmation, achievements, daily quests and a login reward calendar, chat
and friends, as documents over core controls (docs/ui.md §4). They draw what
the game tells them and return intents with stable row keys. They hold no
money or reward logic: the game passes every price, balance, verdict
(sold out, owned, affordable), progress figure, date and countdown, already
formatted, and decides what an intent does.

They are built into the engine: a game's page uses one by name
(`<Shop .../>`) and loading the page loads it, with the `ProgressBar` and
`Spinner` they use. `og ui eject meta/shop` copies one into
`assets/ui/meta/` to change it; the game's copy of the same name wins.

They compose the generic components rather than repeat them: progress is
`ProgressBar`, a wait is `Spinner`. A leaderboard, notifications and a
results screen are the built-in screens of those names; a menu is `Menu`.

## Structure and appearance

Each member's `<style>` supplies only its HTML layout. Shared role parts
dress its surfaces and text, `data-tone` colours its states, `data-motion`
selects shared motion, and `data-layout` with `data-gap` supplies the layout
primitives and spacing. Role aliases preserve the original public parts.
Optional content uses conditionals; bounded lists use CSS overflow.
A style pack restyles the meaningful
pieces through
the members' named parts (docs/ui.md §3.2), qualified by their states and
`data-*` attributes, for example `Shop::offer[data-sold-out=true]` or
`button::part(DailyRewards\.day)` for a part that is a button. Each member
lists its parts in its `<meta name="parts">`. The shop/social members' plain action buttons
(Claim, Buy, Cancel, Send, Accept, Decline, Invite, Add friend, Try again)
name no part: they are core buttons (`variant="primary"`, `secondary`,
`ghost`) and take the pack's plates. The fantasy pack dresses them
through its tokens alone (`templates/styles/fantasy/base.css`).

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
| `Shop` (`shop.svelte`) | `title`, `kicker`, `wallet` keyed `{ id, icon, label, amount }`, `tabs` keyed `{ id, label, count }`, `tab`, `offers` keyed `{ id, name, icon, detail, rarity, badge, badgeTone, featured, soldOut, owned, affordable }`, `prices` keyed `{ id, offer, icon, label, amount, was, short }` (one row per currency of an offer; a struck price is named with the price it replaces), `selected`, `loading`, `error` | `buy` (an offer's key; a sold out, owned or unaffordable offer keeps the focus, is marked `aria-disabled`, says its verdict on its foot and has no buy handler), `choosetab`, `tabNext`/`tabPrev` (E/Q, bumpers), `retry` | `frame`, `band`, `kicker`, `title`, `wallet`, `balance`, `tab`, `count`, `offer`, `art`, `name`, `detail`, `featured`, `badge`, `foot`, `price`, `former`, `status`, `bracket` |
| `ShopConfirm` (`shop-confirm.svelte`) | `title`, `name`, `icon`, `rarity`, `detail`, `prices` keyed `{ id, icon, label, amount, was, after, short, missing }` (the offer's own currencies: `after` the balance before and after, `missing` how much more a short one takes), `canBuy`, `reason` (why it is refused when no price says it), `busy`, `error` (a failed attempt; Buy stays) | `confirm` (Buy; Enter and pad A press the focused button, Cancel first; silent while busy or refused), `cancel` (Cancel, Escape, pad B) | `frame`, `band`, `title`, `name`, `art`, `detail`, `ledger`, `price`, `former`, `after`, `missing`, `reason`, `error`, `actions` |
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

`og ui sheet meta` renders the four review pages from
`packages/engine/sheets/meta/`, in the default style and fantasy pack: `store`
(shop, purchase confirmation, login calendar), `progress` (achievements,
daily quests, calendar), `social` (chat, friends) and `states` (the shop
failed, a purchase that failed, no achievements, quests
loading and empty, today's reward already taken, a muted chat, no friends).
Icons come from the shared ink icon sheet; the default style uses the
sheet's licensed Instrument Sans font. There is no painted art. Visible
words use the members' companion string tables.
`og ui review meta --style scifi` (or `--style cartoon`) renders the same
pages in that pack, dressed by its tokens
(`templates/styles/<pack>/base.css`).

`bun test tests/meta.test.ts` (from `packages/engine`) mounts every
member by name from a game page, from every fixture it declares, at desktop
and phone; checks every intent with its row key and that refused offers
(by pointer, Enter and pad A), days, purchases, claims and sends are silent;
that a shop's selection follows its key through a reorder; that Invite is
absent where the game refuses it; that a hidden achievement leaks its name
neither in its text nor in its accessible name; that typing writes the draft back and
does not switch channels; that the chat log shows its newest line and breaks
a long word; and that each member's markup and its parts list agree.

## Progression and updates

PassTrack, Reveal, Inbox and News display game-owned progression, rewards,
messages and updates. They never roll rewards, grant entitlement, perform
purchases, mark messages read or delete data themselves. Props have typed
defaults; collections carry stable `id` keys and intents return those keys.

| Component | Inputs and states | Intents |
| --- | --- | --- |
| `PassTrack` | Season and refresh words (a kicker), `level`, progress fraction/text to the next tier, premium entitlement, ordered keyed reward tiles with formatted tier/amount and `locked/available/claimed` state in a premium lane over a free lane, loading/error/empty, disabled | `claim` with reward key, `upgrade`, `retry`. Claims require available state and premium entitlement where applicable: without premium every premium tile reads Locked. Disabled prevents claims/upgrades. |
| `Reveal` | Game-owned `sealed/loading/revealing/revealed`, a `kicker`, the sealed prize's `icon`, rewards with rarity (a socket of its colour and the word) and fresh flags, optional skip, disabled/error | `open`, `skip`, `continue`, `retry`, keyed `inspect`. Rewards stay hidden until revealed. No animation timer or roll runs in the document. |
| `Inbox` | Keyed messages, separate attachments referencing their message id, selected message, loading/error/empty, busy, unread/claimed/expired facts | `choose`, `claim`, `delete` with message key; `retry`. Attachments do not replace the enclosing message key. Busy prevents duplicate mutations; expired/claimed rewards cannot be claimed. |
| `News` | Keyed entries in game order, optional art/action, plain multiline body, selected/expanded id, pinned/unread, disabled/loading/error/empty | `choose`, `open` with article key; `retry`. The game updates selected to expand/collapse. Only the expanded article exposes its action. |

Free/premium, rarity, unread/read, claimed/expired and progress states are
worded as well as exposed through declared parts or `data-*` attributes.
Buttons keep shared hover/focus/press/disabled states. Each PassTrack lane
scrolls sideways on its own; Inbox is side-by-side on desktop and stacked on
a phone; News takes its articles' height up to its host's, then scrolls them
under its heading.
List subjects use a whole-line ellipsis with a title tooltip, and the
selected reader shows the full subject. Loading composes shared Spinner.
These are ordinary lists, not virtualised lists.

The components sheet accepts
`style:default;sample:<member>:<fixture>` at desktop and `phonePortrait`:

| Member | Fixtures |
| --- | --- |
| `passTrack` | `default`, `premium`, `disabled`, `empty`, `loading`, `error` |
| `reveal` | `default`, `loading`, `revealing`, `revealed`, `disabled`, `empty`, `error` |
| `inbox` | `default`, `busy`, `unselected`, `claimed`, `expired`, `overflow`, `empty`, `loading`, `error` |
| `news` | `default`, `collapsed`, `disabled`, `overflow`, `empty`, `loading`, `error` |

The sample inputs put an available free reward beside an entitlement-gated
premium reward, unclaimed/read messages, and a long announcement beside a
collapsed one. Review loading and error separately; the absence of a
mutation button is part of those contracts. Native contract tests are in
`tests/meta-components.test.ts`.
