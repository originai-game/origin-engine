# Frames: one document inside another

How to show one of the game's documents inside a box of another with `<iframe>`, what crosses into it and what does not, and how to size the box to what the document shows. Read before building a news panel, an event page or a help pane that several screens share.

The API: `og api show ui`. Documents themselves are in `screens`.

## 1. Embedding a document

```svelte
<iframe src="news.svelte" title="News" items={$news} />
```

`src` names a `.svelte` document of the game, relative to the embedding
document (`news.svelte` beside `ui/hud.svelte` is `ui/news.svelte`). The
cook bundles it with its embedder, as it does a component, and refuses a
`src` that names no document of the project. `src` is a literal: a bound
`src={…}` is refused, and so is a URL (`https://…`). An `<iframe>` holds no
content between its tags, and a document never embeds one it is inside.

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
