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
    0.75,
    2
  ],
  "speed": [
    8.485300064086914,
    33.94110107421875
  ],
  "spread": 180,
  "texture": ref("base/vfx/effekseer-starter/textures/Particle01")
});
