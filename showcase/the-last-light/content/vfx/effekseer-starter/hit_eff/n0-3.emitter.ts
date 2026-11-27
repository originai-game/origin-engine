import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 30,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.16670000553131104,
      [
        1,
        1,
        1,
        0.24899999797344208
      ]
    ],
    [
      0.666700005531311,
      [
        1,
        1,
        1,
        0.24899999797344208
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    0.5,
    0.5
  ],
  "looping": false,
  "maxParticles": 38,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 1,
    "surface": false
  },
  "size": [
    0.03500000014901161,
    0.25
  ],
  "speed": [
    9,
    18
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle01")
});
