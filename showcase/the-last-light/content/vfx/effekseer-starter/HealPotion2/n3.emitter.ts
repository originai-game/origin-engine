import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        1,
        0.20839999616146088,
        0.20839999616146088,
        1
      ]
    ],
    [
      0.75,
      [
        1,
        0.20839999616146088,
        0.20839999616146088,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.20839999616146088,
        0.20839999616146088,
        0
      ]
    ]
  ],
  "duration": 0.13330000638961792,
  "gravity": [
    0,
    -36,
    0
  ],
  "lifetime": [
    0.5,
    0.833299994468689
  ],
  "looping": false,
  "maxParticles": 50,
  "rate": 300,
  "shape": {
    "kind": "point"
  },
  "size": [
    0.5,
    0.5
  ],
  "speed": [
    12.267000198364258,
    16.99180030822754
  ],
  "spread": 11.976699829101562,
  "texture": ref("base/vfx/effekseer-starter/textures/Particle01-b96f716d")
});
