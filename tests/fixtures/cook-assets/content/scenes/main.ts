import { file, ref } from "@origingame/engine/author";

export default file.scene({
  name: "Main",
  entities: [
    { name: "ground", components: { "og.Transform": { position: [0, 0, 0], quaternion: [0, 0, 0, 1], scale: [1, 1, 1] } } },
    { name: "dropship", model: ref("base/models/dropship"), components: { "og.Transform": { position: [0, 1.5, 0], quaternion: [0, 0, 0, 1], scale: [1, 1, 1] } } },
    {
      name: "trickster", model: ref("base/models/walker"),
      components: { "og.Transform": { position: [2, 0, 0], quaternion: [0, 0.7071068, 0, 0.7071068], scale: [1, 1, 1] } },
      children: [{ name: "marker", components: { "og.Transform": { position: [0, 2.2, 0], quaternion: [0, 0, 0, 1], scale: [0.25, 0.25, 0.25] } } }],
    },
  ],
});
