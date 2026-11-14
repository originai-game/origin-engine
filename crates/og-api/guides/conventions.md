# Conventions: units, angles, names, rows, anchors

The rules every member of the API follows, each stated once: which unit a number is in, which way an angle is measured, what a name promises, which way a grid's rows run and which corner an anchor counts from. Read it before writing a game. Where a habit from another engine disagrees with a rule here, the rule is what the API does; a member that breaks one says so in its own entry.

The API: `og api show transform`, `og api show physics`, `og api show input`, `og api show nav`, `og api show tilemap`. Which way things face is in `facing`.

## 1. Units

Metres, kilograms and seconds. A time is in seconds unless its name ends
in `Ms` (milliseconds) or `Ticks` (fixed steps): `duration`, `soakTime`
and `recoverTime` are seconds, `holdMs` milliseconds, `maxCatchUpTicks`
steps. Speeds are metres per second, forces newtons. Each entry names its
field's unit.

## 2. Angles

| In | What |
|---|---|
| radians | every rotation, heading, yaw, joint or steering limit, arc, uv rotation and cone |
| degrees | `fov`, slopes, the sky's angles (the sun's path, disc sizes, the aurora), camera roll, and the angles of post effects, styles and the art contract |

- A `fov` is the full angle, edge to edge. A cone is its half-angle, from
  its axis to its edge (og.SpotLight's `angle`, og.Light2D's and
  og.AudioSource's `innerAngle` and `outerAngle`).
- A horizontal direction given as an angle is a heading: about +Y from +Z
  towards +X, whether in radians (og.OrbitCamera's `azimuth`) or in
  degrees (og.DayNight's `azimuth`, og.Aurora's `direction`). `facing §1`
  has the 2D one.

```ts
world.set(lamp, og.SpotLight, { angle: 0.5 });               // radians, half the cone: a beam about 57 degrees across
world.set(hero, og.CharacterController, { maxSlope: 45 });   // a slope: degrees
must(world.resource(og.Camera)).fov = 60;                    // degrees, the whole view top to bottom
```

## 3. Names

- **`quaternion` and `rotation`.** A `quaternion` is a rotation in 3D. A
  field named `rotation` is one angle in radians: a 2D thing's turn, a
  texture's.
- **`layers` and masks.** `layers` are the bits a thing is on; a `mask` or
  a field ending in `Mask` the bits it looks for. In a desc or a filter a
  mask of 0 means every layer (`QueryFilter.mask`, `collisionMask`,
  `avoidMask`, `occluderMask`). og.Collider's `mask` is the exception: it
  is a component's stored field, every bit by default, and 0 collides
  with nothing.
- **`damping`.** How fast motion dies, per second; 0 loses none. Each tick
  the motion keeps 1 / (1 + damping × dt), so a value behaves the same at
  30 and 60 ticks a second. Two families keep their field's own meaning
  under the word: a spring's damping (og.SpringBone, a joint's, a
  suspension's) and a reverb's damping of high frequencies.
- **Input.** `down` is true while it is down; `justPressed` and
  `justReleased` for the one tick it went down or up; `held` once it has
  been down for the action's `holdMs`, a long press.

```ts
const jump = og.input.action(world, "jump");
if (jump?.justPressed) start();  // once per press; `down` would fire every tick, `held` only after holdMs
```

## 4. Grids and rows

| Grid | Row 0 | Rows run |
|---|---|---|
| a tilemap (`TileCell`, `og.tilemap.cellAt`) | the top | down, as Tiled's |
| a nav grid from `og.nav.grid` (`NavCell`) | the bottom, at its origin | up |
| a nav grid from `og.nav.gridFromTilemap` | the top | down, as its map's: a tile's cell is the grid's cell |
| procgen's cell rectangles and cell lists | the top | down |
| a pixel terrain's cells (og.PixelTerrain) | the bottom, at its entity | up |

Columns always run right. Do not flip a row by hand between a tilemap and
the nav grid made from it: `og.nav.cellAt` and `og.tilemap.cellAt` answer
the same cell for one point.

## 5. Anchors and pivots

- A world sprite's `anchor` is a fraction of its size from its
  bottom-left corner, y up as the world's: (0.5, 0) stands it on its feet.
- An image's or a screen element's `pivot` is a fraction from its
  top-left corner, y down as pixels': (0.5, 1) is its bottom centre.

The same pair of numbers names opposite edges in the two: (0.5, 0) is the
bottom centre of a sprite and the top centre of a screen element.

## 6. Colours and handles

- A `Color` is linear RGB. A hex number or a CSS colour is sRGB:
  `color.fromHex(0xff8800)` converts it. Components above 1 are HDR.
- A handle (a texture, a sound, a material) held by a component is 0 for
  none; in a desc the field is `null` or left out.

## 7. Facing

The axes are stated in one place, `facing`: an object faces its +Z, a
camera or a light looks down its -Z, a 2D thing faces +X (`facing §1`);
the calls that turn things and read their directions, with their 2D
twins (`facing §2`); and the signs input reports, the pointer's and the
wheel's included (`facing §3`).
