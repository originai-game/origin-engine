# Frames: one document inside another

How to show a document inside a box of another with `<iframe>`: one of the game's own, or one another origin serves (an event page on the game's CDN), what crosses into it and what does not, and how to size the box to what the document shows. Read before building a news panel, an event page or a help pane that several screens share.

The API: `og api show ui` (`og.ui.frameMessages`). Documents themselves are in `screens`.

## 1. Embedding a document

```svelte
<iframe src="news.svelte" title="News" items={$news} />
```

`src` names a `.svelte` document of the game, relative to the embedding
document (`news.svelte` beside `ui/hud.svelte` is `ui/news.svelte`). The
cook bundles it with its embedder, as it does a component, and refuses a
`src` that names no document of the project. `src` is a literal: a bound
`src={…}` is refused. An `<iframe>` holds no content between its tags, and
a document never embeds one it is inside. A URL `src` is a remote frame
(§4).

The frame's document is a screen or component like any other, with its own
`<meta name="prop:…">` declarations. Its data comes in only through the
`<iframe>`'s other attributes, which are its props, read in the embedder's
context as a component's are (`items={$news}` follows the store). It reads
no store of the mount: `{$score}` inside a frame is an error. Its actions
and events reach the game as a component's do.

## 2. What crosses the frame's edge

A frame is styled as the game is, not as the page around it is:

- The game's style pack and colour scheme apply inside, so a frame looks
  like the game.
- No rule of the embedding document matches inside the frame, and no
  selector inside sees the page around it: `#Page .row` in the pack does
  not match a row of a frame inside `#Page`; `.row` does.
- Nothing is inherited across the edge. A `font-size` or `color` set on
  the embedder stops at the frame; the frame's document starts from the
  defaults.

To pass a look in, pass a prop, or let the style pack carry it.

## 3. Sizing: `frame-sizing`

An `<iframe>` is a replaced box: with no CSS `width` or `height` it is
300 × 150 px, as HTML's is. CSS sizes it as any box.

`frame-sizing` (CSS Sizing 4) lets the frame take its document's size
instead, on an axis CSS leaves `auto`:

| Value | The frame's natural size |
|---|---|
| `auto` (default) | 300 × 150 |
| `content-height`, `content-width` | that axis is the document's laid-out content size |
| `content-block-size`, `content-inline-size` | the same, by the writing mode |

The content size is used only when the embedded document opts in with
`<meta name="responsive-embedded-sizing">`; without it the frame keeps
300 × 150. The size follows the document as it changes (a row inserted
grows the frame in the same build), with no tick of delay.

```svelte
<iframe id="News" src="news.svelte" items={$news} />
<style>#News { width: 320px; frame-sizing: content-height; }</style>
```

A frame clips what overflows it (`overflow: clip`); give it `overflow:
auto` to scroll a document taller than a fixed height.

## 4. Remote frames

```svelte
<iframe src="https://events.example.com/og/halloween.svelte" title="Event"
        integrity="sha384-…" heading={$eventName} />
```

A `src` that is an `https://` URL (`http://` only on localhost) shows a
`.svelte` document that origin serves. List the origin in project.ogt; the
cook refuses any other:

```text
frames "https://events.example.com"
```

Listing `frames` is also what puts remote frames into the game's engine:
`og build` ships the engine with them only to a game that lists at least
one origin, so a game without the line downloads none of their code.
`og.ui.remoteFrames(world)` answers whether this engine has them; where
it answers false, a remote `src` is refused when the document loads and
`og.ui.frameMessages` stays empty. Local frames work in every game.

`og build` writes the list into og-build.json, so the platform's page can
reach those origins. The server answers CORS reads from the game's origin
(`Access-Control-Allow-Origin`). `crossorigin`, `integrity` and
`referrerpolicy` work as on HTML's iframe and as url()'s request
modifiers: `integrity` pins the document's bytes, so a changed document is
refused.

The engine fetches the document when the embedding one loads, and the
frame shows only its own box until it arrives; an executor (`og test`,
`og render`, captures) waits for it, so frames do not depend on the
network. Inside the document, relative paths are beside its URL: a
`url("art/banner.png")` is fetched from the same directory, and an
`<iframe src="more.svelte">` is another remote frame from there (frames
nest at most 3 deep).

A remote document runs no code of the game and reaches no state of it:

- It reads only its props (the iframe's attributes) and `$ui`; one that
  reads any other `$store` is refused.
- It may use the engine's built-in components the game loaded, not the
  game's own, and no cooked asset of the game.
- Its events never reach `og.ui.docEvents`. Each click, changed control,
  `emit` or action arrives in `og.ui.frameMessages(world)` with the frame's
  path and the origin it came from. Check `origin`, then act: a gameplay
  effect goes through `og.input.intent`, so replays carry it. A text field's
  value leaves only this way.
- `bind:` on a remote frame is refused: pass values in, read messages out.
- Budgets: 256 KiB of source, 4,096 nodes, 64 images. A document past them
  shows nothing.

A refused or failed document is one warning naming its URL and why (an
origin not in `frames`, CORS, integrity, an HTTP status, a store it
reads), and the frame shows nothing; the page around it builds as usual.
`og.loading.idle` stays false while a frame's document is being fetched.
