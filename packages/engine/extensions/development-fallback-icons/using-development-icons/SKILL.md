---
name: using-development-icons
description: "Installs temporary vector icons for UI development. Use when a game lacks its own icon art."
---

# Development icon fallbacks

These are starting points, not finished UI. Replace their silhouettes, stroke
language and palette with the game's own art before publishing.

Run `og ui add development-fallback-icons`, then configure the UI once:

```ts
import { fallbackIcons } from "../assets/ui-kits/development-fallback-icons/icons.ts";

og.ui.configure(world, { fallbackIcons });
```

Use `og.ui.icon("Currency", "coin")` or `[icon=coin]` in rich text. A name
already painted by the game's primary atlas keeps that art. All missing names
resolve to the cooked `type vector` assets, never a raster atlas or a font.
`icons.json` lists all 100 names and groups. `LICENSE` is Iconoir's MIT licence.

The `development-fallback-icons` path and `icons.json.developmentFallback`
identify the whole set for publish checks. Do not rename it to evade those
checks. Remove the import and assets when the game's own set is ready.
