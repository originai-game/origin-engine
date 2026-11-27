import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        0.0012000000569969416,
        0.899399995803833,
        0.3391999900341034,
        1
      ]
    ],
    [
      1,
      [
        0.0012000000569969416,
        0.899399995803833,
        0.3391999900341034,
        1
      ]
    ]
  ],
  "lifetime": [
    1.666700005531311,
    1.666700005531311
  ],
  "maxParticles": 127,
  "rate": 60,
  "shape": {
    "kind": "point"
  },
  "size": [
    -0.4000000059604645,
    0.6000000238418579
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle02-b223f51c")
});
