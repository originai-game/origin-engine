import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        0,
        0.21950000524520874,
        0.5356000065803528,
        1
      ]
    ],
    [
      1,
      [
        0,
        0.21950000524520874,
        0.5356000065803528,
        1
      ]
    ]
  ],
  "lifetime": [
    1.666700005531311,
    1.666700005531311
  ],
  "maxParticles": 127,
  "randomColorOverLife": [
    [
      0,
      [
        0.21580000221729279,
        0.21950000524520874,
        0.5356000065803528,
        1
      ]
    ],
    [
      1,
      [
        0.21580000221729279,
        0.21950000524520874,
        0.5356000065803528,
        1
      ]
    ]
  ],
  "rate": 60,
  "shape": {
    "kind": "point"
  },
  "size": [
    0.25,
    0.75
  ],
  "speed": [
    1.2727999687194824,
    5.09119987487793
  ],
  "spread": 180,
  "texture": ref("base/vfx/effekseer-starter/textures/Particle01")
});
