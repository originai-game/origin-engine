import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        0.0006000000284984708,
        0.8751000165939331,
        0.9322999715805054,
        1
      ]
    ],
    [
      1,
      [
        0.0006000000284984708,
        0.8751000165939331,
        0.9322999715805054,
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
