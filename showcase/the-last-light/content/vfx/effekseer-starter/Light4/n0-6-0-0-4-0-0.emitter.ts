import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        0.21950000524520874,
        0,
        1,
        0.5019999742507935
      ]
    ],
    [
      1,
      [
        0.21950000524520874,
        0,
        1,
        0.5019999742507935
      ]
    ]
  ],
  "duration": 0.33329999446868896,
  "lifetime": [
    1.666700005531311,
    1.666700005531311
  ],
  "looping": false,
  "maxParticles": 300,
  "rate": 720,
  "shape": {
    "kind": "point"
  },
  "size": [
    0.5,
    0.5
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle02-b223f51c")
});
