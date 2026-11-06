<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-LookInfo"></a>

### `LookInfo` (struct)

A loaded look: where it came from and what applying it sets.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `look` | `Look` | `0` | — | — | — | Its handle. |
| `path` | `string` | `""` | — | — | — | Its asset id. |
| `name` | `string` | `""` | — | — | — | Its name (the file's `name`, else its file name without `.ogt`). |
| `extends` | `readonly string[]` | `[]` | — | — | — | The looks it builds on, nearest first: asset ids under assets/ and built-in preset slugs. |
| `mood` | `string \| null` | `null` | — | — | — | The mood it is drawn in (bright, dark, warm, cold, soft or high-contrast): a preset's own, or the one the game's look asked for, whose layer the cook laid over the preset (docs/art.md §2.10); unset for a look that names none. |
| `resources` | `readonly string[]` | `[]` | — | — | — | The resources it sets, by IDL name (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment, and the art contract: ArtPalette, ArtPixels, ArtTexels, ArtShape, ArtLine, ArtMotion, ArtCamera). Applying it replaces each of them whole: the fields the look names take its values, the rest their defaults. |
| `style` | `Style \| null` | `null` | — | — | — | The UI style its `ui` line names (a game's own or a built-in by slug), loaded with it; applying the look configures the UI with it. |
| `antialias` | `Antialias \| null` | `null` | — | — | — | The antialiasing it asks for. GameDesc.look's is the renderer's when GameDesc.render leaves antialias out; a running renderer keeps its method. |

```ogt
struct LookInfo {
  doc "A loaded look: where it came from and what applying it sets."
  field look Look "Its handle."
  field path String "Its asset id."
  field name String "Its name (the file's `name`, else its file name without `.ogt`)."
  field extends List.String "The looks it builds on, nearest first: asset ids under assets/ and built-in preset slugs."
  field mood Option.String "The mood it is drawn in (bright, dark, warm, cold, soft or high-contrast): a preset's own, or the one the game's look asked for, whose layer the cook laid over the preset (docs/art.md §2.10); unset for a look that names none."
  field resources List.String "The resources it sets, by IDL name (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment, and the art contract: ArtPalette, ArtPixels, ArtTexels, ArtShape, ArtLine, ArtMotion, ArtCamera). Applying it replaces each of them whole: the fields the look names take its values, the rest their defaults."
  field style Option.Style "The UI style its `ui` line names (a game's own or a built-in by slug), loaded with it; applying the look configures the UI with it."
  field antialias Option.Antialias "The antialiasing it asks for. GameDesc.look's is the renderer's when GameDesc.render leaves antialias out; a running renderer keeps its method."
}
```

<sub>`api.ogt` line 5606</sub>
