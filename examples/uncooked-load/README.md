# Uncooked load

One scene, no assets: the game makes a glTF and a PNG in code at run time and
loads them uncooked, the way generated or player-made content arrives.

- `src/make.ts` writes the files: a PNG encoder (stored deflate, CRC and
  Adler checksums, about forty lines) and a `.glb` writer that lathes a vase
  profile into a textured mesh with its glaze texture embedded as a PNG.
- `src/main.ts` loads them with `og.load.gltfData` and `og.load.imageData`
  under a `LoadBudget`, waits for `og.load.status` to say `ready` (decoding
  happens off the engine's thread; on the web the browser decodes), then
  spawns the model with `og.Model` and the drawing as an `og.Sprite`.

Run it with `og run .`. The same files from a player would arrive as bytes
(an upload, a platform store read, a message) and load the same way;
`og.load.release` frees them when the game is done with them.
