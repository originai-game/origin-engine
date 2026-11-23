<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Clouds"></a>

### `og.Clouds` (resource)

Volumetric clouds of one kind (`kind`: cumulus by default) over any background, including an authored LDR or HDR Environment, plus thin high cirrus. The same curved-shell march works above, inside or below the deck: above top, see sunlit tops fading with distance. Lit by the DirectionalLight (aim from og.load.environmentSun for an authored sky) and the environment's SH ambient; procedural skies keep their clear-sky lighting. Wind drifts shapes deterministically. Cover dims scene lighting and the clouds cast shadows. Alone clouds bring the built-in sky. Visual only.

See `render §7`.

Token `og.Clouds`, registered under the schema name `og.Clouds`; its value type is `Clouds`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the clouds. |
| `kind` | `CloudKind` | `"cumulus"` | — | — | — | The kind of cloud in the volumetric layer. |
| `coverage` | `number` | `0.45` | 0 … 1 | `ratio` | tunable | How much of the sky the clouds cover: 0 none, 0.3 fair weather, 0.6 broken, 1 overcast. |
| `density` | `number` | `0.04` | 0 … 0.5 | — | tunable | How thick a cloud's body is per metre, reached some tens of metres inside its surface (0.04: cumulus; 0.01: thin and translucent; 0.08: dense storm cloud). |
| `base` | `number` | `1500` | — | `m` | tunable | The cloud base's world y, in metres; negative places it below the game's origin. With og.PhysicalSky, its altitude is the sea-level height of world y 0 for the clouds too. |
| `thickness` | `number` | `900` | ≥ 0 | `m` | tunable | How tall the layer is (0: no cumulus, only the high layer). |
| `scale` | `number` | `9000` | 500 … 200000 | `m` | tunable | The size of the largest cloud shapes. |
| `wind` | `Vector3` | `{ x: 0, y: 0.4, z: 0 }` | — | `mps` | tunable | Their own motion, added to og.Wind's (which carries them twice as fast as it blows at the ground): x and z a drift, y how fast their shapes turn over. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Their albedo: white for fair-weather clouds, a grey for rain clouds. |
| `detail` | `number` | `0.35` | 0 … 1 | `ratio` | tunable | How deep fine noise erodes their surface (0: smooth lumps, 0.35: billows, 1: wispy). |
| `layer` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | The high thin layer's cover (0: none). |
| `layerAltitude` | `number` | `8000` | ≥ 0 | `m` | tunable | The high layer's height (kept above the cumulus). |
| `shadows` | `number` | `1` | 0 … 1 | `ratio` | tunable | How dark the cumulus's shadows fall on the world, drifting with the clouds: 1 the shadows they cast, 0 none (the sun a little dimmer everywhere instead). They fall where the sun casts shadows and fade as the cover closes into an overcast. See `render §7`. |
| `steps` | `number` | `48` | 8 … 256 | — | — | Samples along each view ray through the cumulus: more is smoother and costs more. |

```ogt
resource Clouds {
  doc "Volumetric clouds of one kind (`kind`: cumulus by default) over any background, including an authored LDR or HDR Environment, plus thin high cirrus. The same curved-shell march works above, inside or below the deck: above top, see sunlit tops fading with distance. Lit by the DirectionalLight (aim from og.load.environmentSun for an authored sky) and the environment's SH ambient; procedural skies keep their clear-sky lighting. Wind drifts shapes deterministically. Cover dims scene lighting and the clouds cast shadows. Alone clouds bring the built-in sky. Visual only."
  guide "render §7"
  field enabled bool "Draw the clouds."
  field kind CloudKind "The kind of cloud in the volumetric layer."
  field coverage f32 "How much of the sky the clouds cover: 0 none, 0.3 fair weather, 0.6 broken, 1 overcast." {
    default 0.45
    min 0
    max 1
    unit ratio
    tunable true
  }
  field density f32 "How thick a cloud's body is per metre, reached some tens of metres inside its surface (0.04: cumulus; 0.01: thin and translucent; 0.08: dense storm cloud)." {
    default 0.04
    min 0
    max 0.5
    tunable true
  }
  field base f32 "The cloud base's world y, in metres; negative places it below the game's origin. With og.PhysicalSky, its altitude is the sea-level height of world y 0 for the clouds too." {
    default 1500
    unit m
    tunable true
  }
  field thickness f32 "How tall the layer is (0: no cumulus, only the high layer)." {
    default 900
    min 0
    unit m
    tunable true
  }
  field scale f32 "The size of the largest cloud shapes." {
    default 9000
    min 500
    max 200000
    unit m
    tunable true
  }
  field wind Vector3 "Their own motion, added to og.Wind's (which carries them twice as fast as it blows at the ground): x and z a drift, y how fast their shapes turn over." {
    default 0 0.4 0
    unit mps
    tunable true
  }
  field color Color "Their albedo: white for fair-weather clouds, a grey for rain clouds." {
    default 1 1 1
  }
  field detail f32 "How deep fine noise erodes their surface (0: smooth lumps, 0.35: billows, 1: wispy)." {
    default 0.35
    min 0
    max 1
    unit ratio
    tunable true
  }
  field layer f32 "The high thin layer's cover (0: none)." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field layerAltitude f32 "The high layer's height (kept above the cumulus)." {
    default 8000
    min 0
    unit m
    tunable true
  }
  field shadows f32 "How dark the cumulus's shadows fall on the world, drifting with the clouds: 1 the shadows they cast, 0 none (the sun a little dimmer everywhere instead). They fall where the sun casts shadows and fade as the cover closes into an overcast." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
    guide "render §7"
  }
  field steps u32 "Samples along each view ray through the cumulus: more is smoother and costs more." {
    default 48
    min 8
    max 256
  }
}
```

<sub>Module `atmosphere` line 365</sub>
