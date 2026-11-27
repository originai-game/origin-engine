import { file, ref } from "@origingame/engine/author";

// 256 m valley: a northern mountain, lake basin and walkable knoll.
export default file.terrain({
  source: { kind: "image", source: "assets/terrain/valley.png", height: [-10, 50] },
  size: [256, 256],
  control: ["assets/terrain/valley-control.png"],
  heightBlend: 0.15,
  layers: [
    { albedo: ref("base/textures/terrain-grass"), tile: 5, slope: [0,30], slopeFade: 6, altitude: [3.4,1000], altitudeFade: 0.8 },
    { albedo: ref("base/textures/terrain-path"), tile: 3, slope: [0,45] },
    { albedo: ref("base/textures/terrain-rock"), tile: 9, triplanar: true, slope: [32,90], slopeFade: 6 },
    { albedo: ref("base/textures/terrain-sand"), tile: 4, slope: [0,36], altitude: [-100,3.6], altitudeFade: 0.8 },
  ],
});
