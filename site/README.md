# engine.origingame.ai

The introduction site of Origin Engine: a static Astro site, bilingual
(`/` English, `/zh/` Chinese), in the engine's design language
(`packages/design` tokens) with the platform's brand marks as the engine's
shell vendors them (`packages/engine/shell/brand/`, built by `shell/brand.ts`).

Every picture, film and number is real engine output. A slot that is not yet
final is labelled on the page (`Interim`: real output standing in; `Placeholder`:
footage in production), and `bun scripts/check.ts --final` refuses a build that
still has one.

```sh
bun install
bun scripts/media.ts     # src/media.ts slots → public/m/ + src/generated/media.json (needs OG_SHOWCASE_S3_*)
bun scripts/live.ts      # publish builds of the live scenes → public/live/<name>/ (needs packages/engine built)
bun run build            # → dist/
bun run preview
bun scripts/check.ts     # what still stands in for the final page
```

- `src/media.ts`: every media slot and its source (a showcase archive id, a
  repository file, or `pending` with what will fill it).
- `src/numbers.ts`: the numbers wall, counted from this checkout or read from
  showcase benchmark entries, each with its source; unmeasured ones are pending.
- `src/live.ts`: the live scenes. The hero runs `examples/site-ocean` as a
  publish build in a same-origin, non-interactive frame; the page answers
  Player protocol 2 (`hello`, `start`) and shows the frame on `ready`. Without
  WebGPU, on phones, under reduced motion or save-data, or if the engine has
  not drawn in 25 s, it shows the recording, else the captured frame.

Deployment (not done; needs approval): Cloudflare Pages serving `dist/`, with
`public/m/` on R2 behind `PUBLIC_MEDIA_BASE`. `public/_headers` sets caching.
